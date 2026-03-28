use anyhow::Result;
use clap::Args;

use crate::shared::paths::BrowserFamily;

#[derive(Args)]
pub struct UninstallArgs {
    /// Target browser family
    #[arg(long)]
    pub browser: BrowserFamily,
}

pub fn run(args: UninstallArgs) -> Result<()> {
    let manifest_path = args.browser.native_host_manifest_path();

    if manifest_path.exists() {
        std::fs::remove_file(&manifest_path)?;
        println!("Removed: {}", manifest_path.display());
    } else {
        println!("No manifest found at {}", manifest_path.display());
    }

    Ok(())
}
