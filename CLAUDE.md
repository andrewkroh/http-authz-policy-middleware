# CLAUDE.md - Developer Guide

This document provides comprehensive development workflow instructions for contributors and AI assistants working on the HTTP Authorization Policy Middleware.

## Table of Contents

- [Project Overview](#project-overview)
- [Project Structure](#project-structure)
- [Prerequisites](#prerequisites)
- [Build Commands](#build-commands)
- [Testing](#testing)
- [Integration Testing](#integration-testing)
- [Code Quality](#code-quality)
- [Development Workflow](#development-workflow)
- [Performance Considerations](#performance-considerations)
- [Security Considerations](#security-considerations)
- [Troubleshooting](#troubleshooting)
- [Git History Rewrite](#git-history-rewrite)
- [Contributing](#contributing)
  - [Commit Message Guidelines](#commit-message-guidelines)
  - [Changelog Management](#changelog-management)
- [Release Process](#release-process)

## Project Overview

This is a Traefik v3 middleware plugin compiled to WebAssembly (WASM) that performs attribute-based authorization on HTTP requests using a custom expression language. The plugin:

- Implements the [http-wasm HTTP Handler ABI](https://http-wasm.io/http-handler-abi/)
- Compiles expressions at Traefik startup with type checking
- Evaluates expressions on each request against method, path, host, and headers
- Includes a built-in test framework validated at startup
- Follows a fail-closed security model

**Implementation Language:** Rust
**Target:** `wasm32-wasip1`
**Binary Size:** ~650 KB (release build with optimizations, see [Binary Size](#binary-size))

## Project Structure

```
.
├── src/
│   ├── lib.rs                  # WASM plugin entry point, configuration
│   ├── config.rs               # Configuration schema and deserialization
│   ├── context.rs              # Request context (method, path, host, headers)
│   └── expr/
│       ├── mod.rs              # Expression module exports
│       ├── ast.rs              # Abstract Syntax Tree definitions
│       ├── lexer.rs            # Lexical analysis (tokenization)
│       ├── parser.rs           # Recursive descent parser
│       ├── compiler.rs         # Type checking and compilation
│       └── eval.rs             # Runtime expression evaluation
├── tests/
│   └── integration_test.rs     # Rust integration tests
├── integration-test/           # Docker-based end-to-end tests
│   ├── setup-plugin.sh         # Plugin directory setup script
│   ├── docker-compose.yml      # Traefik container setup
│   ├── traefik.yml             # Traefik static configuration
│   ├── dynamic.yml             # Middleware definitions with test cases
│   ├── test.sh                 # Integration test script
│   └── README.md               # Integration testing documentation
├── examples/                   # Example Traefik configurations
├── scripts/
│   ├── check-license.sh        # License header validation
│   ├── generate-changelog.sh   # Changelog generation helper
│   └── prepare-release.sh      # Release preparation automation
├── docs/
│   ├── DESIGN.md               # Comprehensive design documentation
│   ├── TASKS.md                # Implementation progress tracking
│   └── CHANGELOG_SYSTEM.md     # Changelog system documentation
├── .github/
│   ├── workflows/
│   │   ├── ci.yml              # CI/CD pipeline
│   │   ├── changelog.yml       # Changelog generation workflow
│   │   └── release.yml         # Automated release workflow
│   └── CHANGELOG_GUIDE.md      # Quick changelog reference
├── Cargo.toml                  # Rust dependencies and build config
├── Makefile                    # Build automation
├── cliff.toml                  # git-cliff configuration for changelog generation
├── CHANGELOG.md                # Project changelog (auto-generated)
├── CONTRIBUTING.md             # Contributing guidelines
├── LICENSE                     # MIT license
├── .traefik.yml                # Traefik plugin manifest
└── README.md                   # User-facing documentation

```

### Module Responsibilities

**`src/lib.rs`**
- WASM plugin entry point implementing http-wasm guest ABI
- Configuration parsing and validation
- Plugin initialization and request handling

**`src/config.rs`**
- Configuration schema (expression, denyStatusCode, denyBody, tests)
- Serde deserialization from JSON

**`src/context.rs`**
- Request context abstraction
- Header access (case-insensitive lookup)
- Mock request context for testing

**`src/expr/lexer.rs`**
- Tokenizes expression strings into tokens
- Handles identifiers, operators, string literals, function calls

**`src/expr/parser.rs`**
- Recursive descent parser (operator precedence)
- Constructs AST from token stream

**`src/expr/compiler.rs`**
- Type checking (ensures boolean result)
- Validates function signatures and argument counts
- Catches errors before runtime

**`src/expr/eval.rs`**
- Runtime expression evaluation against request context
- Implements built-in functions (header, headerList, contains, etc.)
- Regex compilation and caching

**`src/expr/ast.rs`**
- AST node definitions
- Expression types (binary ops, function calls, literals, etc.)

## Prerequisites

### Required Tools

- **Rust toolchain** (stable): Install via [rustup](https://rustup.rs/)
- **wasm32-wasip1 target**:
  ```bash
  rustup target add wasm32-wasip1
  ```
- **Make**: For build automation (usually pre-installed on Linux/macOS)

### Optional Tools

- **Docker & Docker Compose**: For integration testing
- **cargo-watch**: For automatic rebuilds during development
  ```bash
  cargo install cargo-watch
  ```

## Build Commands

The project uses a `Makefile` for build automation.

### Debug Build

```bash
make build
```

This compiles the plugin in debug mode:
- Output: `target/wasm32-wasip1/debug/traefik_authz_wasm.wasm`
- Includes debug symbols and assertions
- Larger binary size (~2 MB)
- Faster compilation

### Release Build

```bash
make release
```

This compiles the plugin with optimizations:
- Output: `target/wasm32-wasip1/release/traefik_authz_wasm.wasm`
- Automatically optimizes with wasm-opt (if installed)
- Copies optimized binary to `plugin.wasm` in project root
- Size optimizations enabled (`opt-level = "z"`)
- Link-time optimization (LTO)
- Stripped symbols
- Final size: ~650 KB (with wasm-opt), ~780 KB (without)
- Required for integration testing

### Manual Cargo Commands

You can also use cargo directly:

```bash
# Debug build
cargo build --target wasm32-wasip1

# Release build
cargo build --target wasm32-wasip1 --release

# Watch mode (requires cargo-watch)
cargo watch -x 'build --target wasm32-wasip1'
```

### Build Optimization Settings

The `Cargo.toml` includes aggressive size optimizations for release builds:

```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Enable link-time optimization
codegen-units = 1   # Better optimization (slower compile)
strip = true        # Strip symbols
panic = "abort"     # Smaller binary (no unwinding)

[dependencies]
regex = { version = "1.10", default-features = false, features = ["std", "perf"] }
```

**Regex Optimization:**
The regex crate by default includes ~400 KB of Unicode lookup tables. Since this plugin only uses ASCII regex patterns (e.g., `^/api/v[0-9]+/.*`), we disable Unicode features to save significant space.

**Post-Build Optimization:**
The Makefile automatically runs `wasm-opt -Oz --enable-bulk-memory` if available, which provides an additional 17% size reduction through advanced WASM optimizations. Install with: `cargo install wasm-opt`

## Testing

### Unit and Integration Tests

Run the Rust test suite:

```bash
make test
```

Or directly with cargo:

```bash
cargo test
```

This runs:
- Unit tests embedded in source files (if any)
- Integration tests in `tests/integration_test.rs`
- Expression language tests (lexer, parser, compiler, evaluator)

### Test Coverage

Key test areas:
- Lexer: tokenization of all operators, identifiers, literals
- Parser: operator precedence, function calls, boolean logic
- Compiler: type checking, error detection
- Evaluator: expression evaluation, built-in functions
- Configuration: deserialization, validation

### Writing Tests

Example test structure:

```rust
#[test]
fn test_expression_evaluation() {
    let expr = parse_and_compile("method == \"GET\"").unwrap();
    let ctx = MockContext::new().with_method("GET");
    assert_eq!(eval(&expr, &ctx), Ok(true));
}
```

## Integration Testing

Integration tests validate the plugin in a real Traefik environment using Docker.

### Prerequisites

- Release build: `make release`
- Docker and Docker Compose installed

### Running Integration Tests (Hardened Mode - Recommended)

The recommended approach runs tests inside the Docker network for better security and isolation:

```bash
# From project root
make release

# Navigate to integration test directory
cd integration-test

# Run automated test script (builds container, runs tests, cleans up)
./run-tests.sh
```

This approach provides:
- **Network isolation** - Tests run inside Docker network
- **No port binding** - Services communicate via internal service names
- **Automatic cleanup** - Services shut down after tests complete
- **Health checks** - Proper service startup coordination

### Alternative: Legacy Mode (Tests From Host)

For debugging or interactive testing, you can run tests from the host:

```bash
# From project root
make release

# Navigate to integration test directory
cd integration-test

# Set up plugin directory structure
./setup-plugin.sh

# Start Traefik and backend (ports bound to 127.0.0.1 only)
docker compose up -d traefik backend

# Run integration tests from host
TRAEFIK_URL=http://localhost:8080 ./test.sh

# View results and clean up
docker compose down
```

Note: Ports are bound to 127.0.0.1 (not 0.0.0.0) for security.

### Integration Test Scenarios

The `test.sh` script validates:

1. **Authorized requests** - Correct headers return HTTP 200
2. **Unauthorized requests** - Wrong/missing headers return HTTP 403
3. **Custom deny body** - Configured deny messages are returned
4. **Method-based access** - GET allowed, POST denied (HTTP 405)
5. **Startup test validation** - Plugin test cases run at Traefik startup

### Test Configuration

- **`traefik.yml`** - Static config with `experimental.localPlugins`
- **`dynamic.yml`** - Middleware definitions with expressions and test cases
- **`docker-compose.yml`** - Services with health checks and test runner
- **`Dockerfile.test`** - Test runner container with curl and bash
- **`run-tests.sh`** - Automated test execution script (hardened mode)
- **`test.sh`** - Test script (supports both hardened and legacy modes via TRAEFIK_URL)

### Plugin Directory Structure

Traefik v3 requires local WASM plugins to follow this structure:

```
plugins-local/
└── src/
    └── github.com/
        └── andrewkroh/
            └── http-authz-policy-middleware/
                ├── plugin.wasm
                └── .traefik.yml
```

The `setup-plugin.sh` script creates this structure automatically.

### Viewing Logs

```bash
# Follow Traefik logs (while services are running)
docker compose logs -f traefik

# Check for plugin initialization messages
docker compose logs traefik | grep -i plugin

# Access Traefik dashboard (legacy mode only, when ports are exposed)
open http://127.0.0.1:8081
```

Note: Dashboard access requires running in legacy mode with port exposure.

## Code Quality

### Linting

Run Clippy (Rust linter) with WASM target:

```bash
make check
```

Or directly:

```bash
cargo clippy --target wasm32-wasip1
```

Address all warnings before submitting PRs.

### Formatting

Check code formatting:

```bash
cargo fmt --check
```

Auto-format code:

```bash
cargo fmt
```

The project follows standard Rust formatting conventions (rustfmt defaults).

### Combined Quality Check

The `make check` target runs both Clippy and format check:

```bash
make check
# Runs: cargo clippy --target wasm32-wasip1
#       cargo fmt --check
```

## Development Workflow

### Typical Development Cycle

1. **Make changes** to source files
2. **Build** in debug mode: `make build`
3. **Run tests**: `make test`
4. **Check code quality**: `make check`
5. **Test in Traefik** (if needed):
   - `make release`
   - `cd integration-test`
   - `./run-tests.sh` (hardened mode - recommended)
6. **Format code**: `cargo fmt`
7. **Commit changes**

For interactive debugging, use legacy mode:
```bash
cd integration-test
./setup-plugin.sh
docker compose up -d traefik backend
TRAEFIK_URL=http://localhost:8080 ./test.sh
docker compose logs traefik  # View detailed logs
docker compose down
```

### Adding New Features

When adding new expression language features:

1. **Update AST** (`ast.rs`) - Add new node types
2. **Update Lexer** (`lexer.rs`) - Add new tokens/keywords
3. **Update Parser** (`parser.rs`) - Parse new syntax
4. **Update Compiler** (`compiler.rs`) - Type check new constructs
5. **Update Evaluator** (`eval.rs`) - Implement runtime behavior
6. **Add tests** - Unit tests for each component
7. **Update README** - Document new syntax/functions
8. **Add examples** - Create example configurations
9. **Integration test** - Validate in real Traefik environment

### Adding New Built-in Functions

Example: Adding a new function `toLowerCase(str)`:

1. **Parser** - Add to function call parsing (already generic)
2. **Compiler** - Add validation for function signature
3. **Evaluator** - Implement function logic in `eval_function_call`
4. **Tests** - Add test cases
5. **Documentation** - Update README expression reference

## Performance Considerations

### Compile-Time Optimizations

- Expressions are compiled once at Traefik startup
- Type checking prevents runtime type errors
- AST is traversed efficiently (no interpretation)

### Runtime Optimizations

- Boolean operators use short-circuit evaluation
- Header lookups are optimized (case-insensitive hash map)
- Regex patterns compiled on-demand (consider caching for hot paths)

### Binary Size

The release build is optimized for size:
- Current size: ~650 KB (optimized with wasm-opt)
- Unoptimized size: ~780 KB
- Size breakdown:
  - Regex engine: ~300 KB (with Unicode features disabled)
  - Serde/JSON: ~200 KB
  - Expression engine: ~100 KB
  - Other dependencies: ~50 KB
- **Optimizations applied:**
  - Cargo.toml: `opt-level = "z"`, LTO, strip symbols
  - Regex: Disabled Unicode features (using only ASCII patterns)
  - Post-processing: wasm-opt with `-Oz --enable-bulk-memory`
- **Further optimization potential:**
  - Replace regex with custom pattern matcher: Could save ~300 KB
  - Minimize serde_json features: Could save ~50-100 KB
  - Target size with full optimization: ~200-300 KB

### Benchmarking

To add benchmarks (consider using criterion.rs):

```toml
[dev-dependencies]
criterion = "0.5"
```

## Security Considerations

### Fail-Closed Model

- Any expression evaluation error returns HTTP 500 (deny access)
- Never fail open on unexpected errors
- Type errors caught at compile time

### Type Safety

- Expressions are type-checked before runtime
- Invalid expressions prevent Traefik startup
- No runtime type coercion

### Regex Safety

- Uses RE2-compatible regex engine (linear time matching)
- Prevents ReDoS (Regular Expression Denial of Service) attacks
- Invalid regex patterns caught at evaluation time (returns error)

### No Code Injection

- Expressions parsed into AST (not executed as code)
- No eval() or arbitrary code execution
- Sandboxed WASM environment

### Header Security

- Case-insensitive header lookup (HTTP/1.1 and HTTP/2 compatible)
- All header values treated as strings
- No buffer overflows (Rust memory safety)

## Troubleshooting

### Build Issues

**Error: `wasm32-wasip1` target not found**
```bash
rustup target add wasm32-wasip1
```

**Error: Linker errors or missing dependencies**
```bash
cargo clean
cargo build --target wasm32-wasip1
```

### Test Failures

**Expression compilation errors**
- Check syntax in test expressions
- Verify operator precedence
- Ensure functions have correct argument counts

**Integration test failures**
- Ensure `make release` ran successfully
- Check `plugin.wasm` exists in project root
- Verify Docker is running
- Check Traefik logs: `docker compose logs traefik`

### Traefik Plugin Loading Issues

**Plugin not found**
- Verify directory structure in `integration-test/plugins-local/`
- Run `./setup-plugin.sh` to rebuild plugin directory
- Check `traefik.yml` has correct `experimental.localPlugins` config

**Plugin compilation errors in Traefik logs**
- Expression syntax errors in `dynamic.yml`
- Type checking failures (non-boolean top-level expression)
- Invalid function names or argument counts

**Test failures at startup**
- Check test case `expect` values match expression results
- Verify mock request headers in test cases
- Review Traefik logs for detailed error messages

### Runtime Errors

**HTTP 500 responses**
- Usually caused by regex compilation errors in `matches()` operator
- Check Traefik logs for evaluation error details
- Validate regex patterns offline before deployment

**Unexpected authorization results**
- Enable debug logging in Traefik
- Add more test cases to narrow down issue
- Test expression in isolation

## Git History Rewrite

This repository's Git history has been rewritten to use a personal account instead of the original corporate/work email. All commits now use:

- **Name:** Andrew Kroh
- **Email:** id-github@andrewkroh.com

### What Was Changed

The history rewrite updated both author and committer fields for ALL commits in the repository while preserving:
- Commit messages
- Commit timestamps (author date and committer date)
- File changes and diffs
- Branch structure
- All other metadata

### How It Was Done

A script was created at `/workspace/scripts/rewrite-history.sh` to perform the rewrite safely:

1. **Backup Created** - A backup branch (`backup-before-rewrite`) was created before any changes
2. **History Rewritten** - Used `git filter-branch` to update all author/committer information
3. **Verification** - Validated that all commits now use the new identity
4. **Force Push** - Updated the remote repository (all commit SHAs changed)

### Using the Rewrite Script

The script supports two modes:

**Preview Mode (Default):**
```bash
./scripts/rewrite-history.sh preview
# OR simply
./scripts/rewrite-history.sh
```

This shows:
- Current authors and committers
- What will change
- Total number of commits to rewrite
- No actual changes are made

**Rewrite Mode:**
```bash
./scripts/rewrite-history.sh rewrite
```

This:
- Creates a backup branch
- Rewrites all history
- Verifies the changes
- Provides next steps for force pushing

### Important Notes for Collaborators

If you have an existing clone of this repository:

1. **DO NOT** try to pull or merge - your history is incompatible
2. **Backup** any local work (stash or create branches)
3. **Re-clone** the repository from scratch
4. **Reapply** any local work to the new clone

Any open pull requests will need to be recreated since the base branch history has changed.

### Why Rewrite History?

History rewrites are typically done to:
- Update author identity (work email → personal email)
- Remove sensitive data (passwords, keys)
- Clean up commit messages
- Consolidate authorship

In this case, the rewrite ensures all contributions are attributed to the personal GitHub account.

## Contributing

### Before Submitting PRs

1. **Run all tests**: `cargo test`
2. **Format code**: `cargo fmt`
3. **Lint**: `cargo clippy --target wasm32-wasip1`
4. **Integration tests**: Full integration test cycle
5. **Update documentation**: README, examples, DESIGN.md if needed
6. **Check CI**: Ensure GitHub Actions pass

### Commit Message Guidelines

Follow conventional commit format:

```
feat: add toLowerCase() built-in function
fix: correct case-insensitive header lookup
docs: update expression language reference
test: add test cases for regex matching
refactor: simplify parser error handling
```

**Conventional Commit Types:**
- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `test:` - Test additions or modifications
- `refactor:` - Code refactoring without feature changes
- `perf:` - Performance improvements
- `style:` - Code style changes (formatting, etc.)
- `chore:` - Build system, dependencies, or other maintenance tasks
- `ci:` - CI/CD configuration changes

These commit messages are used to automatically generate changelogs using git-cliff.

### Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make changes and test thoroughly
4. Commit with descriptive messages
5. Push to your fork: `git push origin feature/my-feature`
6. Open a pull request with description of changes
7. Address review feedback
8. Ensure CI passes

### Code Review Focus Areas

- Correctness of expression evaluation
- Type safety and error handling
- Security implications
- Performance impact
- Test coverage
- Documentation completeness

### Changelog Management

This project uses [git-cliff](https://git-cliff.org/) for automatic changelog generation based on conventional commits.

**View Current Changelog:**
```bash
# View the current CHANGELOG.md
cat CHANGELOG.md
```

**Generate Changelog for Unreleased Changes:**
```bash
# Install git-cliff (if not already installed)
cargo install git-cliff

# Generate changelog for unreleased commits
git cliff --unreleased --prepend CHANGELOG.md

# Preview without writing
git cliff --unreleased
```

**Generate Changelog for New Release:**
```bash
# Generate changelog for a specific tag
git cliff --tag v0.2.0 --prepend CHANGELOG.md

# Generate changelog for all releases
git cliff --prepend CHANGELOG.md
```

**Configuration:**
- Changelog configuration is in `cliff.toml`
- Follows conventional commit format
- Automatically categorizes commits by type (feat, fix, docs, etc.)
- Skips non-conventional commits and dependency updates

**Note:** The changelog is automatically updated during the release process by GitHub Actions. Manual updates are only needed for local testing or preview purposes.

## Release Process

This project uses automated GitHub Actions workflows to create releases. Releases are triggered by pushing semantic version tags.

### Creating a Release

#### 1. Ensure Version Consistency

Before creating a release, verify that the version in `Cargo.toml` matches the intended release tag:

```toml
[package]
name = "traefik-authz-wasm"
version = "0.1.0"  # Should match the tag (v0.1.0)
```

While not strictly required, keeping the Cargo.toml version synchronized with git tags is a best practice.

#### 2. Verify Pre-Release Checklist

Before tagging a release, ensure:

- [ ] All tests pass: `cargo test`
- [ ] Integration tests pass: `cd integration-test && ./run-tests.sh`
- [ ] Code is formatted: `cargo fmt --check`
- [ ] No clippy warnings: `cargo clippy --target wasm32-wasip1`
- [ ] Documentation is up to date
- [ ] CHANGELOG.md reflects recent changes (preview with `git cliff --unreleased`)
- [ ] All recent commits follow conventional commit format

#### 3. Create and Push a Version Tag

**Option A: Using the Helper Script (Recommended)**

A helper script automates pre-release checks and guides you through the process:

```bash
# Run the prepare-release script
./scripts/prepare-release.sh v0.2.0
```

This script will:
- Validate the version format
- Update Cargo.toml version (if needed)
- Run all pre-release checks (tests, formatting, clippy)
- Build and verify the release
- Preview the changelog
- Provide instructions for tagging

**Option B: Manual Process**

Tags must follow semantic versioning with a `v` prefix:

```bash
# For a new minor version
git tag v0.2.0

# For a patch version
git tag v0.1.1

# For a major version
git tag v1.0.0

# Push the tag to trigger the release workflow
git push origin v0.2.0
```

**Tag Format Requirements:**
- Must start with `v`
- Must follow semantic versioning: `vMAJOR.MINOR.PATCH`
- Examples: `v0.1.0`, `v1.2.3`, `v2.0.0`
- Invalid: `0.1.0`, `v1.2`, `release-1.0`

#### 4. Automated Release Process

Once the tag is pushed, GitHub Actions automatically:

1. **Validates the tag format** - Ensures it follows `vX.Y.Z` pattern
2. **Builds the WASM plugin** - Compiles with release optimizations
3. **Generates changelog** - Creates release notes using git-cliff
4. **Packages the plugin** - Creates a ZIP archive with:
   - `plugin.wasm` - The compiled WASM binary
   - `.traefik.yml` - The plugin manifest
5. **Creates GitHub Release** - Publishes release with:
   - Automatic release notes from commits
   - Changelog entry for this version
   - Plugin package ZIP
   - Individual plugin files (plugin.wasm, .traefik.yml)
   - Full CHANGELOG.md
6. **Updates CHANGELOG.md** - Commits updated changelog back to main branch

#### 5. Post-Release Steps

After the release workflow completes:

1. **Verify the release** on the GitHub Releases page
2. **Download and test** the plugin package
3. **Update version** in `Cargo.toml` for next development cycle (optional)
4. **Announce the release** (if applicable)
5. **Submit to Traefik Plugin Catalog** (if desired)

### Release Artifacts

Each release includes these artifacts:

| Artifact | Description | Use Case |
|----------|-------------|----------|
| `http-authz-policy-middleware-vX.Y.Z.zip` | Complete plugin package | Submit to Traefik Plugin Catalog |
| `plugin.wasm` | Compiled WASM binary | Direct use in Traefik |
| `.traefik.yml` | Plugin manifest | Required metadata |
| `CHANGELOG.md` | Full project changelog | View all changes |

### Traefik Plugin Catalog Submission

To submit your plugin to the Traefik Plugin Catalog:

1. **Download the release ZIP** from GitHub Releases
2. **Extract and verify** contents (plugin.wasm + .traefik.yml)
3. **Tag your repository** with `traefik-plugin` topic on GitHub
4. **Follow Traefik's plugin submission process**

The ZIP archive format matches Traefik's requirements:
- Contains `plugin.wasm` (default WASM file name)
- Contains `.traefik.yml` (plugin manifest)
- Ready for catalog submission

### Versioning Strategy

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR version** (v2.0.0) - Incompatible API changes or breaking changes
  - Example: Changing expression syntax that breaks existing configurations
- **MINOR version** (v0.2.0) - New features in a backward-compatible manner
  - Example: Adding new built-in functions or operators
- **PATCH version** (v0.1.1) - Backward-compatible bug fixes
  - Example: Fixing a bug in expression evaluation

### Release Troubleshooting

**Tag push rejected:**
```bash
# Tag already exists locally - delete and recreate
git tag -d v0.1.0
git tag v0.1.0
git push origin v0.1.0
```

**Release workflow fails:**
- Check GitHub Actions logs for specific errors
- Verify tag format matches `vX.Y.Z` pattern
- Ensure all CI checks pass on main branch
- Check that `wasm32-wasip1` target builds successfully

**Changelog not updating:**
- Ensure commits follow conventional commit format
- Check `cliff.toml` configuration
- Verify git history is available (fetch-depth: 0)

**Release artifacts missing:**
- Verify `plugin.wasm` is created during build step
- Check ZIP archive creation in workflow logs
- Ensure `.traefik.yml` exists in repository root

### Resources

- **http-wasm ABI**: https://http-wasm.io/http-handler-abi/
- **Traefik Plugins**: https://doc.traefik.io/traefik/plugins/overview/
- **Rust WASM Book**: https://rustwasm.github.io/docs/book/
- **Design Document**: [docs/DESIGN.md](docs/DESIGN.md)
- **git-cliff Documentation**: https://git-cliff.org/

---

**Last Updated**: 2026-02-12

For questions or issues, please open a GitHub issue or discussion.
