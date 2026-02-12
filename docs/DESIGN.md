# Design Document: Traefik WASM Authorization Middleware Plugin

**Status:** Draft — Ready for Review  
**Author:** Andrew Kroh  
**Date:** 2025-02-12

-----

## 1. Overview

This document describes the design of a Traefik middleware plugin that performs attribute-based authorization (authZ) on HTTP requests. The plugin is compiled to WebAssembly (WASM) and runs inside Traefik via the [http-wasm HTTP Handler ABI](https://http-wasm.io/http-handler-abi/).

The plugin evaluates a user-defined expression against properties of each incoming request — method, path, host, and headers. If the expression evaluates to `true`, the request is forwarded to the next middleware. If `false`, the plugin returns a configurable HTTP status code (default 403).

The expression language is custom-built and tailored to the HTTP authorization domain. A built-in test framework allows operators to validate their expressions at Traefik startup time, catching configuration errors before traffic is served.

### 1.1 Motivating Use Case

An upstream ForwardAuth middleware authenticates users via GitHub OAuth and injects the following headers into requests:

|Header             |Value                                        |Example           |
|-------------------|---------------------------------------------|------------------|
|`X-Auth-User-Login`|GitHub username                              |`jdoe`            |
|`X-Auth-User-Id`   |GitHub numeric user ID                       |`12345678`        |
|`X-Auth-User-Email`|User’s email (if available)                  |`jdoe@company.com`|
|`X-Auth-User-Teams`|Comma-separated list of team slugs within org|`platform-eng,sre`|

This plugin sits downstream of that ForwardAuth middleware and enforces team-based (or other attribute-based) access policies, for example: *“only members of the `platform-eng` or `sre` teams may access this service.”*

However, the plugin is **general-purpose** — it has no hardcoded knowledge of GitHub headers or team semantics. It operates purely on HTTP method, path, host, and header values.

-----

## 2. Architecture

### 2.1 Runtime Environment

The plugin is compiled to WebAssembly targeting `wasm32-wasip1` and loaded by Traefik’s WASM plugin engine. It implements the [http-wasm HTTP Handler ABI](https://http-wasm.io/http-handler-abi/).

The implementation language is an open decision for reviewers (see **Section 5: Implementation Language**). Both options have proven http-wasm guest libraries:

|Language   |Guest Library                                                                       |Reference Plugin                                                   |
|-----------|------------------------------------------------------------------------------------|-------------------------------------------------------------------|
|Go (TinyGo)|[http-wasm-guest-tinygo](https://github.com/http-wasm/http-wasm-guest-tinygo) v0.4.0|[traefik/plugindemowasm](https://github.com/traefik/plugindemowasm)|
|Rust       |[http-wasm-guest](https://crates.io/crates/http-wasm-guest) v0.7.0                  |[crate examples](https://crates.io/crates/http-wasm-guest)         |

### 2.2 Plugin Lifecycle

```
┌──────────────────────────────────────────────────────┐
│                  Traefik Startup                     │
│                                                      │
│  1. Load WASM binary                                 │
│  2. Call init() / main()                             │
│     a. Read config JSON via handler.Host.GetConfig() │
│     b. Parse JSON into Config struct                 │
│     c. Compile expression (catch syntax errors)      │
│     d. Run all test cases against compiled expr      │
│     e. If any step fails → log error, os.Exit(1)    │
│     f. Register handleRequest function               │
│                                                      │
│  Traefik refuses to start if plugin init fails.      │
└──────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────┐
│              Per-Request Handling                     │
│                                                      │
│  1. Build RequestContext from api.Request             │
│     - method, path, host (normalized)                │
│     - headers (case-insensitive access)              │
│  2. Evaluate compiled expression against context     │
│  3. If true  → return (next=true, 0)                 │
│  4. If false → set status code, write body, return   │
│                (next=false, 0)                        │
└──────────────────────────────────────────────────────┘
```

### 2.3 Dependencies

**If Go (TinyGo):**

|Dependency                                   |Purpose                       |
|---------------------------------------------|------------------------------|
|`github.com/http-wasm/http-wasm-guest-tinygo`|http-wasm guest ABI for TinyGo|

All other functionality uses Go standard library packages available in TinyGo: `encoding/json`, `strings`, `fmt`, `regexp`, `strconv`, `os`.

**If Rust:**

|Dependency            |Purpose                     |
|----------------------|----------------------------|
|`http-wasm-guest`     |http-wasm guest ABI for Rust|
|`serde` + `serde_json`|JSON config deserialization |
|`regex`               |Regular expression support  |

The expression engine is custom-built in either case, with no additional dependencies.

### 2.4 Build Command

**Go (TinyGo):**

```bash
tinygo build -o plugin.wasm -scheduler=none --no-debug -target=wasi main.go
```

**Rust:**

```bash
cargo build --target wasm32-wasip1 --release
cp target/wasm32-wasip1/release/traefik_authz.wasm plugin.wasm
```

-----

## 3. Configuration

Configuration is passed to the plugin as JSON bytes via `handler.Host.GetConfig()`. Traefik delivers this from the dynamic middleware configuration.

### 3.1 Config Schema

```json
{
  "expression": "<expression string>",
  "denyStatusCode": 403,
  "denyBody": "Forbidden",
  "tests": [
    {
      "name": "<human-readable test name>",
      "request": {
        "method": "GET",
        "path": "/api/v1/resource",
        "host": "app.example.com",
        "headers": {
          "X-Auth-User-Teams": "platform-eng,sre",
          "X-Auth-User-Login": "jdoe"
        }
      },
      "expect": true
    }
  ]
}
```

|Field           |Type        |Required|Default      |Description                                       |
|----------------|------------|--------|-------------|--------------------------------------------------|
|`expression`    |`string`    |Yes     |—            |The authorization expression to evaluate          |
|`denyStatusCode`|`int`       |No      |`403`        |HTTP status code returned when expression is false|
|`denyBody`      |`string`    |No      |`"Forbidden"`|Response body returned when expression is false   |
|`tests`         |`[]TestCase`|No      |`[]`         |Test cases validated at startup                   |

**TestCase fields:**

|Field    |Type         |Required|Description                                    |
|---------|-------------|--------|-----------------------------------------------|
|`name`   |`string`     |Yes     |Descriptive name shown on failure              |
|`request`|`TestRequest`|Yes     |Mock request to evaluate the expression against|
|`expect` |`bool`       |Yes     |Expected result of the expression              |

**TestRequest fields:**

|Field    |Type               |Required|Default|Description                                                   |
|---------|-------------------|--------|-------|--------------------------------------------------------------|
|`method` |`string`           |No      |`"GET"`|HTTP method                                                   |
|`path`   |`string`           |No      |`"/"`  |Request path (without query string)                           |
|`host`   |`string`           |No      |`""`   |Host header value                                             |
|`headers`|`map[string]string`|No      |`{}`   |Headers to include in the mock request (keys case-insensitive)|

### 3.2 Traefik Configuration Example

**Static configuration (load the plugin):**

```yaml
experimental:
  plugins:
    authz:
      moduleName: github.com/yourorg/traefik-authz-wasm
      version: v1.0.0
```

**Dynamic configuration (use the plugin as middleware):**

```yaml
http:
  routers:
    my-router:
      rule: Host(`app.example.com`)
      service: my-service
      entryPoints:
        - web
      middlewares:
        - platform-team-only

  middlewares:
    platform-team-only:
      plugin:
        authz:
          expression: >-
            anyOf(headerList("X-Auth-User-Teams"), "platform-eng", "sre")
          denyStatusCode: 403
          denyBody: "Access denied: requires platform-eng or sre team membership"
          tests:
            - name: "platform-eng member is allowed"
              request:
                method: GET
                path: /api/v1/deploy
                headers:
                  X-Auth-User-Teams: "platform-eng,devops"
              expect: true
            - name: "marketing member is denied"
              request:
                method: GET
                path: /api/v1/deploy
                headers:
                  X-Auth-User-Teams: "marketing"
              expect: false
            - name: "missing teams header is denied"
              request:
                method: GET
                path: /api/v1/deploy
              expect: false

  services:
    my-service:
      loadBalancer:
        servers:
          - url: http://127.0.0.1:8080
```

### 3.3 Plugin Manifest (`.traefik.yml`)

```yaml
displayName: Attribute-Based Authorization
type: middleware
runtime: wasm
summary: >-
  Authorization middleware that evaluates expressions against request
  attributes (method, path, headers). Ideal for enforcing team-based
  access policies from upstream ForwardAuth headers.

testData:
  expression: 'header("X-Test") == "pass"'
  tests:
    - name: "catalog validation"
      request:
        headers:
          X-Test: "pass"
      expect: true
```

-----

## 4. Expression Language

The plugin implements a custom expression language optimized for HTTP request authorization. It is a small, deterministic, side-effect-free language with the following design goals:

- Zero external dependencies (no `reflect`, TinyGo-safe)
- Complete and exhaustive documentation (no hidden features)
- Fast failure: syntax and type errors caught at parse time
- Focused on the domain: string comparisons, list membership, boolean logic

### 4.1 Grammar

```
expr        → or_expr
or_expr     → and_expr ("OR" and_expr)*
and_expr    → not_expr ("AND" not_expr)*
not_expr    → "NOT" not_expr | comparison
comparison  → value (comp_op value)?
            | func_call
            | "(" expr ")"
comp_op     → "==" | "!=" | "startsWith" | "endsWith"
            | "contains" | "matches"
value       → STRING | func_call | IDENT
func_call   → IDENT "(" arg_list? ")"
arg_list    → expr ("," expr)*
STRING      → '"' <characters> '"'
IDENT       → "method" | "path" | "host"
```

Operator precedence (highest to lowest):

1. `NOT`
2. `AND`
3. `OR`

### 4.2 Types

The language has three types. There is no implicit type coercion.

|Type      |Produced by                                                      |Consumed by                         |
|----------|-----------------------------------------------------------------|------------------------------------|
|`string`  |String literals, `method`, `path`, `host`, `header()`            |Comparison operators, function args |
|`[]string`|`headerValues()`, `headerList()`                                 |`contains()`, `anyOf()`, `allOf()`  |
|`bool`    |Comparisons, `contains()`, `anyOf()`, `allOf()`, `NOT`/`AND`/`OR`|`AND`, `OR`, `NOT`, top-level result|

The top-level expression **must** evaluate to `bool`. A type error at any point is caught during parsing/compilation (not at request evaluation time).

### 4.3 Built-in Identifiers

|Identifier|Type    |Description                           |
|----------|--------|--------------------------------------|
|`method`  |`string`|HTTP method, e.g. `"GET"`, `"POST"`   |
|`path`    |`string`|Request URI path, e.g. `"/api/v1/foo"`|
|`host`    |`string`|Host header value                     |

### 4.4 Built-in Functions

|Function                        |Signature                     |Description                                                                                                                 |
|--------------------------------|------------------------------|----------------------------------------------------------------------------------------------------------------------------|
|`header(name)`                  |`(string) → string`           |Returns the first value of the named header, or `""` if missing. Case-insensitive lookup.                                   |
|`headerValues(name)`            |`(string) → []string`         |Returns all values of the named header, or empty list if missing. Case-insensitive lookup.                                  |
|`headerList(name)`              |`(string) → []string`         |Returns the first value of the named header split by comma with whitespace trimmed. Returns empty list if header is missing.|
|`contains(list, item)`          |`([]string, string) → bool`   |Returns `true` if `item` is in `list`.                                                                                      |
|`anyOf(list, item1, item2, ...)`|`([]string, string...) → bool`|Returns `true` if **any** of the given items is in `list`.                                                                  |
|`allOf(list, item1, item2, ...)`|`([]string, string...) → bool`|Returns `true` if **all** of the given items are in `list`.                                                                 |

### 4.5 Comparison Operators

All comparison operators take `(string, string)` and return `bool`.

|Operator    |Description                                                            |Example                           |
|------------|-----------------------------------------------------------------------|----------------------------------|
|`==`        |Exact string equality                                                  |`method == "GET"`                 |
|`!=`        |String inequality                                                      |`header("X-Env") != "production"` |
|`startsWith`|Left operand starts with right operand                                 |`path startsWith "/api/"`         |
|`endsWith`  |Left operand ends with right operand                                   |`path endsWith "/health"`         |
|`contains`  |Left operand contains right operand as substring                       |`header("Accept") contains "json"`|
|`matches`   |Left operand matches right operand as a regular expression (RE2 syntax)|`path matches "^/api/v[0-9]+/"`   |

**Note on `contains`:** When used as an infix operator (`string contains string`), it performs a substring check. When used as a function call (`contains([]string, string)`), it performs list membership. These are distinct operations resolved by the parser based on argument types.

### 4.6 Header Case Normalization

HTTP/2 lowercases all header names. HTTP/1.1 preserves case. To ensure expressions work consistently regardless of protocol version, **all header lookups are case-insensitive**. The following expressions are equivalent:

```
header("X-Auth-User-Teams")
header("x-auth-user-teams")
header("X-AUTH-USER-TEAMS")
```

This is implemented by lowercasing both the lookup key and the stored header names when building the request context.

### 4.7 Expression Examples

**Team-based access:**

```
# Allow a single team
contains(headerList("X-Auth-User-Teams"), "platform-eng")

# Allow any of several teams
anyOf(headerList("X-Auth-User-Teams"), "platform-eng", "sre", "devops")

# Require membership in multiple teams
allOf(headerList("X-Auth-User-Teams"), "platform-eng", "on-call")
```

**Path-based restrictions:**

```
# Admin paths require admin team
NOT (path startsWith "/admin") OR contains(headerList("X-Auth-User-Teams"), "admin")

# Read-only access for non-privileged teams
method == "GET" OR anyOf(headerList("X-Auth-User-Teams"), "platform-eng", "sre")
```

**Combined conditions:**

```
# Platform team can do anything; others are read-only on /api
contains(headerList("X-Auth-User-Teams"), "platform-eng")
  OR (path startsWith "/api" AND method == "GET")
```

**User-specific access:**

```
header("X-Auth-User-Login") == "deploy-bot"
```

**Regex matching:**

```
path matches "^/api/v[0-9]+/(deploy|rollback)"
  AND anyOf(headerList("X-Auth-User-Teams"), "platform-eng", "sre")
```

-----

## 5. Implementation Language

**Decision required:** The implementation language is an open choice between **Go (TinyGo)** and **Rust**. The expression language specification (Section 4), configuration schema (Section 3), test framework (Section 6), and overall architecture are all language-agnostic. This section presents both options for reviewer decision.

### 5.1 Recommendation Summary

Rust is the stronger technical choice for this specific project. Go is the pragmatic choice if the team wants to minimize ramp-up time. Either will produce a correct, performant plugin.

### 5.2 Arguments for Rust

**Language-level fit for the problem domain.** The core of this project is a small language implementation: a lexer, parser, AST, type-checker, and evaluator. Rust’s `enum` with data variants and exhaustive `match` is purpose-built for this. Token types, AST nodes, and evaluated values each become a single `enum`, and the compiler refuses to let you miss a case. In Go, these are modeled as interface types with type switches, where a missing case is a silent bug discovered at runtime.

For example, the AST value type in Rust:

```rust
enum Value {
    Str(String),
    StrList(Vec<String>),
    Bool(bool),
}
```

A `match` on `Value` that forgets a variant is a compile error. The Go equivalent (`interface{}` or a custom interface with type switches) provides no such guarantee.

**First-class WASM target.** Rust’s `wasm32-wasip1` target is mature and used extensively in production (Cloudflare Workers, Fastly Compute, Fermyon Spin, etc.). There is no “TinyGo” indirection layer — the full language and standard library are available. The entire reason this design uses a custom expression engine instead of the `expr-lang/expr` library is TinyGo’s incomplete `reflect` support. With Rust, this class of problem does not exist.

**Smaller binaries.** Rust WASM binaries are typically 50–200 KB for a project of this scope. TinyGo binaries for equivalent functionality are often 500 KB–1 MB due to the bundled garbage collector runtime and runtime support code.

**No garbage collector.** Rust’s ownership model means no GC pauses, no GC overhead in the binary, and deterministic memory behavior — desirable properties for middleware in a hot request path.

**Proven http-wasm guest crate.** The [http-wasm-guest](https://crates.io/crates/http-wasm-guest) crate (v0.7.0) provides a typed Rust API for the http-wasm ABI. It is explicitly documented as targeting Traefik plugin development. Example from the crate:

```rust
use http_wasm_guest::{Guest, host::{Bytes, Request, Response}, register};

struct Plugin;

impl Guest for Plugin {
    fn handle_request(&self, request: Request, _response: Response) -> (bool, i32) {
        request.header().add(&Bytes::from("X-Foo"), &Bytes::from("Bar"));
        (true, 0)
    }
}

fn main() {
    register(Plugin);
}
```

**Regex safety.** Rust’s `regex` crate uses the same RE2 algorithm as Go’s `regexp` package, guaranteeing linear-time matching with no catastrophic backtracking.

**Richer error handling.** Rust’s `Result<T, E>` and `?` operator produce cleaner error propagation in the parser and evaluator than Go’s `if err != nil` chains, reducing boilerplate in exactly the code that needs to be most readable.

### 5.3 Arguments for Go (TinyGo)

**Team familiarity.** The team is primarily Go developers. A Go implementation is immediately reviewable, maintainable, and contributable by the whole team without a language learning curve.

**Ecosystem alignment.** Traefik is a Go project. The official plugin demo, the primary documentation, and community examples are all Go. Debugging integration issues is easier when the plugin and host share a language ecosystem.

**Faster initial development.** For developers already fluent in Go, the implementation time is lower. The expression engine is ~300-400 lines of Go — small enough that TinyGo’s quirks (no `reflect`, limited stdlib) are manageable.

**Simpler CI/CD.** TinyGo is a single binary install. The build step is one command. Rust requires the `wasm32-wasip1` target to be installed via `rustup`, and Cargo builds involve more moving parts (though this is a one-time setup cost).

### 5.4 Comparison Matrix

|Factor                         |Go (TinyGo)                                                                  |Rust                                                       |
|-------------------------------|-----------------------------------------------------------------------------|-----------------------------------------------------------|
|AST / enum modeling            |Interface + type switches (no exhaustiveness check)                          |`enum` + `match` (compile-time exhaustiveness)             |
|WASM target maturity           |TinyGo WASI (workable but has gaps)                                          |`wasm32-wasip1` (first-class, battle-tested)               |
|Binary size (estimated)        |500 KB – 1 MB                                                                |50 – 200 KB                                                |
|Garbage collector              |Yes (bundled in binary)                                                      |None                                                       |
|`reflect` limitations          |Severe — drove the custom expr engine decision                               |Not applicable                                             |
|http-wasm guest library        |[http-wasm-guest-tinygo](https://github.com/http-wasm/http-wasm-guest-tinygo)|[http-wasm-guest](https://crates.io/crates/http-wasm-guest)|
|Regex engine                   |`regexp` (RE2, linear-time)                                                  |`regex` crate (RE2, linear-time)                           |
|Team familiarity               |High                                                                         |Low (but growing)                                          |
|Traefik ecosystem alignment    |Strong                                                                       |Moderate (crate docs reference Traefik explicitly)         |
|Error handling ergonomics      |`if err != nil` chains                                                       |`Result<T, E>` + `?` operator                              |
|Estimated implementation effort|1–2 days                                                                     |2–4 days (including Rust ramp-up)                          |

-----

## 6. Implementation Structure

The architecture is the same regardless of language. This section shows both layouts and key type sketches.

### 6.1 Package Layout

**Go:**

```
traefik-authz-wasm/
├── main.go              # Plugin entrypoint: init(), config loading, handler registration
├── config.go            # Config struct, JSON parsing, test runner
├── expr/
│   ├── lexer.go         # Tokenizer
│   ├── parser.go        # Recursive-descent parser → AST
│   ├── ast.go           # AST node types
│   ├── compiler.go      # Type-checks AST → Program
│   ├── eval.go          # Evaluates Program against RequestContext
│   └── eval_test.go     # Unit tests (run with `go test`, not TinyGo)
├── context.go           # RequestContext: built from api.Request or TestRequest
├── go.mod
├── go.sum
├── .traefik.yml
├── Makefile
└── README.md
```

**Rust:**

```
traefik-authz-wasm/
├── src/
│   ├── main.rs          # Plugin entrypoint: config loading, handler registration
│   ├── config.rs        # Config structs, serde deserialization, test runner
│   ├── context.rs       # RequestContext: built from http-wasm Request or TestRequest
│   └── expr/
│       ├── mod.rs
│       ├── lexer.rs     # Tokenizer
│       ├── parser.rs    # Recursive-descent parser → AST
│       ├── ast.rs       # AST node types (enums)
│       ├── compiler.rs  # Type-checks AST → Program
│       └── eval.rs      # Evaluates Program against RequestContext
├── Cargo.toml
├── .traefik.yml
├── Makefile
└── README.md
```

### 6.2 Key Types (Go)

```go
// RequestContext is the data structure the expression evaluates against.
// It is built from a live api.Request or from a TestRequest at startup.
type RequestContext struct {
    Method  string
    Path    string
    Host    string
    headers map[string]string   // lowercase key → first value
    allHeaders map[string][]string // lowercase key → all values
}

// Methods called by the expression evaluator:
func (r *RequestContext) Header(name string) string
func (r *RequestContext) HeaderValues(name string) []string
func (r *RequestContext) HeaderList(name string) []string
```

```go
// Program is the compiled, type-checked expression ready for evaluation.
type Program struct {
    root ast.Node
}

func Compile(expression string) (*Program, error)
func (p *Program) Eval(ctx *RequestContext) (bool, error)
```

### 6.3 Key Types (Rust)

```rust
/// The data structure the expression evaluates against.
pub struct RequestContext {
    pub method: String,
    pub path: String,
    pub host: String,
    headers: HashMap<String, String>,       // lowercase key → first value
    all_headers: HashMap<String, Vec<String>>, // lowercase key → all values
}

impl RequestContext {
    pub fn header(&self, name: &str) -> &str { /* ... */ }
    pub fn header_values(&self, name: &str) -> &[String] { /* ... */ }
    pub fn header_list(&self, name: &str) -> Vec<String> { /* ... */ }
}

/// AST node types — exhaustive matching enforced by compiler.
pub enum Expr {
    BoolLiteral(bool),
    StringLiteral(String),
    Ident(Ident),                           // method, path, host
    FuncCall { name: String, args: Vec<Expr> },
    BinaryOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

pub enum Value {
    Str(String),
    StrList(Vec<String>),
    Bool(bool),
}

/// Compiled, type-checked expression.
pub struct Program {
    root: Expr,
}

impl Program {
    pub fn compile(expression: &str) -> Result<Self, CompileError> { /* ... */ }
    pub fn eval(&self, ctx: &RequestContext) -> Result<bool, EvalError> { /* ... */ }
}
```

### 6.4 Startup Flow (Go)

```go
func init() {
    configBytes := handler.Host.GetConfig()

    var config Config
    if err := json.Unmarshal(configBytes, &config); err != nil {
        handler.Host.Log(api.LogLevelError, "invalid config JSON: "+err.Error())
        os.Exit(1)
    }

    program, err := expr.Compile(config.Expression)
    if err != nil {
        handler.Host.Log(api.LogLevelError, "invalid expression: "+err.Error())
        os.Exit(1)
    }

    // Run test cases
    for _, tc := range config.Tests {
        ctx := NewRequestContextFromTest(tc.Request)
        result, err := program.Eval(ctx)
        if err != nil {
            handler.Host.Log(api.LogLevelError,
                fmt.Sprintf("test %q eval error: %v", tc.Name, err))
            os.Exit(1)
        }
        if result != tc.Expect {
            handler.Host.Log(api.LogLevelError,
                fmt.Sprintf("test %q failed: got %v, expected %v", tc.Name, result, tc.Expect))
            os.Exit(1)
        }
        handler.Host.Log(api.LogLevelInfo, fmt.Sprintf("test %q passed", tc.Name))
    }

    mw := &AuthzMiddleware{program: program, config: config}
    handler.HandleRequestFn = mw.handleRequest
}
```

### 6.5 Startup Flow (Rust)

```rust
fn main() {
    let config_bytes = http_wasm_guest::host::config();
    let config: Config = serde_json::from_slice(&config_bytes)
        .unwrap_or_else(|e| {
            log_error(&format!("invalid config JSON: {e}"));
            std::process::exit(1);
        });

    let program = Program::compile(&config.expression)
        .unwrap_or_else(|e| {
            log_error(&format!("invalid expression: {e}"));
            std::process::exit(1);
        });

    // Run test cases
    for tc in &config.tests {
        let ctx = RequestContext::from_test(&tc.request);
        match program.eval(&ctx) {
            Err(e) => {
                log_error(&format!("test {:?} eval error: {e}", tc.name));
                std::process::exit(1);
            }
            Ok(result) if result != tc.expect => {
                log_error(&format!(
                    "test {:?} failed: got {result}, expected {}", tc.name, tc.expect
                ));
                std::process::exit(1);
            }
            Ok(_) => log_info(&format!("test {:?} passed", tc.name)),
        }
    }

    register(AuthzPlugin { program, config });
}
```

### 6.6 Request Handler (Go)

```go
func (m *AuthzMiddleware) handleRequest(req api.Request, resp api.Response) (next bool, reqCtx uint32) {
    ctx := NewRequestContextFromAPI(req)

    allowed, err := m.program.Eval(ctx)
    if err != nil {
        handler.Host.Log(api.LogLevelError, "expression eval error: "+err.Error())
        resp.SetStatusCode(uint32(500))
        resp.Body().WriteString("Internal Server Error")
        return false, 0
    }

    if !allowed {
        resp.SetStatusCode(uint32(m.config.DenyStatusCode))
        resp.Body().WriteString(m.config.DenyBody)
        return false, 0
    }

    return true, 0
}
```

### 6.7 Request Handler (Rust)

```rust
impl Guest for AuthzPlugin {
    fn handle_request(&self, request: Request, response: Response) -> (bool, i32) {
        let ctx = RequestContext::from_request(&request);

        match self.program.eval(&ctx) {
            Err(e) => {
                log_error(&format!("expression eval error: {e}"));
                response.set_status_code(500);
                response.body().write(&Bytes::from("Internal Server Error"));
                (false, 0)
            }
            Ok(false) => {
                response.set_status_code(self.config.deny_status_code);
                response.body().write(&Bytes::from(self.config.deny_body.as_str()));
                (false, 0)
            }
            Ok(true) => (true, 0),
        }
    }
}
```

### 6.8 Expression Engine Design

The expression engine is a standard four-phase pipeline, identical in either language:

1. **Lexer**: Converts the expression string into a stream of tokens. Token types: `STRING`, `IDENT`, `LPAREN`, `RPAREN`, `COMMA`, `OP_EQ`, `OP_NEQ`, `OP_STARTS_WITH`, `OP_ENDS_WITH`, `OP_CONTAINS`, `OP_MATCHES`, `KW_AND`, `KW_OR`, `KW_NOT`, `EOF`.
2. **Parser**: Recursive-descent parser that consumes the token stream and produces an AST. The grammar is defined in Section 4.1. Parser errors include the token position for diagnostics.
3. **Compiler**: Walks the AST and performs type checking. Verifies that operators receive the correct types, that the top-level expression is `bool`, and that function calls have the correct arity and argument types. Returns a `Program` or a type error with location. In Rust, the type system enforces exhaustive handling of all AST variants at compile time. In Go, this must be enforced by convention and testing.
4. **Evaluator**: Walks the type-checked AST with a `RequestContext`, evaluating each node. Since the AST is already type-checked, the evaluator contains minimal error paths (primarily regex compilation errors for `matches`, which could optionally be moved to compile time).

In both languages, the engine operates on concrete types (`string`, `[]string`/`Vec<String>`, `bool`) with **zero use of `reflect`** (Go) or dynamic dispatch (Rust). This is critical for TinyGo compatibility and for Rust is simply idiomatic.

### 6.9 Regex Handling

The `matches` operator compiles the regex pattern at evaluation time or, preferably, at compile time for string-literal patterns. Both Go’s `regexp` package and Rust’s `regex` crate implement the RE2 algorithm, guaranteeing linear-time matching with no catastrophic backtracking. Regex patterns that are string literals should be compiled once during the `Compile` phase and cached in the AST node to avoid per-request compilation overhead.

-----

## 7. Test Framework

### 7.1 Purpose

Expressions are the primary configuration surface of this plugin. A typo or logic error in the expression can either lock out all users or grant access to unauthorized users. The built-in test framework mitigates this by allowing operators to define test cases alongside their expression, validated at Traefik startup.

### 7.2 How It Works

1. Each test case in the `tests` array defines a mock `request` (method, path, host, headers) and an `expect`ed boolean result.
2. The mock request is converted into a `RequestContext` using the same code path as live requests.
3. The compiled `Program` is evaluated against the `RequestContext`.
4. If the result does not match `expect`, the plugin logs a descriptive error and exits with a non-zero status, preventing Traefik from starting with a broken policy.

### 7.3 Test Case Design Guidance

Operators should include test cases that cover:

- **Positive cases:** Requests that should be allowed (e.g., correct team membership).
- **Negative cases:** Requests that should be denied (e.g., wrong team, missing header).
- **Edge cases:** Missing headers entirely, empty header values, boundary conditions in path matching.

Example with comprehensive coverage:

```json
{
  "expression": "anyOf(headerList(\"X-Auth-User-Teams\"), \"platform-eng\", \"sre\") AND path startsWith \"/api/\"",
  "tests": [
    {
      "name": "platform-eng on /api path → allow",
      "request": {
        "method": "POST",
        "path": "/api/v1/deploy",
        "headers": { "X-Auth-User-Teams": "platform-eng,devops" }
      },
      "expect": true
    },
    {
      "name": "sre on /api path → allow",
      "request": {
        "method": "GET",
        "path": "/api/v2/status",
        "headers": { "X-Auth-User-Teams": "sre" }
      },
      "expect": true
    },
    {
      "name": "platform-eng on non-api path → deny",
      "request": {
        "method": "GET",
        "path": "/dashboard",
        "headers": { "X-Auth-User-Teams": "platform-eng" }
      },
      "expect": false
    },
    {
      "name": "marketing on /api path → deny",
      "request": {
        "method": "GET",
        "path": "/api/v1/deploy",
        "headers": { "X-Auth-User-Teams": "marketing" }
      },
      "expect": false
    },
    {
      "name": "no teams header → deny",
      "request": {
        "method": "GET",
        "path": "/api/v1/deploy"
      },
      "expect": false
    },
    {
      "name": "empty teams header → deny",
      "request": {
        "method": "GET",
        "path": "/api/v1/deploy",
        "headers": { "X-Auth-User-Teams": "" }
      },
      "expect": false
    }
  ]
}
```

-----

## 8. Alternatives Considered

### 8.1 Use `github.com/expr-lang/expr` (Rejected)

[expr-lang/expr](https://github.com/expr-lang/expr) is a mature, well-tested Go expression library with a clean API and zero runtime dependencies. It was the original design choice.

**Why rejected:** The library makes extensive use of `reflect` for type-checking during compilation, for evaluating expressions against Go maps/structs, and for its built-in functions. TinyGo’s `reflect` support when targeting WASM is incomplete — specifically `reflect.MapOf`, `reflect.SliceOf`, dynamic type construction, and method reflection on interfaces are known to fail. This is a fundamental incompatibility.

**References:**

- [TinyGo language support — reflect](https://tinygo.org/docs/reference/lang-support/)
- [TinyGo issue tracker](https://github.com/tinygo-org/tinygo/issues?q=reflect+wasm)

### 8.2 Use Google CEL (Rejected)

[Common Expression Language (CEL)](https://github.com/google/cel-go) is Google’s expression language used in Kubernetes, Envoy, and Firebase. It is well-specified and has a Go implementation.

**Why rejected:** `cel-go` depends on protobuf (`google.golang.org/protobuf`), gRPC types, and makes heavy use of `reflect`. The dependency tree is far too large for TinyGo WASM compilation. The binary size alone would be problematic.

### 8.3 Use `github.com/Knetic/govaluate` (Rejected)

[govaluate](https://github.com/Knetic/govaluate) is a simpler expression evaluator for Go.

**Why rejected:** Also relies on `reflect` for parameter binding. Has not been actively maintained. Does not support typed list operations needed for the `headerList` / `contains` / `anyOf` pattern.

### 8.4 Use Starlark or Lua (Rejected)

Embeddable scripting languages like [Starlark (Go)](https://github.com/google/starlark-go) or [GopherLua](https://github.com/yuin/gopher-lua) would provide maximum expressiveness.

**Why rejected:** Both have large dependency trees, use `reflect`, and would produce enormous WASM binaries. Starlark-go alone is ~30k lines of Go. The security surface area of a general-purpose scripting language is also undesirable for a security-sensitive authorization middleware.

### 8.5 Structured Config / No Expression Language (Rejected)

Instead of an expression language, use a declarative JSON config with rule matching:

```json
{
  "rules": [
    {
      "match": { "teams": { "anyOf": ["platform-eng"] }, "path": { "startsWith": "/api/" } },
      "action": "allow"
    }
  ],
  "defaultAction": "deny"
}
```

**Why rejected:** Composing boolean logic becomes awkward quickly. A rule like *“platform-eng can do anything, but others need read-only access on /api paths only”* requires understanding implicit AND-within-rules and OR-across-rules semantics. Users inevitably need `NOT`, nested `OR`, and other patterns that make the structured config format increasingly complex to specify and reason about.

**Trade-off acknowledged:** A structured config requires no parser. For deployments with very simple policies (single team check), this approach is simpler. However, the expression language is small enough (~300-400 lines for the full lexer/parser/evaluator) that the implementation cost is modest, and the flexibility gain is significant.

### 8.6 Yaegi Plugin Instead of WASM (Considered — Valid Alternative)

Traefik also supports [Yaegi-based plugins](https://doc.traefik.io/traefik/extend/extend-traefik/) that interpret Go source at runtime. With Yaegi, the `expr-lang/expr` library could be used directly since Yaegi supports `reflect`.

**Why not chosen:** The project specifically targets the WASM plugin system for the following reasons:

- WASM plugins can be written in any language that compiles to WASM (future flexibility).
- WASM provides a stronger sandbox boundary than Yaegi’s interpreted Go.
- Traefik’s WASM support is the newer, more actively invested plugin path.
- Yaegi has its own compatibility quirks with certain Go packages and patterns.

This remains a valid alternative if the WASM constraint is relaxed.

-----

## 9. Security Considerations

### 9.1 Expression Safety

The expression language is intentionally limited:

- No loops or recursion in the language itself (the AST is a tree, not a graph).
- No variable assignment or side effects.
- No access to the filesystem, network, or environment variables.
- No string interpolation or template execution.
- Evaluation time is bounded: proportional to AST depth × number of headers.

### 9.2 Regex Denial of Service

The `matches` operator uses RE2-based regex engines (Go’s `regexp` or Rust’s `regex` crate), which guarantee linear-time matching with no catastrophic backtracking. This is safe for use in a hot request path.

### 9.3 Fail Closed

If expression evaluation encounters an unexpected error (which should not happen after type-checking, but is handled defensively), the plugin returns HTTP 500, **not** a pass-through. This ensures the middleware fails closed.

### 9.4 Header Trust

The plugin trusts headers as presented by the upstream middleware. It is the operator’s responsibility to ensure that the ForwardAuth (or equivalent) middleware upstream is correctly configured and that clients cannot forge the `X-Auth-User-*` headers directly. Traefik’s middleware chaining ensures this when configured correctly.

-----

## 10. Performance Considerations

- **Expression compilation** happens once at startup. There is no per-request parsing.
- **Header access** via the http-wasm ABI involves copying bytes across the WASM boundary. The plugin reads only the headers referenced in the expression (determined at compile time, if desired, as a future optimization). As a first implementation, headers are read lazily: `header()`, `headerValues()`, and `headerList()` call into the ABI on demand rather than pre-fetching all headers.
- **Regex patterns** with string-literal arguments are compiled once at startup and reused.
- **Memory:** The `RequestContext` is allocated per-request and is small (a few strings and a map). No persistent memory growth across requests. In Rust, memory is freed deterministically at the end of each request handler. In Go (TinyGo), the garbage collector handles cleanup, with a small runtime overhead.
- **Binary size:** Rust will produce significantly smaller WASM binaries (estimated 50–200 KB vs. 500 KB–1 MB for TinyGo), which affects plugin load time at Traefik startup.

-----

## 11. Future Considerations

These items are explicitly **out of scope** for v1 but are worth noting for future iterations:

- **Multiple rules with path matching:** Allow a list of `(path pattern, expression)` rules so different paths can have different policies in a single middleware instance.
- **Audit logging:** Log the evaluated expression result, matched headers, and user identity for denied requests.
- **Custom deny responses:** Support JSON or HTML response bodies, or response headers on deny.
- **`headerExists(name)` function:** Returns `bool`, avoids the pattern `header("X-Foo") != ""` (which is currently the workaround).
- **Query parameter access:** Add `query(name)` function for URL query parameter matching.
- **Source address matching:** Expose `sourceAddr` for IP-based rules.
- **Expression pre-optimization:** Constant folding, short-circuit evaluation hints.

-----

## 12. References

|Resource                           |URL                                                                                 |
|-----------------------------------|------------------------------------------------------------------------------------|
|http-wasm HTTP Handler ABI         |https://http-wasm.io/http-handler-abi/                                              |
|http-wasm Guest Library for TinyGo |https://github.com/http-wasm/http-wasm-guest-tinygo                                 |
|http-wasm Guest API (GoDoc)        |https://pkg.go.dev/github.com/http-wasm/http-wasm-guest-tinygo/handler/api          |
|http-wasm Guest Crate for Rust     |https://crates.io/crates/http-wasm-guest                                            |
|Traefik WASM Plugin Demo           |https://github.com/traefik/plugindemowasm                                           |
|Traefik Plugin Catalog - WASM Demo |https://plugins.traefik.io/plugins/6568c2afce37949adf28307e/demo-plugin-wasm        |
|Traefik Plugin Development Guide   |https://doc.traefik.io/traefik-hub/api-gateway/guides/plugin-development-guide      |
|Traefik v3 WASM Deep Dive          |https://traefik.io/blog/traefik-3-deep-dive-into-wasm-support-with-coraza-waf-plugin|
|TinyGo WASM/WASI Target            |https://tinygo.org/docs/guides/webassembly/wasm/                                    |
|Rust `wasm32-wasip1` Target        |https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip1.html                 |
|expr-lang/expr (rejected dep)      |https://github.com/expr-lang/expr                                                   |
|Go `regexp` (RE2 guarantees)       |https://pkg.go.dev/regexp                                                           |
|Rust `regex` crate (RE2 guarantees)|https://crates.io/crates/regex                                                      |

-----

## 13. Open Questions

1. **Implementation language: Go (TinyGo) or Rust?** See Section 5 for detailed comparison. Rust is the stronger technical fit; Go has lower ramp-up cost for the current team. This project is well-scoped enough to serve as a Rust learning opportunity if the team is interested.
2. **Should `matches` regex patterns be restricted to string literals only?** This would allow all regexes to be compiled at startup, eliminating any per-request regex compilation. The cost is that you cannot match against a dynamically constructed pattern (which is an unlikely use case in authZ).
3. **Should the `contains` operator be renamed when used as infix string-contains vs. function list-contains?** The current design overloads `contains` based on type context. An alternative is to use `has` for substring check and `contains` only for list membership. The overloading may confuse users.
4. **Should the plugin strip the `X-Auth-*` headers after evaluation?** This would prevent downstream services from seeing authentication metadata. This is a policy decision that may be better handled by a separate Traefik middleware (headers transform).
5. **How should the expression language handle multi-valued headers?** Currently `header()` returns the first value and `headerValues()` returns all values. The http-wasm ABI supports multi-valued headers natively. Is this sufficient, or should there be a `headerJoin(name, separator)` function?