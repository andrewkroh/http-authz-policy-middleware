#!/bin/bash
set -e

PLUGIN_DIR="plugins-local/src/github.com/andrewkroh/http-authz-policy-middleware"

echo "Creating plugin directory structure..."
mkdir -p "$PLUGIN_DIR"

echo "Copying WASM binary..."
cp ../plugin.wasm "$PLUGIN_DIR/plugin.wasm"

echo "Copying plugin manifest..."
cp ../.traefik.yml "$PLUGIN_DIR/.traefik.yml"

echo "Plugin setup complete."
ls -lah "$PLUGIN_DIR/"
