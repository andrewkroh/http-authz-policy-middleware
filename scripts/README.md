# Scripts Directory

This directory contains utility scripts for maintaining and releasing the HTTP Authorization Policy Middleware project.

## Available Scripts

### `check-license-headers.sh`

Validates that all Rust source files have the required MIT license header.

**Usage:**
```bash
./scripts/check-license-headers.sh
```

**Exit Codes:**
- `0` - All files have valid license headers
- `1` - One or more files are missing license headers

**What it checks:**
- All `.rs` files in `src/` and `tests/` directories
- Verifies MIT license header is present
- Shows which files are missing headers

---

### `generate-changelog.sh`

Helper script for generating changelog entries using git-cliff.

**Usage:**
```bash
# Preview unreleased changes
./scripts/generate-changelog.sh

# Generate changelog for a specific version
./scripts/generate-changelog.sh v0.2.0
```

**Features:**
- Generates changelog from conventional commits
- Supports both unreleased and tagged versions
- Preview mode before updating CHANGELOG.md
- Automatically categorizes commits by type

**Requirements:**
- `git-cliff` must be installed: `cargo install git-cliff`

---

### `prepare-release.sh`

Automated release preparation script that runs pre-release checks and guides you through the release process.

**Usage:**
```bash
./scripts/prepare-release.sh v0.2.0
```

**What it does:**
1. Validates version format (must be `vX.Y.Z`)
2. Updates `Cargo.toml` version if needed
3. Runs pre-release checks:
   - Unit tests (`cargo test`)
   - Code formatting (`cargo fmt --check`)
   - Linting (`cargo clippy`)
   - Integration tests (if available)
4. Builds release binary
5. Verifies WASM plugin size
6. Previews changelog
7. Provides git tag instructions

**Example:**
```bash
$ ./scripts/prepare-release.sh v0.2.0

======================================================================
Preparing release: v0.2.0
======================================================================

[1/8] Validating version format...
✓ Version format valid: v0.2.0

[2/8] Checking Cargo.toml version...
✓ Cargo.toml version matches: 0.2.0

[3/8] Running tests...
✓ All tests passed

[4/8] Checking code formatting...
✓ Code is properly formatted

[5/8] Running clippy...
✓ No clippy warnings

[6/8] Building release...
✓ Release build successful

[7/8] Verifying plugin...
✓ Plugin size: 187 KB

[8/8] Previewing changelog...

To create the release:
  git tag v0.2.0
  git push origin v0.2.0
```

---

### `rewrite-history.sh`

Rewrites Git commit history to update author/committer information for all commits.

**Usage:**
```bash
# Preview mode (shows what will change, no modifications)
./scripts/rewrite-history.sh preview
# OR
./scripts/rewrite-history.sh

# Rewrite mode (actually modifies history)
./scripts/rewrite-history.sh rewrite
```

**Configuration:**
The script is currently configured to rewrite all commits to:
- **Name:** Andrew Kroh
- **Email:** id-github@andrewkroh.com

**Preview Mode:**
Shows:
- Current unique authors and committers
- Preview of first 10 commits
- Total number of commits to rewrite
- What the new author/committer will be
- No changes are made

**Rewrite Mode:**
1. Creates backup branch: `backup-before-rewrite`
2. Rewrites all commit history using `git filter-branch` or `git-filter-repo`
3. Updates both author and committer fields
4. Preserves commit messages, dates, and content
5. Verifies the rewrite was successful
6. Provides next steps for force pushing

**Safety Features:**
- Requires clean working directory (no uncommitted changes)
- Creates automatic backup branch before rewriting
- Requires explicit confirmation (`yes`) before proceeding
- Verifies results after rewriting
- Uses faster `git-filter-repo` if available

**Next Steps After Rewrite:**
```bash
# 1. Review the changes
git log --pretty=format:'%h %an <%ae> %s' | head -20

# 2. Compare with backup
git log backup-before-rewrite --pretty=format:'%h %an <%ae> %s' | head -20

# 3. Force push to remote (this updates GitHub)
git push --force --all origin
git push --force --tags origin

# 4. If something went wrong, restore from backup
git reset --hard backup-before-rewrite
```

**Warning:**
- This rewrites Git history - ALL commit SHAs will change
- After force pushing, collaborators must re-clone the repository
- Open pull requests will need to be recreated
- Cannot be easily undone after force pushing to remote

**When to Use:**
- Updating author identity (work email → personal email)
- Removing sensitive information from history
- Consolidating authorship across commits

---

## Script Development Guidelines

When creating new scripts:

1. **Add shebang**: Use `#!/usr/bin/env bash` for portability
2. **Use strict mode**: Include `set -euo pipefail`
3. **Make executable**: `chmod +x scripts/your-script.sh`
4. **Add help text**: Support `-h` or `--help` flag
5. **Validate inputs**: Check for required tools/files
6. **Use colors**: For clear output (info, success, warning, error)
7. **Document**: Add entry to this README

## Common Color Codes

Scripts use ANSI color codes for output:

```bash
RED='\033[0;31m'      # Errors
GREEN='\033[0;32m'    # Success
YELLOW='\033[1;33m'   # Warnings
BLUE='\033[0;34m'     # Info
NC='\033[0m'          # No Color (reset)
```

## Dependencies

Most scripts require:
- Bash 4.0+
- Git
- Standard Unix tools (grep, sed, awk)

Additional requirements per script:
- `generate-changelog.sh`: git-cliff
- `prepare-release.sh`: cargo, rustc, git-cliff
- `rewrite-history.sh`: git (optionally git-filter-repo for faster operation)

---

**Last Updated:** 2026-02-12
