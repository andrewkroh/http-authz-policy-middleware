# Changelog System Documentation

This document provides a comprehensive overview of the changelog system implemented for the HTTP Authorization Policy Middleware project.

## Overview

The project uses **[git-cliff](https://git-cliff.org/)**, a highly customizable changelog generator written in Rust, to automatically generate changelogs based on conventional commit messages.

### Key Benefits

- **Automatic**: Changelogs generated from commit history
- **Consistent**: Follows Keep a Changelog format
- **Lightweight**: Single configuration file, no build dependencies
- **Low maintenance**: No manual changelog fragments to manage
- **Conventional Commits**: Enforces good commit message discipline
- **Rust-native**: Perfect fit for a Rust project

## System Architecture

```
Developer Commits (Conventional Format)
    ↓
Git History
    ↓
git-cliff (reads cliff.toml config)
    ↓
CHANGELOG.md (Keep a Changelog format)
    ↓
GitHub Release Notes
```

## Components

### 1. Configuration: `cliff.toml`

The main configuration file that defines:
- Changelog header and footer templates
- Commit parsing rules (conventional commits)
- Commit grouping by type (feat → Features, fix → Bug Fixes, etc.)
- Filters to exclude non-conventional commits and noise

### 2. Changelog: `CHANGELOG.md`

The generated changelog file that:
- Follows Keep a Changelog format
- Contains an [Unreleased] section for uncommitted changes
- Organizes commits by version and category
- Includes timestamps for each release

### 3. GitHub Actions: `.github/workflows/changelog.yml`

A manual workflow that:
- Can be triggered via workflow_dispatch
- Generates changelog for unreleased changes or specific tags
- Creates a pull request with the updated CHANGELOG.md
- Useful for manual changelog updates or verification

### 4. Helper Script: `scripts/generate-changelog.sh`

A convenience script for local development:
```bash
# Preview unreleased changes
./scripts/generate-changelog.sh

# Update CHANGELOG.md with unreleased changes
./scripts/generate-changelog.sh --write

# Generate changelog for a specific version
./scripts/generate-changelog.sh --write v0.2.0
```

### 5. Documentation

- **CONTRIBUTING.md**: Comprehensive guide for contributors
- **CLAUDE.md**: Developer workflow documentation
- **.github/CHANGELOG_GUIDE.md**: Quick reference guide
- **docs/CHANGELOG_SYSTEM.md**: This document (system architecture)

## Conventional Commit Format

All commit messages must follow this format:

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

### Commit Types and Mapping

| Type | Description | Changelog Section |
|------|-------------|-------------------|
| `feat` | New feature | Features |
| `fix` | Bug fix | Bug Fixes |
| `docs` | Documentation | Documentation |
| `perf` | Performance improvement | Performance |
| `refactor` | Code refactoring | Refactoring |
| `style` | Code style changes | Styling |
| `test` | Test changes | Testing |
| `chore` | Maintenance tasks | Miscellaneous Tasks |
| `ci` | CI/CD changes | Miscellaneous Tasks |
| `revert` | Revert previous commit | Revert |

### Commit Filtering

**Included in changelog:**
- All commits following conventional format
- Breaking changes (marked with `!` or `BREAKING CHANGE:`)

**Excluded from changelog:**
- Non-conventional commits (don't follow the format)
- `chore(deps)` - Dependency updates
- `chore(release)` - Release preparation
- `chore(pr)` - PR automation

### Examples

```bash
# Feature
git commit -m "feat: add regex negation support in expressions"
git commit -m "feat(parser): implement OR operator precedence"

# Bug fix
git commit -m "fix: correct case-sensitive header comparison"
git commit -m "fix(eval): handle empty header values correctly"

# Documentation
git commit -m "docs: update expression language reference"
git commit -m "docs(api): add examples for allOf function"

# Breaking change
git commit -m "feat!: change expression syntax for function calls"
# or
git commit -m "feat: change expression syntax

BREAKING CHANGE: Function calls now require parentheses"
```

## Workflow for Contributors

### During Development

1. **Write code** with proper tests and documentation
2. **Commit changes** using conventional commit format:
   ```bash
   git commit -m "feat: add new function to expression language"
   ```
3. **Push to fork** and open pull request
4. **Changelog is automatically generated** during release

### What NOT to Do

- ❌ Don't manually edit CHANGELOG.md (except for corrections)
- ❌ Don't create changelog fragment files
- ❌ Don't skip conventional commit format
- ❌ Don't use vague commit messages like "fix stuff" or "update"

## Workflow for Maintainers

### Preview Changes

Before a release, preview the changelog:

```bash
# View unreleased changes
git cliff --unreleased

# Or use helper script
./scripts/generate-changelog.sh
```

### Local Testing

Test changelog generation locally:

```bash
# Install git-cliff
cargo install git-cliff

# Generate changelog for unreleased changes
./scripts/generate-changelog.sh --write

# Review changes
git diff CHANGELOG.md

# Revert if needed
git checkout CHANGELOG.md
```

### During Release

The release workflow (Phase 21) will automatically:

1. Run git-cliff to generate changelog for the new version
2. Update CHANGELOG.md
3. Create GitHub release with changelog as release notes
4. Tag the release

## Integration with GitHub Actions

### Manual Changelog Update

Use the workflow_dispatch trigger:

1. Go to Actions → Generate Changelog
2. Click "Run workflow"
3. Enter tag name (optional, leave empty for unreleased)
4. Review the generated PR
5. Merge when ready

### Automatic During Release

The release workflow will:

1. Checkout repository with full history
2. Install git-cliff
3. Generate changelog for new version
4. Update CHANGELOG.md
5. Include in release notes

## Troubleshooting

### Issue: git-cliff not installed

**Solution:**
```bash
cargo install git-cliff
```

### Issue: Commits not appearing in changelog

**Possible causes:**
- Commit doesn't follow conventional format
- Commit matches a filter rule (e.g., `chore(deps)`)
- Commit is in a skipped tag

**Solution:**
- Review commit message format
- Check cliff.toml filter rules
- Use `git cliff --unreleased -vv` for verbose output

### Issue: Wrong changelog section

**Problem:** Commit appears in wrong section

**Solution:**
- Verify commit type is correct
- Check cliff.toml `commit_parsers` configuration
- Commit type determines the section

### Issue: Breaking changes not highlighted

**Solution:**
- Use `feat!:` or `fix!:` in commit message
- Or add `BREAKING CHANGE:` in commit body

## Configuration Reference

### cliff.toml Structure

```toml
[changelog]
header = "..."        # Changelog file header
body = "..."          # Template for each version section
footer = "..."        # Changelog file footer
trim = true           # Remove leading/trailing whitespace

[git]
conventional_commits = true        # Parse conventional commits
filter_unconventional = true       # Exclude non-conventional
split_commits = false              # Don't split multi-line commits
commit_preprocessors = [...]       # Regex preprocessing
commit_parsers = [...]             # Commit type → section mapping
filter_commits = false             # Don't filter matched commits
sort_commits = "oldest"            # Sort order within sections
```

### Customization

To customize the changelog format:

1. Edit `cliff.toml`
2. Modify the `body` template (uses Tera template engine)
3. Update `commit_parsers` to change grouping
4. Test locally: `git cliff --unreleased`
5. Commit changes

## Comparison with Alternatives

| Feature | git-cliff | release-please | conventional-changelog |
|---------|-----------|----------------|------------------------|
| Language | Rust | TypeScript | JavaScript |
| Config | Single TOML | Multiple files | Multiple files |
| Changelog Fragments | No | No | No |
| Conventional Commits | Yes | Yes | Yes |
| GitHub Actions | Manual/Release | Automatic PRs | Manual |
| Maintenance | Low | Medium | Medium |
| Customization | High | Medium | High |
| Speed | Very Fast | Fast | Medium |

**Why git-cliff?**
- ✅ Lightweight (single config file)
- ✅ Fast (Rust-native)
- ✅ Perfect for Rust projects
- ✅ Highly customizable
- ✅ No runtime dependencies
- ✅ Simple maintenance

## Resources

- **git-cliff Documentation**: https://git-cliff.org/
- **Conventional Commits**: https://www.conventionalcommits.org/
- **Keep a Changelog**: https://keepachangelog.com/
- **Semantic Versioning**: https://semver.org/

## Future Enhancements

Potential improvements for the future:

1. **Automatic PR labeling** based on commit types
2. **Changelog validation** in CI (ensure all PRs have conventional commits)
3. **Release notes template** with categorized changes
4. **GitHub release assets** with changelog excerpt
5. **Commit message linting** with commitlint or similar

---

**Last Updated**: 2026-02-12

For questions about the changelog system, see [CONTRIBUTING.md](../CONTRIBUTING.md) or open a GitHub discussion.
