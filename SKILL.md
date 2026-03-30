---
name: ptd-cli
description: Use when the user asks to search torrents, download torrents, check PT site user info or stats, manage downloaders, query cross-seeding tasks, or interact with the PT-Depiler browser extension from the terminal
---

# ptd-cli

CLI for the PT-Depiler browser extension via Chrome Native Messaging. All operations execute through the running browser extension, reusing its cookies, site definitions, and downloader configurations.

## Before you start

Run `ptd status` to confirm a healthy connection to the browser extension. If it fails, the user needs to ensure the browser is running with PT-Depiler loaded and the native host registered (`ptd install`).

## CRITICAL: Always discover site IDs and downloader IDs first

**NEVER guess site IDs or downloader IDs.** They are internal identifiers that often don't match the site's display name (e.g., PTerClub = `pter`, not `pterclub`; M-Team = `mteam`, not `m-team`).

Before performing ANY site-specific or downloader-specific operation, you MUST first retrieve the available IDs:

### Discover site IDs

```bash
# Run a search with no --site flag. stderr prints each site ID:
#   [pter] success: 5 results
#   [mteam] success: 12 results
# The bracketed values are the actual site IDs.
ptd search "test" 2>&1 | grep '^\[' | sed 's/\[\(.*\)\].*/\1/'
```

### Discover downloader IDs

```bash
# Check download history — downloaderId fields contain the IDs
ptd download-history --pretty
```

## Commands

### Search

```bash
ptd search "keyword"                          # All configured sites
ptd search "keyword" --site <site-id>         # Specific site
ptd search "keyword" --site a --site b        # Multiple sites
ptd search "keyword" --pretty                 # Human-readable output
```

Results are cached for `ptd download <index>`.

### Download

```bash
ptd download 0 --downloader <downloader-id>   # By index from last search
ptd download --option-file ./dl.json           # Full option payload
```

The downloader ID is an internal key (e.g. `6JsFPshE1tXYVUVmh_ZL_`), not the human name.

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
ptd user-info current <site-id> | jq '.ratio'
```

## Exit Codes

- 0: success
- 1: command failed
- 2: no healthy instance (browser not running or extension not loaded)
- 3: multiple instances, use `--instance` to select
