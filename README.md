# HTTP Authorization Policy Middleware

[![CI](https://github.com/andrewkroh/http-authz-policy-middleware/actions/workflows/ci.yml/badge.svg)](https://github.com/andrewkroh/http-authz-policy-middleware/actions/workflows/ci.yml)

A Traefik middleware plugin that performs attribute-based authorization on HTTP requests using a type-safe expression language.

## Overview

This plugin enables fine-grained access control based on HTTP request attributes:
- Request method, path, and host
- HTTP headers (ideal for ForwardAuth integration)
- Custom expression language with type safety
- Built-in test framework for validation at startup

## Features

- **Expression-based authorization**: Define complex access rules using a simple, powerful expression language
- **Type-safe compilation**: Expressions are compiled and type-checked at Traefik startup - catch errors before they hit production
- **Built-in testing framework**: Test cases validated at startup - Traefik won't start with invalid config
- **Fail-closed security**: Any evaluation errors result in request denial (HTTP 500)
- **Minimal overhead**: Compiled WASM binary (~960 KB) with fast runtime evaluation
- **Case-insensitive headers**: All header lookups are case-insensitive for HTTP/1.1 and HTTP/2 compatibility
- **Rich built-in functions**: Header access, array operations, regex matching, and more

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

See the [`examples/`](examples/) directory for complete Traefik configuration examples:

- **[team-based-access.yml](examples/team-based-access.yml)** - Control access based on team membership headers
- **[path-restrictions.yml](examples/path-restrictions.yml)** - Restrict access to specific API paths
- **[combined-rules.yml](examples/combined-rules.yml)** - Complex boolean logic combining multiple conditions

## Troubleshooting

### Expression Compilation Errors

If Traefik fails to start with compilation errors, check:
- Expression syntax is correct (operators, parentheses match)
- All identifiers are valid (`method`, `path`, `host`)
- Function names and argument counts are correct
- Top-level expression evaluates to boolean

### Test Failures at Startup

When test cases fail:
- Check that `expect` matches the actual expression result
- Verify test request properties (method, path, host, headers)
- Use simple expressions to isolate the issue
- Check Traefik logs for detailed error messages

### Runtime Evaluation Errors

Evaluation errors return HTTP 500:
- Usually caused by invalid regex patterns in `matches()` operator
- Check Traefik logs for detailed error messages
- All other type errors are caught at compile time

### Header Access

**Note**: Current WASM implementation has limited header access due to http-wasm-guest 0.7 API constraints. Header functions may not work as expected until this is enhanced.

For full header support, expressions using `header()`, `headerValues()`, and `headerList()` should be tested with the built-in test framework.

## Performance

- Expression compilation happens once at Traefik startup
- Runtime evaluation is fast (compiled AST, not interpreted)
- Regex patterns in `matches()` are compiled on demand (consider caching optimization)
- Boolean operators use short-circuit evaluation

## Security

- **Fail-closed**: Unexpected errors return HTTP 500, never allow unauthorized access
- **Type safety**: Invalid expressions caught at compile time before any requests are processed
- **RE2 regex**: Linear-time regex matching prevents ReDoS attacks
- **No code injection**: Expressions are parsed into AST, not executed as code

## Contributing

Contributions welcome! Please:
1. Run tests: `cargo test`
2. Format code: `cargo fmt`
3. Lint: `cargo clippy --target wasm32-wasip1`
4. Ensure CI passes before submitting PR

## License

MIT
