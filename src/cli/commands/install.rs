use anyhow::{Context, Result};
use clap::Args;

use crate::shared::constants::NATIVE_HOST_NAME;
use crate::shared::paths::BrowserFamily;

#[derive(Args)]
pub struct InstallArgs {
    /// Target browser family
    #[arg(long)]
    pub browser: BrowserFamily,

    /// Extension ID (required for Chrome-family browsers)
    #[arg(long)]
    pub extension_id: Option<String>,
}

pub fn run(args: InstallArgs) -> Result<()> {
    let host_binary = std::env::current_exe()
        .context("cannot determine current executable path")?
        .parent()
        .context("executable has no parent directory")?
        .join(if cfg!(windows) { "ptd-host.exe" } else { "ptd-host" });

    if !host_binary.exists() {
        anyhow::bail!(
            "ptd-host binary not found at {}. Make sure both binaries are in the same directory.",
            host_binary.display()
        );
    }

    let host_path = host_binary
        .canonicalize()
        .context("failed to resolve ptd-host path")?;

    let manifest = if args.browser.is_firefox() {
        serde_json::json!({
            "name": NATIVE_HOST_NAME,
            "description": "PT-Depiler CLI Native Messaging Host",
            "path": host_path.to_string_lossy(),
            "type": "stdio",
            "allowed_extensions": ["ptdepiler.ptplugins@gmail.com"]
        })
    } else {
        let ext_id = args.extension_id.as_deref().unwrap_or_else(|| {
            eprintln!("--extension-id is required for Chrome-family browsers.");
            eprintln!("Find it at chrome://extensions with Developer Mode enabled.");
            std::process::exit(1);
        });
        serde_json::json!({
            "name": NATIVE_HOST_NAME,
            "description": "PT-Depiler CLI Native Messaging Host",
            "path": host_path.to_string_lossy(),
            "type": "stdio",
            "allowed_origins": [format!("chrome-extension://{ext_id}/")]
        })
    };

    let manifest_path = args.browser.native_host_manifest_path();
    let manifest_dir = manifest_path.parent().unwrap();

    std::fs::create_dir_all(manifest_dir)
        .with_context(|| format!("failed to create directory {}", manifest_dir.display()))?;

    let json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(&manifest_path, &json)
        .with_context(|| format!("failed to write manifest to {}", manifest_path.display()))?;

    println!("Native messaging host manifest installed:");
    println!("  Path: {}", manifest_path.display());
    println!("  Host binary: {}", host_path.display());
    println!();
    println!("Restart your browser or reload the PT-Depiler extension to activate.");

    Ok(())
}
