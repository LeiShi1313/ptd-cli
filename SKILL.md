---
name: ptd-cli
description: Use when the user asks to search torrents, download torrents, check PT site user info or stats, manage downloaders, query cross-seeding tasks, or interact with the PT-Depiler browser extension from the terminal
---

# ptd-cli

CLI for the PT-Depiler browser extension via Chrome Native Messaging. All operations execute through the running browser extension, reusing its cookies, site definitions, and downloader configurations.

Binary location: `~/workspace/created/ptd-cli/target/release/ptd`

## Prerequisites

- PT-Depiler extension running in browser
- Native host installed: `ptd install --browser chrome --extension-id <ID>`
- Verify: `ptd status` should show a healthy instance

## Commands

### Search

```bash
ptd search "keyword"                          # All configured sites
ptd search "keyword" --site chdbits           # Specific site
ptd search "keyword" --site a --site b        # Multiple sites
ptd search "keyword" --pretty                 # Human-readable output
```

Results are cached for `ptd download <index>`.

### Download

```bash
ptd download 0 --downloader <downloader-id>   # By index from last search
ptd download --option-file ./dl.json           # Full option payload
```

The downloader ID is the internal key (e.g. `6JsFPshE1tXYVUVmh_ZL_`), not the human name. Find it via `ptd downloader config <name>` or `ptd download-history --pretty`.

### User Info

```bash
ptd user-info current <site-id>               # Live stats (ratio, bonus, etc.)
ptd user-info history <site-id>               # Historical snapshots
```

### Downloader

```bash
ptd downloader status <id>                    # dl/up speed
ptd downloader config <id>                    # Full config (address, type, etc.)
ptd downloader version <id>
```

### Other

```bash
ptd site config <site-id>                     # Site settings
ptd download-history                          # List all download history
ptd keep-upload list                          # Cross-seeding tasks
ptd status                                    # Running browser instances
```

## Global Options

```
--instance <id>    Select instance (prefix match). Env: PTD_INSTANCE
--timeout <secs>   Default 30
--pretty           Human-readable JSON
--table            Table format for lists
```

Default output is compact JSON, pipe to `jq` for filtering:

```bash
ptd search "test" --site chdbits | jq '.[0].title'
ptd user-info current chdbits | jq '.ratio'
```

## Exit Codes

- 0: success
- 1: command failed
- 2: no healthy instance (browser not running or extension not loaded)
- 3: multiple instances, use `--instance` to select

## Key Patterns

- **Cross-site search**: omit `--site` to search all configured sites
- **Download workflow**: search → pick index → download with downloader ID
- **Instance auto-select**: works automatically with one browser; use `--instance` prefix match with multiple
- **Extension must be initialized**: open the extension options page at least once to populate site/downloader config
