mod shared;
mod host;

use tracing_subscriber::EnvFilter;

fn main() {
    // On Windows, Chrome's Native Messaging may provide handles that atty
    // misidentifies as a TTY, causing the daemon to exit immediately.
    #[cfg(not(windows))]
    if atty::is(atty::Stream::Stdin) {
        eprintln!("This process is meant to be launched by a browser via Native Messaging.");
        eprintln!("Run 'ptd status' to inspect available instances.");
        std::process::exit(1);
    }

    // On Windows, stdin/stdout default to text mode which corrupts binary data
    // (e.g., \r\n translation, 0x1A treated as EOF). Native Messaging uses a
    // 4-byte binary length prefix, so we must switch to binary mode.
    #[cfg(windows)]
    {
        unsafe extern "C" {
            fn _setmode(fd: i32, mode: i32) -> i32;
        }
        // 0 = stdin, 1 = stdout, _O_BINARY = 0x8000
        unsafe {
            _setmode(0, 0x8000);
            _setmode(1, 0x8000);
        }
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
