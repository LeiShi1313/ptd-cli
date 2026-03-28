use anyhow::{Context, Result};

use crate::host::registry;
use crate::shared::protocol::InstanceRegistry;

/// Discover and select an instance.
///
/// - If `explicit` is Some, find that instance (supports prefix match).
/// - If exactly one healthy instance exists, auto-select it.
/// - If zero healthy instances, return error (exit code 2).
/// - If multiple healthy instances and none selected, return error (exit code 3).
pub fn select_instance(explicit: Option<&str>) -> Result<InstanceRegistry> {
    registry::prune_stale().ok();
    let all = registry::list_all().context("failed to list instances")?;
    let healthy: Vec<_> = all
        .into_iter()
        .filter(|r| registry::is_instance_healthy(r))
        .collect();

    if let Some(prefix) = explicit {
        let matches: Vec<_> = healthy
            .into_iter()
            .filter(|r| r.instance_id.starts_with(prefix))
            .collect();
        match matches.len() {
            0 => anyhow::bail!("no healthy instance matching '{prefix}'"),
            1 => Ok(matches.into_iter().next().unwrap()),
            _ => {
                eprintln!("Multiple instances match '{prefix}':");
                for m in &matches {
                    eprintln!("  {} ({})", m.instance_id, m.browser);
                }
                std::process::exit(3);
            }
        }
    } else {
        match healthy.len() {
            0 => {
                eprintln!("No running PT-Depiler instance found.");
                eprintln!("Make sure the browser is open with the PT-Depiler extension.");
                std::process::exit(2);
            }
            1 => Ok(healthy.into_iter().next().unwrap()),
            _ => {
                eprintln!("Multiple PT-Depiler instances found. Use --instance <id> or PTD_INSTANCE to select one:");
                for h in &healthy {
                    eprintln!("  {} ({}, ext: {})", h.instance_id, h.browser, h.extension_id);
                }
                std::process::exit(3);
            }
        }
    }
}
