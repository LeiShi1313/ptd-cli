use anyhow::{Context, Result};
use chrono::Utc;

use crate::shared::paths;
use crate::shared::protocol::{HelloMessage, InstanceRegistry};

/// Attempt to connect to an existing IPC endpoint to check if it's alive.
///
/// On Unix, this connects to a filesystem socket path.
/// On Windows, this connects to a named pipe.
fn is_ipc_alive(ipc_name: &str) -> bool {
    use interprocess::local_socket::prelude::*;
    #[cfg(unix)]
    {
        use interprocess::local_socket::GenericFilePath;
        match ipc_name.to_fs_name::<GenericFilePath>() {
            Ok(name) => LocalSocketStream::connect(name).is_ok(),
            Err(_) => false,
        }
    }
    #[cfg(windows)]
    {
        use interprocess::local_socket::GenericNamespaced;
        match ipc_name.to_ns_name::<GenericNamespaced>() {
            Ok(name) => LocalSocketStream::connect(name).is_ok(),
            Err(_) => false,
        }
    }
}

/// Publish instance registry file and prepare the IPC endpoint.
/// Returns an error if a live daemon already owns this instance.
pub fn publish(hello: &HelloMessage) -> Result<()> {
    let ipc_name = paths::instance_ipc_name(&hello.instance_id);
    let registry_path = paths::instance_registry_path(&hello.instance_id);

    // Ensure directories exist
    std::fs::create_dir_all(paths::instances_dir())
        .context("failed to create instances directory")?;
    std::fs::create_dir_all(paths::logs_dir())
        .context("failed to create logs directory")?;

    // Check for existing live daemon
    if is_ipc_alive(&ipc_name) {
        anyhow::bail!(
            "another daemon is already running for instance {}",
            hello.instance_id
        );
    }

    // Remove stale socket file on Unix (named pipes on Windows have no file to remove)
    #[cfg(unix)]
    {
        let socket_file = std::path::Path::new(&ipc_name);
        if socket_file.exists() {
            std::fs::remove_file(socket_file).ok();
        }
    }
    if registry_path.exists() {
        std::fs::remove_file(&registry_path).ok();
    }

    let now = Utc::now().to_rfc3339();
    let registry = InstanceRegistry {
        instance_id: hello.instance_id.clone(),
        browser: hello.browser.clone(),
        extension_id: hello.extension_id.clone(),
        version: hello.version.clone(),
        socket_path: ipc_name,
        connected_at: now.clone(),
        last_seen_at: now,
    };

    let json = serde_json::to_string_pretty(&registry)
        .context("failed to serialize registry")?;
    std::fs::write(&registry_path, json)
        .context("failed to write registry file")?;

    Ok(())
}

/// Remove instance IPC artifacts and registry files.
pub fn cleanup(instance_id: &str) {
    // Remove socket file on Unix (named pipes on Windows have no file to remove)
    #[cfg(unix)]
    {
        let ipc_name = paths::instance_ipc_name(instance_id);
        let socket_file = std::path::Path::new(&ipc_name);
        if socket_file.exists() {
            std::fs::remove_file(socket_file).ok();
        }
    }

    let registry_path = paths::instance_registry_path(instance_id);
    if registry_path.exists() {
        std::fs::remove_file(&registry_path).ok();
    }
}

/// List all registry entries from disk, regardless of health.
pub fn list_all() -> Result<Vec<InstanceRegistry>> {
    let dir = paths::instances_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&dir).context("failed to read instances directory")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(registry) = serde_json::from_str::<InstanceRegistry>(&content) {
                    entries.push(registry);
                }
            }
        }
    }
    Ok(entries)
}

/// Check if an instance's IPC endpoint is alive.
pub fn is_instance_healthy(registry: &InstanceRegistry) -> bool {
    is_ipc_alive(&registry.socket_path)
}

/// Remove stale registry entries whose sockets are gone.
pub fn prune_stale() -> Result<usize> {
    let entries = list_all()?;
    let mut pruned = 0;
    for entry in &entries {
        if !is_instance_healthy(entry) {
            cleanup(&entry.instance_id);
            pruned += 1;
        }
    }
    Ok(pruned)
}
