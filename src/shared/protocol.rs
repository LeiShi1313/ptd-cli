use serde::{Deserialize, Serialize};

/// Top-level message envelope used on both the Native Messaging channel and the Unix socket.
/// Discriminated by the `type` field in JSON.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HostMessage {
    Hello(HelloMessage),
    Request(RequestMessage),
    Response(ResponseMessage),
}

/// Sent by the extension immediately after `connectNative()` succeeds.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelloMessage {
    pub instance_id: String,
    pub browser: String,
    pub extension_id: String,
    pub version: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// A request from CLI -> daemon -> extension.
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestMessage {
    pub id: String,
    pub method: String,
    #[serde(default = "default_params")]
    pub params: serde_json::Value,
}

fn default_params() -> serde_json::Value {
    serde_json::Value::Null
}

/// A response from extension -> daemon -> CLI.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

/// Structured error returned in a response.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
}

/// Instance registry metadata written by the daemon, read by the CLI.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceRegistry {
    pub instance_id: String,
    pub browser: String,
    pub extension_id: String,
    pub version: String,
    pub socket_path: String,
    pub connected_at: String,
    pub last_seen_at: String,
}

impl ResponseMessage {
    /// Create a success response.
    pub fn success(id: String, result: serde_json::Value) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: String, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id,
            result: None,
            error: Some(ErrorInfo {
                code: code.into(),
                message: message.into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_round_trip() {
        let msg = HostMessage::Hello(HelloMessage {
            instance_id: "abc-123".into(),
            browser: "chrome".into(),
            extension_id: "ext-id".into(),
            version: "0.0.5".into(),
            capabilities: vec!["bridge-v1".into()],
        });
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"hello""#));
        let parsed: HostMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            HostMessage::Hello(h) => assert_eq!(h.instance_id, "abc-123"),
            _ => panic!("expected Hello"),
        }
    }

    #[test]
    fn request_round_trip() {
        let json = r#"{"type":"request","id":"r1","method":"getSiteSearchResult","params":{"siteId":"chdbits"}}"#;
        let parsed: HostMessage = serde_json::from_str(json).unwrap();
        match parsed {
            HostMessage::Request(r) => {
                assert_eq!(r.id, "r1");
                assert_eq!(r.method, "getSiteSearchResult");
                assert_eq!(r.params["siteId"], "chdbits");
            }
            _ => panic!("expected Request"),
        }
    }

    #[test]
    fn response_success_round_trip() {
        let resp = ResponseMessage::success("r1".into(), serde_json::json!({"status": "ok"}));
        let msg = HostMessage::Response(resp);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"response""#));
        assert!(json.contains(r#""result""#));
        assert!(!json.contains(r#""error""#));
    }

    #[test]
    fn response_error_round_trip() {
        let resp = ResponseMessage::error("r1".into(), "TIMEOUT", "timed out");
        let msg = HostMessage::Response(resp);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""error""#));
        assert!(!json.contains(r#""result""#));
    }
}
