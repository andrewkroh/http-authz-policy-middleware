# Release Guide

Quick reference for creating releases of the HTTP Authorization Policy Middleware.

## Prerequisites

- [x] All tests passing
- [x] Code formatted (`cargo fmt`)
- [x] No clippy warnings
- [x] Integration tests passing
- [x] Documentation up to date
- [x] All commits follow conventional commit format

## Quick Release Process

### 1. Prepare Release

Use the automated helper script:

```bash
./scripts/prepare-release.sh v0.2.0
```

Or manually:
- Update version in `Cargo.toml`
- Run `cargo test`
- Run `cargo fmt --check`
- Run `cargo clippy --target wasm32-wasip1`
- Run `make release && cd integration-test && ./run-tests.sh`

### 2. Commit Version Bump (if needed)

```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"
git push origin main
```

### 3. Create and Push Tag

```bash
git tag v0.2.0
git push origin v0.2.0
```

### 4. Automated Release

GitHub Actions will automatically:
- Build WASM plugin
- Generate changelog
- Create GitHub Release
- Package plugin for Traefik catalog

## Tag Format

- **Valid:** `v0.1.0`, `v1.2.3`, `v2.0.0`
- **Invalid:** `0.1.0`, `v1.2`, `release-1.0`

Pattern: `v[MAJOR].[MINOR].[PATCH]`

## Release Artifacts

Each release includes:

| File | Description |
|------|-------------|
| `http-authz-policy-middleware-vX.Y.Z.zip` | Plugin package for Traefik catalog |
| `plugin.wasm` | Compiled WASM binary |
| `.traefik.yml` | Plugin manifest |
| `CHANGELOG.md` | Full changelog |

## Versioning

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (v2.0.0) - Breaking changes
- **MINOR** (v0.2.0) - New features (backward-compatible)
- **PATCH** (v0.1.1) - Bug fixes (backward-compatible)

## Troubleshooting

### Tag already exists

```bash
git tag -d v0.1.0
git tag v0.1.0
git push origin v0.1.0
```

### Release workflow fails

Check:
- GitHub Actions logs for errors
- Tag format matches `vX.Y.Z`
- All CI checks pass on main
- WASM target builds successfully

### Need to update release

1. Delete the tag: `git tag -d v0.1.0 && git push origin :refs/tags/v0.1.0`
2. Delete the GitHub release (via web UI)
3. Fix issues and create tag again

## Resources

- [Full Release Documentation](../CLAUDE.md#release-process)
- [Traefik Plugin Catalog](https://plugins.traefik.io/)
- [Semantic Versioning](https://semver.org/)
