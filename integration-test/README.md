# Docker-Based Traefik Integration Tests

This directory contains end-to-end integration tests that validate the plugin in a real Traefik environment.

## Prerequisites

- Docker and Docker Compose installed
- Plugin built: `make release` (from project root)

## Running the Tests

### Recommended: Hardened Mode (Tests Run Inside Docker Network)

This is the secure way to run tests - tests execute inside the Docker network without exposing ports to the host:

1. **Build the release binary** (from project root):
   ```bash
   make release
   ```

2. **Run the automated test script**:
   ```bash
   cd integration-test
   ./run-tests.sh
   ```

This script will:
- Set up the plugin directory structure
- Build the test runner container
- Start Traefik and backend services
- Run tests inside the Docker network (using service names)
- Display results and clean up

### Alternative: Legacy Mode (Tests Run From Host)

For debugging or if you need to access Traefik from your host:

1. **Build the release binary** (from project root):
   ```bash
   make release
   ```

2. **Set up the plugin directory**:
   ```bash
   cd integration-test
   ./setup-plugin.sh
   ```

3. **Start services** (ports bound to 127.0.0.1 only):
   ```bash
   docker compose up -d traefik backend
   ```

4. **Run tests from host**:
   ```bash
   TRAEFIK_URL=http://localhost:8080 ./test.sh
   ```

5. **View Traefik logs** (to see plugin initialization):
   ```bash
   docker compose logs traefik
   ```

6. **Clean up**:
   ```bash
   docker compose down
   ```

Note: In legacy mode, ports are bound to 127.0.0.1:8080 and 127.0.0.1:8081 (not 0.0.0.0) for security.

## Security Improvements

The integration test suite has been hardened with the following security measures:

1. **Network Isolation** - Tests run inside Docker network, not from host
2. **Restricted Port Exposure** - Ports bound to 127.0.0.1 only (not 0.0.0.0)
3. **No Host Port Dependencies** - Tests use internal service names (traefik:80)
4. **Health Checks** - Proper service dependency management with health checks

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

## How It Works

### Plugin Directory Structure

Traefik v3 local WASM plugins must follow a specific directory layout:

```
plugins-local/
└── src/
    └── github.com/
        └── andrewkroh/
            └── http-authz-policy-middleware/
                ├── plugin.wasm
                └── .traefik.yml
```

The `setup-plugin.sh` script creates this structure and copies the built
`plugin.wasm` and `.traefik.yml` manifest into the correct location.

### Configuration

- **traefik.yml** - Static config declaring the local plugin via `experimental.localPlugins`
- **dynamic.yml** - Dynamic config defining middleware instances with expressions and test cases
- **docker-compose.yml** - Mounts `./plugins-local:/plugins-local` so Traefik can find the plugin

## Troubleshooting

### Traefik won't start
- Check if plugin.wasm exists in project root: `ls -lh ../plugin.wasm`
- Run `./setup-plugin.sh` to ensure the plugin directory is set up
- View logs: `docker compose logs traefik`
- Look for compilation or test errors in Traefik output

### Tests fail
- Ensure Traefik is fully started before running tests
- Check Traefik logs for plugin errors: `docker compose logs traefik`
- Verify plugin loaded successfully (look for middleware creation in logs)

### Port conflicts
- Ports bound to 127.0.0.1:8080 (HTTP) and 127.0.0.1:8081 (Dashboard)
- Use hardened mode (run-tests.sh) to avoid port binding altogether
- Modify ports in docker-compose.yml if needed for legacy mode

## Traefik Dashboard

When running in legacy mode with port exposure, access Traefik dashboard at: http://127.0.0.1:8081

View middlewares, routers, and services configuration.

Note: Dashboard is not accessible in hardened mode (tests run inside Docker network).

## GitHub Actions Integration

The integration tests can be run in CI/CD using the hardened mode:

```yaml
- name: Build release WASM
  run: make release

- name: Run integration tests
  run: |
    cd integration-test
    ./run-tests.sh
```

Or using the legacy approach for more control:

```yaml
- name: Build release WASM
  run: make release

- name: Run integration tests
  run: |
    cd integration-test
    ./setup-plugin.sh
    docker compose up -d traefik backend
    sleep 10  # Wait for Traefik startup
    TRAEFIK_URL=http://localhost:8080 ./test.sh
    docker compose logs traefik
    docker compose down
```

## Notes

- **Local plugin loading**: The plugin must follow Traefik's strict directory structure under `plugins-local/`
- **WASM experimental feature**: Requires Traefik v3.0+ with `experimental.localPlugins`
- **Startup tests**: Test cases in dynamic.yml are validated when Traefik starts - if any fail, Traefik will abort
