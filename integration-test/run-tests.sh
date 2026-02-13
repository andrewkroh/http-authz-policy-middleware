#!/bin/bash
set -e

echo "=== Running Integration Tests ==="
echo ""

# Ensure plugin is set up
if [ ! -f "plugins-local/src/github.com/andrewkroh/http-authz-policy-middleware/plugin.wasm" ]; then
    echo "Plugin not found. Running setup-plugin.sh..."
    ./setup-plugin.sh
fi

# Build test container image
echo "Building test container..."
docker compose build test-runner

# Start infrastructure services
echo "Starting Traefik and backend services..."
docker compose up -d traefik backend

# Wait for services to be healthy
echo "Waiting for services to be ready..."
sleep 2

# Run tests inside Docker network
echo ""
echo "Running tests inside Docker network..."
docker compose run --rm test-runner

# Show result
TEST_EXIT_CODE=$?
echo ""
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo "✅ All tests passed!"
else
    echo "❌ Tests failed!"
    echo ""
    echo "To view Traefik logs:"
    echo "  docker compose logs traefik"
fi

# Cleanup
echo ""
echo "Cleaning up..."
docker compose down

exit $TEST_EXIT_CODE
