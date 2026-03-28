mod shared;
mod host;

use tracing_subscriber::EnvFilter;

fn main() {
    if atty::is(atty::Stream::Stdin) {
        eprintln!("This process is meant to be launched by a browser via Native Messaging.");
        eprintln!("Run 'ptd status' to inspect available instances.");
        std::process::exit(1);
    }

    // Initialize logging
    let log_dir = shared::paths::logs_dir();
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "ptd-host.log");
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("PTD_LOG_LEVEL").add_directive("info".parse().unwrap()))
        .with_writer(file_appender)
        .with_ansi(false)
        .init();

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    if let Err(e) = rt.block_on(host::daemon::run()) {
        tracing::error!("daemon exited with error: {e:#}");
        std::process::exit(1);
    }
}
