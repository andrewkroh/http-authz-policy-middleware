# Changelog System Guide

This project uses [git-cliff](https://git-cliff.org/) for automatic changelog generation based on conventional commits.

## Quick Reference

### Commit Message Format

```
<type>(<scope>): <subject>
```

**Types:**
- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation
- `test:` - Tests
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `style:` - Code style
- `chore:` - Maintenance
- `ci:` - CI/CD changes

**Examples:**
```bash
git commit -m "feat: add regex negation support"
git commit -m "fix(eval): handle empty header values"
git commit -m "docs: update expression language reference"
```

## How It Works

1. **Write conventional commits** - Follow the format above
2. **Commits are categorized** - git-cliff groups by type automatically
3. **Changelog generated** - During release, CHANGELOG.md is updated
4. **Keep a Changelog format** - Output follows standard format

## Configuration

- **Config file**: `cliff.toml`
- **Changelog**: `CHANGELOG.md`
- **Workflow**: `.github/workflows/changelog.yml`
- **Helper script**: `scripts/generate-changelog.sh`

## For Contributors

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for detailed guidelines.

## For Maintainers

### Preview Unreleased Changes

```bash
git cliff --unreleased
```

### Update CHANGELOG.md Locally

```bash
# For unreleased changes
./scripts/generate-changelog.sh --write

# For a specific version
./scripts/generate-changelog.sh --write v0.2.0
```

### During Release

The release workflow automatically:
1. Generates changelog for the new version
2. Updates CHANGELOG.md
3. Includes changelog in GitHub release notes

## What Gets Included

**Included:**
- `feat:` commits → Features section
- `fix:` commits → Bug Fixes section
- `docs:` commits → Documentation section
- `perf:` commits → Performance section
- `refactor:` commits → Refactoring section
- Other conventional types → Appropriate sections

**Excluded:**
- Non-conventional commits (filtered out)
- `chore(deps)` commits (dependency updates)
- `chore(release)` commits (release preparation)
- `chore(pr)` commits (PR automation)

## Resources

- [git-cliff Documentation](https://git-cliff.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Keep a Changelog](https://keepachangelog.com/)
