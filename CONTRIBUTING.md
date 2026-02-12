# Contributing to HTTP Authorization Policy Middleware

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Commit Message Guidelines](#commit-message-guidelines)
- [Changelog System](#changelog-system)
- [Pull Request Process](#pull-request-process)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)

## Code of Conduct

This project follows a professional and respectful code of conduct. Please be kind and constructive in all interactions.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/http-authz-policy-middleware.git
   cd http-authz-policy-middleware
   ```
3. **Install prerequisites**:
   - Rust toolchain (stable)
   - wasm32-wasip1 target: `rustup target add wasm32-wasip1`
   - Docker and Docker Compose (for integration tests)

4. **Read the developer guide**: See [CLAUDE.md](CLAUDE.md) for detailed development instructions

## Development Workflow

### Before You Start

1. Create a feature branch:
   ```bash
   git checkout -b feature/my-feature
   ```

2. Make your changes following the project structure and conventions

### During Development

1. **Run tests frequently**:
   ```bash
   cargo test
   ```

2. **Format your code**:
   ```bash
   cargo fmt
   ```

3. **Run linter**:
   ```bash
   cargo clippy --target wasm32-wasip1
   ```

4. **Test the integration** (if applicable):
   ```bash
   make release
   cd integration-test
   ./run-tests.sh
   ```

## Commit Message Guidelines

This project uses **Conventional Commits** for automatic changelog generation. All commit messages must follow this format:

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

### Commit Types

- **feat**: A new feature (generates "Features" section in changelog)
- **fix**: A bug fix (generates "Bug Fixes" section in changelog)
- **docs**: Documentation only changes (generates "Documentation" section)
- **test**: Adding or updating tests (generates "Testing" section)
- **refactor**: Code change that neither fixes a bug nor adds a feature (generates "Refactoring" section)
- **perf**: Performance improvements (generates "Performance" section)
- **style**: Code style changes (formatting, missing semicolons, etc.)
- **chore**: Changes to build process, dependencies, or auxiliary tools
- **ci**: Changes to CI/CD configuration files

### Examples

```bash
# Good commit messages
git commit -m "feat: add support for regex negation in expressions"
git commit -m "fix: correct case-sensitive header comparison bug"
git commit -m "docs: update expression language reference"
git commit -m "test: add integration tests for allOf function"
git commit -m "refactor: simplify parser error handling"
git commit -m "perf: optimize regex compilation with caching"

# Bad commit messages (will be filtered out of changelog)
git commit -m "update stuff"
git commit -m "WIP"
git commit -m "fix things"
```

### Scope (Optional)

You can add a scope to provide more context:

```bash
git commit -m "feat(parser): add support for OR operator"
git commit -m "fix(eval): handle empty header values correctly"
git commit -m "test(lexer): add tokenization tests for new operators"
```

### Breaking Changes

For breaking changes, add `BREAKING CHANGE:` in the commit body or use `!` after type/scope:

```bash
git commit -m "feat!: change expression syntax for function calls"
# or
git commit -m "feat: change expression syntax

BREAKING CHANGE: Function calls now require parentheses"
```

## Changelog System

This project uses [git-cliff](https://git-cliff.org/) to automatically generate changelogs from commit messages.

### How It Works

1. **Automatic**: Changelogs are generated automatically during releases
2. **Conventional Commits**: Only commits following the conventional format are included
3. **Categorization**: Commits are automatically grouped by type (features, fixes, etc.)
4. **Keep a Changelog**: Output follows the [Keep a Changelog](https://keepachangelog.com/) format

### Configuration

The changelog generation is configured in `cliff.toml`. Key settings:

- Conventional commit parsing enabled
- Non-conventional commits filtered out
- Automatic grouping by commit type
- Dependency updates and PR preparation commits skipped

### Manual Changelog Generation (Optional)

For testing or preview purposes:

```bash
# Install git-cliff
cargo install git-cliff

# Preview unreleased changes
git cliff --unreleased

# Generate changelog entry for unreleased changes
git cliff --unreleased --prepend CHANGELOG.md

# Generate changelog for a specific version
git cliff --tag v0.2.0 --prepend CHANGELOG.md
```

### What NOT to Do

- **Do NOT manually edit CHANGELOG.md** unless fixing a mistake
- **Do NOT create changelog fragments** (git-cliff generates from commits)
- **Do NOT skip conventional commit format** (your changes won't appear in the changelog)

## Pull Request Process

### Before Submitting

1. **Ensure all tests pass**:
   ```bash
   cargo test
   make release
   cd integration-test && ./run-tests.sh
   ```

2. **Format and lint**:
   ```bash
   cargo fmt
   cargo clippy --target wasm32-wasip1
   ```

3. **Update documentation** if needed:
   - Update README.md for user-facing changes
   - Update CLAUDE.md for developer workflow changes
   - Add examples if introducing new features

4. **Verify commit messages** follow conventional format

### Submitting the PR

1. **Push to your fork**:
   ```bash
   git push origin feature/my-feature
   ```

2. **Open a Pull Request** on GitHub with:
   - **Clear title**: Summarize the changes
   - **Description**: Explain what, why, and how
   - **Related issues**: Link any related issues
   - **Testing**: Describe how you tested the changes
   - **Breaking changes**: Highlight any breaking changes

3. **PR Template** (use this structure):
   ```markdown
   ## Summary
   Brief description of changes

   ## Motivation
   Why is this change needed?

   ## Changes
   - List of specific changes

   ## Testing
   How were these changes tested?

   ## Checklist
   - [ ] Tests pass locally
   - [ ] Code is formatted (cargo fmt)
   - [ ] No clippy warnings
   - [ ] Integration tests pass
   - [ ] Documentation updated
   - [ ] Commit messages follow conventional format
   ```

### During Review

1. **Respond to feedback** constructively
2. **Make requested changes** in new commits (do NOT force-push during review)
3. **Update tests** if logic changes
4. **Keep the PR focused** - one feature or fix per PR

### After Approval

1. **Squash commits** if requested (maintainers may do this)
2. **Wait for CI** to pass
3. **Maintainers will merge** when ready

## Testing Guidelines

### Unit Tests

- Add tests for new functions and logic
- Test edge cases and error conditions
- Keep tests focused and isolated
- Use descriptive test names

Example:
```rust
#[test]
fn test_header_case_insensitive_lookup() {
    let mut ctx = RequestContext::new();
    ctx.add_header("Content-Type", "application/json");
    assert_eq!(ctx.header("content-type"), "application/json");
    assert_eq!(ctx.header("CONTENT-TYPE"), "application/json");
}
```

### Integration Tests

- Add integration tests in `tests/integration_test.rs` for end-to-end scenarios
- Add Docker-based tests in `integration-test/` for Traefik validation
- Ensure tests are reproducible and independent

### Test Coverage

Aim for high coverage:
- All new features should have tests
- All bug fixes should have regression tests
- Critical security logic should have comprehensive tests

## Documentation

### Code Documentation

- Add doc comments to public functions, structs, and modules
- Explain complex logic with inline comments
- Use examples in doc comments where helpful

```rust
/// Evaluates an expression against a request context.
///
/// # Arguments
/// * `expr` - The compiled expression to evaluate
/// * `ctx` - The request context containing headers, method, path, etc.
///
/// # Returns
/// * `Ok(true)` - Request is authorized
/// * `Ok(false)` - Request is denied
/// * `Err(_)` - Evaluation error (treated as denial)
///
/// # Example
/// ```
/// let expr = compile("method == \"GET\"").unwrap();
/// let ctx = RequestContext::from_test(&test_request);
/// assert_eq!(eval(&expr, &ctx), Ok(true));
/// ```
pub fn eval(expr: &Expr, ctx: &RequestContext) -> Result<bool, EvalError> {
    // ...
}
```

### User Documentation

Update these files for user-facing changes:

- **README.md**: User-facing features, configuration, examples
- **examples/**: Add example configurations for new features
- **integration-test/README.md**: Integration testing instructions

### Developer Documentation

Update these files for developer workflow changes:

- **CLAUDE.md**: Development workflow, build instructions, troubleshooting
- **docs/DESIGN.md**: Architecture, design decisions, implementation details

## Releases

### For Maintainers

Releases are automated via GitHub Actions when version tags are pushed. See [CLAUDE.md - Release Process](CLAUDE.md#release-process) for complete instructions.

**Quick Release Steps:**
1. Ensure all tests pass and commits follow conventional format
2. Update version in `Cargo.toml` (e.g., `version = "0.2.0"`)
3. Create and push a version tag:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```
4. GitHub Actions will automatically:
   - Build the WASM plugin
   - Generate changelog
   - Create GitHub Release with plugin package
   - Update CHANGELOG.md

**Tag Format:** Must follow semantic versioning with `v` prefix (e.g., `v0.1.0`, `v1.2.3`)

### For Contributors

Contributors do not create releases. Maintainers will:
- Review and merge your PRs
- Include changes in the next release
- Credit you in the automated changelog (based on git commits)

Your commit messages following conventional commit format ensure your contributions are properly documented in release notes.

## Questions?

- Open a [GitHub Discussion](../../discussions) for questions
- Open a [GitHub Issue](../../issues) for bugs or feature requests
- Check [CLAUDE.md](CLAUDE.md) for detailed development instructions

Thank you for contributing!
