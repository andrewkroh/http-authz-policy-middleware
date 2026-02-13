# Release Guide

Quick reference for creating releases of the HTTP Authorization Policy Middleware.

## Creating a Release

Releases are fully automated via GitHub Actions. To create a release:

1. Go to **Actions** > **Release** workflow
2. Click **Run workflow**
3. Enter the version (e.g., `v0.2.0`)
4. Click **Run workflow**

The workflow handles everything automatically:
- Runs quality checks (clippy, formatting, license headers)
- Runs unit and integration tests
- Updates `Cargo.toml` version (if needed) and commits to main
- Creates and pushes the git tag
- Builds the optimized WASM plugin
- Generates release notes from conventional commits
- Creates the GitHub Release with artifacts

## Version Format

- **Valid:** `v0.1.0`, `v1.2.3`, `v2.0.0`
- **Invalid:** `0.1.0`, `v1.2`, `release-1.0`

Pattern: `vMAJOR.MINOR.PATCH`

## Release Artifacts

Each release includes a single artifact:

| File | Description |
|------|-------------|
| `http-authz-policy-middleware-vX.Y.Z.zip` | Plugin package (plugin.wasm, .traefik.yml, LICENSE) |

## Versioning

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (v2.0.0) - Breaking changes
- **MINOR** (v0.2.0) - New features (backward-compatible)
- **PATCH** (v0.1.1) - Bug fixes (backward-compatible)

## Troubleshooting

### Release workflow fails

Check:
- GitHub Actions logs for errors
- Version format matches `vX.Y.Z`
- All CI checks pass on main
- WASM target builds successfully

### Tag already exists

The workflow will fail if the tag already exists. To re-release:

1. Delete the tag: `git tag -d v0.1.0 && git push origin :refs/tags/v0.1.0`
2. Delete the GitHub release (via web UI)
3. Re-run the workflow

## Resources

- [Full Release Documentation](../CLAUDE.md#release-process)
- [Traefik Plugin Catalog](https://plugins.traefik.io/)
- [Semantic Versioning](https://semver.org/)
