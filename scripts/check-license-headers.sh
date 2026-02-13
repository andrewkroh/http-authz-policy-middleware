#!/usr/bin/env bash
# Check that all Rust source files have MIT license headers

set -euo pipefail

# Expected license header pattern (first 2 lines)
EXPECTED_LINE1="// Copyright (c) 2025 Andrew Kroh"
EXPECTED_LINE2="// SPDX-License-Identifier: MIT"

# Find all .rs files in src/
RUST_FILES=$(find src -name "*.rs" -type f)

MISSING_HEADER=0

for file in $RUST_FILES; do
    # Read first two lines
    LINE1=$(head -n 1 "$file")
    LINE2=$(sed -n '2p' "$file")

    # Check if they match expected header
    if [[ "$LINE1" != "$EXPECTED_LINE1" ]] || [[ "$LINE2" != "$EXPECTED_LINE2" ]]; then
        echo "ERROR: Missing or incorrect MIT license header in: $file"
        echo "  Expected:"
        echo "    $EXPECTED_LINE1"
        echo "    $EXPECTED_LINE2"
        echo "  Found:"
        echo "    $LINE1"
        echo "    $LINE2"
        echo ""
        MISSING_HEADER=1
    fi
done

if [[ $MISSING_HEADER -eq 0 ]]; then
    echo "✓ All Rust source files have correct MIT license headers"
    exit 0
else
    echo ""
    echo "ERROR: Some files are missing MIT license headers"
    exit 1
fi
