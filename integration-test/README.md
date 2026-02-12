# Docker-Based Traefik Integration Tests

This directory contains end-to-end integration tests that validate the plugin in a real Traefik environment.

## Prerequisites

- Docker and Docker Compose installed
- Plugin built: `make release` (from project root)

## Running the Tests

1. **Build the release binary** (from project root):
   ```bash
   make release
   ```

2. **Start Traefik with the plugin**:
   ```bash
   cd integration-test
   docker compose up -d
   ```

3. **Run the integration tests**:
   ```bash
   ./test.sh
   ```

4. **View Traefik logs** (to see plugin initialization):
   ```bash
   docker compose logs traefik
   ```

5. **Clean up**:
   ```bash
   docker compose down
   ```

## What Gets Tested

The integration tests validate:

1. **Authorized requests** - Requests with correct headers are allowed (HTTP 200)
2. **Unauthorized requests** - Requests with wrong/missing headers are denied (HTTP 403)
3. **Deny body** - Custom deny messages are returned correctly
4. **Method-based access** - GET allowed, POST denied (HTTP 405)
5. **Startup test validation** - Plugin tests run at Traefik startup (check logs)

## Test Scenarios

### Team-Based Authorization
- Expression: `contains(headerList("X-Auth-User-Teams"), "platform-eng")`
- Tests header-based access control with comma-separated values

### Method-Based Authorization
- Expression: `method == "GET" OR method == "HEAD"`
- Tests request method restrictions

## Troubleshooting

### Traefik won't start
- Check if plugin.wasm exists in project root: `ls -lh ../plugin.wasm`
- View logs: `docker compose logs traefik`
- Look for compilation or test errors in Traefik output

### Tests fail
- Ensure Traefik is fully started before running tests
- Check Traefik logs for plugin errors: `docker compose logs traefik`
- Verify plugin loaded successfully (look for test results in logs)

### Port conflicts
- Default ports: 8080 (HTTP), 8081 (Dashboard)
- Modify ports in docker-compose.yml if needed

## Traefik Dashboard

Access Traefik dashboard at: http://localhost:8081

View middlewares, routers, and services configuration.

## GitHub Actions Integration

The integration tests can be run in CI/CD:

```yaml
- name: Build release WASM
  run: make release

- name: Run integration tests
  run: |
    cd integration-test
    docker compose up -d
    sleep 10  # Wait for Traefik startup
    ./test.sh
    docker compose logs traefik
    docker compose down
```

## Notes

- **Local plugin loading**: The plugin is mounted from `../plugin.wasm`, not loaded from the Traefik plugin catalog
- **WASM experimental feature**: Requires Traefik v3.0+ with `experimental.wasm.enabled: true`
- **Startup tests**: Test cases in dynamic.yml are validated when Traefik starts - if any fail, Traefik will abort
