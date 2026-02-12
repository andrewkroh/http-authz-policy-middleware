#!/bin/bash
# Helper script to prepare for a new release
# Usage: ./scripts/prepare-release.sh v0.2.0

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v0.2.0"
    exit 1
fi

VERSION="$1"

# Validate version format
if ! [[ "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must follow format vX.Y.Z (e.g., v0.2.0)"
    exit 1
fi

# Extract version without 'v' prefix
VERSION_NUMBER="${VERSION#v}"

echo "Preparing release $VERSION..."
echo

# Check if working directory is clean
if ! git diff-index --quiet HEAD --; then
    echo "Error: Working directory has uncommitted changes"
    echo "Please commit or stash changes before preparing a release"
    exit 1
fi

# Check if on main branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo "Warning: Not on main branch (current: $CURRENT_BRANCH)"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo "Step 1: Update Cargo.toml version..."
if grep -q "^version = \"$VERSION_NUMBER\"" Cargo.toml; then
    echo "  ✓ Cargo.toml already has version $VERSION_NUMBER"
else
    # Update version in Cargo.toml
    if command -v sed &> /dev/null; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS
            sed -i '' "s/^version = \".*\"/version = \"$VERSION_NUMBER\"/" Cargo.toml
        else
            # Linux
            sed -i "s/^version = \".*\"/version = \"$VERSION_NUMBER\"/" Cargo.toml
        fi
        echo "  ✓ Updated Cargo.toml to version $VERSION_NUMBER"
    else
        echo "  ! Could not update Cargo.toml automatically (sed not found)"
        echo "  Please manually update version to $VERSION_NUMBER in Cargo.toml"
    fi
fi

echo
echo "Step 2: Preview changelog for this release..."
if command -v git-cliff &> /dev/null; then
    git cliff --unreleased --tag "$VERSION" | head -50
    echo
else
    echo "  ! git-cliff not installed, skipping changelog preview"
    echo "  Install with: cargo install git-cliff"
fi

echo
echo "Step 3: Run pre-release checks..."

# Run tests
echo "  Running tests..."
if cargo test --quiet; then
    echo "  ✓ Tests passed"
else
    echo "  ✗ Tests failed"
    exit 1
fi

# Check formatting
echo "  Checking code formatting..."
if cargo fmt --check; then
    echo "  ✓ Code is formatted"
else
    echo "  ✗ Code is not formatted. Run: cargo fmt"
    exit 1
fi

# Check clippy
echo "  Running clippy..."
if cargo clippy --target wasm32-wasip1 --quiet 2>&1 | grep -q "warning:"; then
    echo "  ✗ Clippy warnings found"
    cargo clippy --target wasm32-wasip1
    exit 1
else
    echo "  ✓ No clippy warnings"
fi

# Build release
echo "  Building release..."
if cargo build --target wasm32-wasip1 --release --quiet; then
    WASM_SIZE=$(stat -c%s target/wasm32-wasip1/release/traefik_authz_wasm.wasm 2>/dev/null || stat -f%z target/wasm32-wasip1/release/traefik_authz_wasm.wasm)
    WASM_SIZE_KB=$((WASM_SIZE / 1024))
    echo "  ✓ Release build successful (${WASM_SIZE_KB} KB)"
else
    echo "  ✗ Release build failed"
    exit 1
fi

echo
echo "========================================"
echo "Release preparation complete!"
echo "========================================"
echo
echo "Version: $VERSION"
echo "Cargo.toml: $VERSION_NUMBER"
echo "WASM size: ${WASM_SIZE_KB} KB"
echo
echo "Next steps:"
echo "  1. Review changes in Cargo.toml"
echo "  2. Commit version bump (if changed):"
echo "     git add Cargo.toml"
echo "     git commit -m \"chore: bump version to $VERSION_NUMBER\""
echo "     git push origin main"
echo "  3. Create and push the release tag:"
echo "     git tag $VERSION"
echo "     git push origin $VERSION"
echo "  4. GitHub Actions will automatically create the release"
echo
