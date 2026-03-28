use anyhow::Result;

use crate::host::registry;

pub fn run() -> Result<()> {
    registry::prune_stale().ok();
    let all = registry::list_all()?;

    if all.is_empty() {
        println!("No PT-Depiler instances found.");
        println!("Make sure the browser is open with the PT-Depiler extension and native host installed.");
        return Ok(());
    }

    for entry in &all {
        let healthy = registry::is_instance_healthy(entry);
        let status = if healthy { "healthy" } else { "stale" };
        println!(
            "  {} [{}] browser={} ext={} since={}",
            &entry.instance_id[..8],
            status,
            entry.browser,
            entry.extension_id,
            entry.connected_at,
        );
    }

    let healthy_count = all.iter().filter(|e| registry::is_instance_healthy(e)).count();
    println!();
    println!("{} instance(s), {} healthy", all.len(), healthy_count);

    Ok(())
}
