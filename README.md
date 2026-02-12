# Traefik WASM Authorization Middleware Plugin

[![CI](https://github.com/USER/REPO/actions/workflows/ci.yml/badge.svg)](https://github.com/USER/REPO/actions/workflows/ci.yml)

A Traefik middleware plugin that performs attribute-based authorization on HTTP requests using a custom expression language.

## Overview

This plugin enables fine-grained access control based on HTTP request attributes:
- Request method, path, and host
- HTTP headers (ideal for ForwardAuth integration)
- Custom expression language with type safety
- Built-in test framework for validation at startup

## Features

- **Expression-based authorization**: Define complex access rules using a simple expression language
- **Type-safe compilation**: Expressions are compiled and type-checked at Traefik startup
- **Built-in testing**: Test cases validated at startup - Traefik won't start with invalid config
- **Minimal overhead**: Small WASM binary (< 200 KB) with fast evaluation
- **Fail-closed security**: Any evaluation errors result in request denial

## Quick Start

### Configuration

```yaml
http:
  middlewares:
    team-auth:
      plugin:
        authz:
          expression: 'contains(headerList("X-Auth-User-Teams"), "platform-eng")'
          denyStatusCode: 403
          denyBody: "Access denied: requires platform-eng team membership"
          tests:
            - name: "platform-eng team allowed"
              request:
                headers:
                  X-Auth-User-Teams: "platform-eng,devops"
              expect: true
            - name: "other teams denied"
              request:
                headers:
                  X-Auth-User-Teams: "marketing"
              expect: false
```

## Expression Language Reference

### Built-in Identifiers
- `method` - HTTP request method (GET, POST, etc.)
- `path` - Request path
- `host` - Request host

### Comparison Operators
- `==` - String equality
- `!=` - String inequality
- `startsWith` - String prefix match
- `endsWith` - String suffix match
- `contains` - Substring match
- `matches` - Regex match (RE2 syntax)

### Boolean Operators
- `AND` - Logical AND
- `OR` - Logical OR
- `NOT` - Logical NOT

### Built-in Functions
- `header(name)` - Get first header value (empty string if missing)
- `headerValues(name)` - Get all header values as array
- `headerList(name)` - Get header value split by comma into array
- `contains(list, item)` - Check if array contains item
- `anyOf(list, item1, item2, ...)` - Check if array contains any of the items
- `allOf(list, item1, item2, ...)` - Check if array contains all of the items

### Expression Examples

```
# Simple method check
method == "GET"

# Path-based access
path startsWith "/api/admin"

# Header-based authorization
contains(headerList("X-Auth-User-Teams"), "platform-eng")

# Complex boolean logic
(method == "GET" OR method == "HEAD") AND path startsWith "/public"

# Regex matching
matches(path, "^/api/v[0-9]+/.*")

# Multiple team check
anyOf(headerList("X-Auth-User-Teams"), "platform-eng", "devops", "sre")
```

## Configuration Schema

- `expression` (string, required) - Authorization expression
- `denyStatusCode` (int, default: 403) - HTTP status code for denied requests
- `denyBody` (string, default: "Forbidden") - Response body for denied requests
- `tests` (array, optional) - Test cases validated at startup

### Test Case Schema

- `name` (string) - Descriptive test name
- `request` (object) - Mock request
  - `method` (string, optional) - HTTP method
  - `path` (string, optional) - Request path
  - `host` (string, optional) - Request host
  - `headers` (map, optional) - Request headers
- `expect` (boolean) - Expected authorization result (true = allow, false = deny)

## Build Instructions

### Prerequisites

- Rust toolchain (stable)
- `wasm32-wasip1` target: `rustup target add wasm32-wasip1`

### Building

```bash
# Debug build
make build

# Release build (optimized, < 200 KB)
make release

# Run tests
make test

# Lint and format check
make check
```

## Development

See `/workspace/docs/DESIGN.md` for comprehensive design documentation.

See `/workspace/docs/TASKS.md` for implementation progress tracking.

## Examples

See `/workspace/examples/` directory for complete Traefik configuration examples.

## License

MIT
