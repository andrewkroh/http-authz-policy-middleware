![authz-ferris.png](docs/authz-ferris.png)

# HTTP Authorization Policy Middleware

[![CI](https://github.com/andrewkroh/http-authz-policy-middleware/actions/workflows/ci.yml/badge.svg)](https://github.com/andrewkroh/http-authz-policy-middleware/actions/workflows/ci.yml)

A Traefik middleware plugin that performs attribute-based authorization on HTTP requests using a type-safe expression language.

## Overview

This plugin enables fine-grained access control based on HTTP request attributes (method, path, host, headers) using a custom expression language. Expressions are compiled and type-checked at Traefik startup, catching configuration errors before they reach production.

**Key Features:**
- Type-safe expression language with compile-time validation
- Built-in test framework validated at startup
- Fail-closed security model (errors deny access)
- Case-insensitive header lookups
- Minimal overhead (compiled WASM)

## Quick Start

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

### Operators
- `==`, `!=` - String equality/inequality
- `startsWith`, `endsWith` - String prefix/suffix match
- `contains` - Substring match
- `matches` - Regex match (RE2 syntax)
- `AND`, `OR`, `NOT` - Boolean operators

### Built-in Functions
- `header(name)` - Get first header value (empty string if missing)
- `headerValues(name)` - Get all header values as array
- `headerList(name)` - Get header value split by comma into array
- `contains(list, item)` - Check if array contains item
- `anyOf(list, item1, item2, ...)` - Check if array contains any of the items
- `allOf(list, item1, item2, ...)` - Check if array contains all of the items

### Examples

```
# Method check
method == "GET"

# Path-based access
path startsWith "/api/admin"

# Team membership
contains(headerList("X-Auth-User-Teams"), "platform-eng")

# Complex logic
(method == "GET" OR method == "HEAD") AND path startsWith "/public"

# Regex
matches(path, "^/api/v[0-9]+/.*")

# Multiple teams
anyOf(headerList("X-Auth-User-Teams"), "platform-eng", "devops", "sre")
```

## Configuration Schema

**Middleware Configuration:**
- `expression` (string, required) - Authorization expression
- `denyStatusCode` (int, default: 403) - HTTP status for denied requests
- `denyBody` (string, default: "Forbidden") - Response body for denied requests
- `tests` (array, optional) - Test cases validated at startup

**Test Case Schema:**
- `name` (string) - Test description
- `request` (object) - Mock request with `method`, `path`, `host`, `headers`
- `expect` (boolean) - Expected result (true = allow, false = deny)

## Examples

Complete Traefik configurations in [`examples/`](examples/):
- [team-based-access.yml](examples/team-based-access.yml) - Team membership authorization
- [path-restrictions.yml](examples/path-restrictions.yml) - API path restrictions
- [combined-rules.yml](examples/combined-rules.yml) - Complex boolean logic

## Testing

The plugin includes comprehensive testing:

- **Unit tests** - Run with `cargo test`
- **Integration tests** - Docker-based end-to-end validation with hardened security (tests run inside Docker network)

See [integration-test/README.md](integration-test/README.md) for integration test details.

## Documentation

- **[CLAUDE.md](CLAUDE.md)** - Development workflow and contributor guide
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contributing guidelines and changelog system
- **[docs/DESIGN.md](docs/DESIGN.md)** - Comprehensive design documentation
- **[integration-test/README.md](integration-test/README.md)** - Integration testing guide

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:
- Setting up your development environment
- Writing conventional commit messages for automatic changelog generation
- Submitting pull requests
- Running tests

## License

MIT
