---
name: ptd-cli
description: Use when the user asks to search torrents, download torrents, check PT site user info or stats, manage downloaders, query cross-seeding tasks, or interact with the PT-Depiler browser extension from the terminal
---

# ptd-cli

CLI for the PT-Depiler browser extension via Chrome Native Messaging. All operations execute through the running browser extension, reusing its cookies, site definitions, and downloader configurations.

Binary location: `C:\Users\leish\bin\ptd.exe`

## Prerequisites

- PT-Depiler extension running in browser
- Native host installed: `ptd install --browser chrome --extension-id <ID>`
- Verify: `ptd status` should show a healthy instance

## IMPORTANT: Always discover site IDs and downloader IDs first

**NEVER guess site IDs or downloader IDs.** They are internal identifiers that don't always match the site's display name (e.g., PTerClub's site ID is `pter`, not `pterclub`).

Before performing ANY site-specific or downloader-specific operation, you MUST first retrieve the available sites and downloaders:

### List all configured sites

```bash
# Search for test on all enabled sites - this reveals all site IDs
ptd search "test" --timeout 10 2>&1 | head -20
# The stderr output shows lines like: [pter] success: 5 results
# The bracketed values are the actual site IDs to use
```

### List all configured downloaders

```bash
# Get extension metadata which contains downloader configs
# Use the raw protocol: send getExtStorage with key "metadata"
# Then look at the "clients" field for downloader IDs and names
ptd downloader config <downloader-id> --pretty
```

Since there is no `ptd downloader list` command, check the download history to discover downloader IDs:

```bash
ptd download-history --pretty | head -50
# Look for "downloaderId" fields in the output
```

## Commands

### Search

```bash
ptd search "keyword"                          # All configured sites
ptd search "keyword" --site pter              # Specific site (use discovered site ID!)
ptd search "keyword" --site pter --site mteam # Multiple sites
ptd search "keyword" --pretty                 # Human-readable output
```

Results are cached for `ptd download <index>`.

### Download

```bash
ptd download 0 --downloader <downloader-id>   # By index from last search
ptd download --option-file ./dl.json           # Full option payload
```

The downloader ID is the internal key (e.g. `6JsFPshE1tXYVUVmh_ZL_`), not the human name. Discover it from download history or site config.

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
ptd search "test" --site pter | jq '.[0].title'
ptd user-info current pter | jq '.ratio'
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
- **Always discover IDs first**: never assume site IDs or downloader IDs — always query them from the extension before use
