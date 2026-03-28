use std::collections::HashMap;

use tokio::sync::oneshot;
use tracing::warn;

use crate::shared::protocol::ResponseMessage;

/// Routes responses from the browser back to the CLI client that sent the request.
pub struct Router {
    pending: HashMap<String, oneshot::Sender<ResponseMessage>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }

    /// Register a pending request. Returns the receiver that the CLI handler should await.
    pub fn register(&mut self, request_id: String) -> oneshot::Receiver<ResponseMessage> {
        let (tx, rx) = oneshot::channel();
        self.pending.insert(request_id, tx);
        rx
    }

    /// Deliver a response to the waiting CLI client. Returns `true` if delivered.
    pub fn deliver(&mut self, response: ResponseMessage) -> bool {
        if let Some(tx) = self.pending.remove(&response.id) {
            if tx.send(response).is_err() {
                warn!("CLI client disconnected before receiving response");
            }
            true
        } else {
            warn!(id = %response.id, "received response for unknown request");
            false
        }
    }

    /// Fail all pending requests with a disconnect error. Called during shutdown.
    pub fn fail_all(&mut self, message: &str) {
        for (id, tx) in self.pending.drain() {
            let _ = tx.send(ResponseMessage::error(id, "DISCONNECTED", message));
        }
    }

    /// Number of in-flight requests.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_and_deliver() {
        let mut router = Router::new();
        let rx = router.register("r1".into());
        assert_eq!(router.pending_count(), 1);

        let resp = ResponseMessage::success("r1".into(), serde_json::json!("ok"));
        assert!(router.deliver(resp));
        assert_eq!(router.pending_count(), 0);

        let received = rx.await.unwrap();
        assert_eq!(received.id, "r1");
        assert!(received.result.is_some());
    }

    #[tokio::test]
    async fn deliver_unknown_id_returns_false() {
        let mut router = Router::new();
        let resp = ResponseMessage::success("unknown".into(), serde_json::json!("ok"));
        assert!(!router.deliver(resp));
    }

    #[tokio::test]
    async fn fail_all_sends_errors() {
        let mut router = Router::new();
        let rx1 = router.register("r1".into());
        let rx2 = router.register("r2".into());

        router.fail_all("shutting down");
        assert_eq!(router.pending_count(), 0);

        let resp1 = rx1.await.unwrap();
        assert_eq!(resp1.error.unwrap().code, "DISCONNECTED");

        let resp2 = rx2.await.unwrap();
        assert_eq!(resp2.error.unwrap().code, "DISCONNECTED");
    }
}
