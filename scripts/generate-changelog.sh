#!/usr/bin/env bash
#
# Helper script to generate changelog using git-cliff
#
# Usage:
#   ./scripts/generate-changelog.sh          # Preview unreleased changes
#   ./scripts/generate-changelog.sh --write  # Update CHANGELOG.md with unreleased changes
#   ./scripts/generate-changelog.sh v0.2.0   # Generate for specific version

set -e

# Check if git-cliff is installed
if ! command -v git-cliff &> /dev/null; then
    echo "Error: git-cliff is not installed"
    echo "Install with: cargo install git-cliff"
    exit 1
fi

# Parse arguments
WRITE=false
TAG=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --write|-w)
            WRITE=true
            shift
            ;;
        v*.*.*)
            TAG="$1"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--write] [vX.Y.Z]"
            exit 1
            ;;
    esac
done

# Generate changelog
if [ -n "$TAG" ]; then
    echo "Generating changelog for tag: $TAG"
    if [ "$WRITE" = true ]; then
        git cliff --tag "$TAG" --prepend CHANGELOG.md
        echo "✓ CHANGELOG.md updated for $TAG"
    else
        git cliff --tag "$TAG"
    fi
else
    echo "Generating changelog for unreleased changes"
    if [ "$WRITE" = true ]; then
        git cliff --unreleased --prepend CHANGELOG.md
        echo "✓ CHANGELOG.md updated with unreleased changes"
    else
        git cliff --unreleased
    fi
fi

# Show diff if written
if [ "$WRITE" = true ]; then
    echo ""
    echo "=== Recent changes to CHANGELOG.md ==="
    git diff CHANGELOG.md | head -n 50
fi
