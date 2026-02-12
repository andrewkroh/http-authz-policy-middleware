# Implementation Plan: Traefik WASM Authorization Middleware Plugin (Rust)

## Context

This project implements a Traefik middleware plugin that performs attribute-based authorization on HTTP requests. The plugin compiles to WebAssembly and runs inside Traefik via the http-wasm HTTP Handler ABI.

**Why this change is needed:**
- Enable team-based access control policies based on headers injected by upstream ForwardAuth middleware
- Provide a general-purpose expression-based authorization mechanism for HTTP requests
- Catch configuration errors at startup time through built-in testing framework
- Deliver a production-ready WASM plugin with minimal binary size and deterministic performance

**Implementation language decision:** Rust
- Better language-level fit for AST/parser implementation (exhaustive pattern matching)
- Smaller WASM binary size (50-200 KB vs 500 KB-1 MB for TinyGo)
- No garbage collector overhead
- First-class wasm32-wasip1 target support
- Strong type safety for expression engine

**Current state:** Project contains only `/workspace/docs/DESIGN.md` - no implementation code exists yet.

---

## Implementation Phases

### Phase 1: Project Scaffolding
**Goal:** Set up Rust project structure, dependencies, and build infrastructure

**Files to create:**
- `/workspace/Cargo.toml` - Rust project configuration with dependencies
- `/workspace/.traefik.yml` - Traefik plugin manifest
- `/workspace/Makefile` - Build automation
- `/workspace/README.md` - Project documentation
- `/workspace/.gitignore` - Git ignore patterns for Rust/WASM
- `/workspace/src/main.rs` - Minimal entrypoint stub

**Dependencies (Cargo.toml):**
```toml
[dependencies]
http-wasm-guest = "0.7.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.10"
```

**Build target:** `wasm32-wasip1`

**Verification:**
- `cargo build --target wasm32-wasip1` succeeds
- Produces `/workspace/target/wasm32-wasip1/debug/traefik_authz_wasm.wasm`

---

### Phase 2: GitHub Actions CI/CD Workflow
**Goal:** Set up continuous integration to verify every commit as implementation progresses

**File to create:**
- `/workspace/.github/workflows/ci.yml` - GitHub Actions workflow

**Workflow jobs:**

1. **Build and Test:**
   - Checkout code
   - Install Rust toolchain (stable)
   - Add `wasm32-wasip1` target
   - Cache cargo dependencies
   - Run `cargo fmt --check` (code formatting)
   - Run `cargo clippy --target wasm32-wasip1 -- -D warnings` (linting)
   - Run `cargo test` (unit and integration tests)
   - Run `cargo build --target wasm32-wasip1 --release` (release build)
   - Report binary size

2. **Triggers:**
   - Push to main branch
   - Pull requests
   - Manual workflow dispatch

**Workflow template:**
```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  workflow_dispatch:

jobs:
  build-and-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
          targets: wasm32-wasip1

      - name: Cache cargo dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Check formatting
        run: cargo fmt --check

      - name: Lint with clippy
        run: cargo clippy --target wasm32-wasip1 -- -D warnings

      - name: Run tests
        run: cargo test

      - name: Build release
        run: cargo build --target wasm32-wasip1 --release

      - name: Report binary size
        run: |
          ls -lh target/wasm32-wasip1/release/*.wasm
          SIZE=$(stat -f%z target/wasm32-wasip1/release/*.wasm 2>/dev/null || stat -c%s target/wasm32-wasip1/release/*.wasm)
          echo "Binary size: $(numfmt --to=iec-i --suffix=B $SIZE)"
```

**Commit strategy:**
- Commit after each major phase completion
- Each commit includes verification that CI passes
- Commit messages reference phase number and description

**Verification:**
- Workflow file is valid YAML
- Initial commit triggers workflow
- Workflow badge can be added to README.md
- Future commits will automatically trigger CI

---

### Phase 3: Configuration and Data Structures
**Goal:** Implement configuration parsing and core data structures

**Files to create:**
- `/workspace/src/config.rs` - Config struct, JSON deserialization, test case definitions
- `/workspace/src/context.rs` - RequestContext for evaluating expressions against requests

**Key types:**

**`config.rs`:**
- `struct Config` with fields: `expression`, `deny_status_code`, `deny_body`, `tests`
- `struct TestCase` with fields: `name`, `request`, `expect`
- `struct TestRequest` with fields: `method`, `path`, `host`, `headers`
- JSON deserialization using serde
- Default values: `deny_status_code` = 403, `deny_body` = "Forbidden"

**`context.rs`:**
- `struct RequestContext` with:
  - Public fields: `method`, `path`, `host` (all `String`)
  - Private fields: `headers: HashMap<String, String>` (lowercase key → first value)
  - Private fields: `all_headers: HashMap<String, Vec<String>>` (lowercase key → all values)
- Methods:
  - `pub fn from_request(request: &Request) -> Self` - Build from http-wasm Request
  - `pub fn from_test(test_req: &TestRequest) -> Self` - Build from TestRequest
  - `pub fn header(&self, name: &str) -> &str` - Get first header value (case-insensitive)
  - `pub fn header_values(&self, name: &str) -> &[String]` - Get all header values
  - `pub fn header_list(&self, name: &str) -> Vec<String>` - Get comma-separated list

**Header normalization:** All header lookups are case-insensitive (lowercase both stored keys and lookup keys)

**Verification:**
- Unit tests for `RequestContext::from_test()`
- Unit tests for case-insensitive header access
- JSON deserialization tests for Config struct

---

### Phase 4: Expression Language - Lexer
**Goal:** Implement tokenization of expression strings

**File to create:**
- `/workspace/src/expr/mod.rs` - Module declaration
- `/workspace/src/expr/lexer.rs` - Tokenizer implementation

**Token types (enum Token):**
- `String(String)` - String literal
- `Ident(String)` - Identifier (method, path, host, function names)
- `LParen`, `RParen`, `Comma`
- `OpEq`, `OpNeq`, `OpStartsWith`, `OpEndsWith`, `OpContains`, `OpMatches`
- `KwAnd`, `KwOr`, `KwNot`
- `Eof`

**Lexer struct:**
- `input: Vec<char>` - Input string as characters
- `pos: usize` - Current position
- `current_char: Option<char>` - Current character

**Methods:**
- `pub fn new(input: &str) -> Self`
- `pub fn next_token(&mut self) -> Result<Token, LexError>`
- Private helpers: `advance()`, `peek()`, `skip_whitespace()`, `read_string()`, `read_ident_or_keyword()`

**Keywords recognized:**
- `AND`, `OR`, `NOT`
- `startsWith`, `endsWith`, `contains`, `matches`

**Error handling:**
- `LexError` with position and message
- Handle unterminated strings, invalid characters

**Verification:**
- Unit tests tokenizing simple expressions: `method == "GET"`
- Unit tests for string literals with escaping
- Unit tests for all operators and keywords
- Error case tests (unterminated strings, etc.)

---

### Phase 5: Expression Language - Parser and AST
**Goal:** Build Abstract Syntax Tree from token stream

**File to create:**
- `/workspace/src/expr/ast.rs` - AST node definitions
- `/workspace/src/expr/parser.rs` - Recursive descent parser

**AST nodes (ast.rs):**
```rust
pub enum Expr {
    BoolLiteral(bool),
    StringLiteral(String),
    Ident(Ident),                                    // method, path, host
    FuncCall { name: String, args: Vec<Expr> },
    BinaryOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

pub enum Ident {
    Method,
    Path,
    Host,
}

pub enum BinOp {
    Eq,
    Neq,
    StartsWith,
    EndsWith,
    Contains,
    Matches,
}
```

**Parser (parser.rs):**
- `struct Parser` with `lexer: Lexer`, `current_token: Token`, `peek_token: Token`
- `pub fn parse(input: &str) -> Result<Expr, ParseError>`
- Recursive descent methods following grammar:
  - `parse_expr()` → `parse_or_expr()`
  - `parse_or_expr()` → `parse_and_expr()` (`OR` `parse_and_expr()`)*
  - `parse_and_expr()` → `parse_not_expr()` (`AND` `parse_not_expr()`)*
  - `parse_not_expr()` → `NOT` `parse_not_expr()` | `parse_comparison()`
  - `parse_comparison()` → `parse_value()` (comp_op `parse_value()`)?
  - `parse_value()` → string | func_call | ident | `(` expr `)`
  - `parse_func_call()` → ident `(` arg_list? `)`

**Operator precedence:** NOT > AND > OR

**Error handling:**
- `ParseError` with position and descriptive message
- Unexpected token errors
- Missing closing parenthesis

**Verification:**
- Unit test parsing: `method == "GET"`
- Unit test parsing: `contains(headerList("X-Auth-User-Teams"), "platform-eng")`
- Unit test parsing: `path startsWith "/api" AND method == "GET"`
- Unit test complex expression with nested logic
- Error cases: unclosed parenthesis, invalid syntax

---

### Phase 6: Expression Language - Type Checker and Compiler
**Goal:** Type-check AST and produce compiled Program

**File to create:**
- `/workspace/src/expr/compiler.rs` - Type checking and compilation

**Type system:**
```rust
enum Type {
    Str,
    StrList,
    Bool,
}
```

**Compiler:**
- `pub fn compile(input: &str) -> Result<Program, CompileError>`
- Type-check all AST nodes:
  - Ensure top-level expression is `bool`
  - Verify function arguments match expected types
  - Verify binary operators receive correct operand types
  - Check function arity

**Built-in function signatures:**
- `header(string) → string`
- `headerValues(string) → []string`
- `headerList(string) → []string`
- `contains([]string, string) → bool`
- `anyOf([]string, string...) → bool` (variadic)
- `allOf([]string, string...) → bool` (variadic)

**Comparison operators:** All take `(string, string) → bool`

**Program struct:**
```rust
pub struct Program {
    root: Expr,
    // Future optimization: precompiled regex patterns
}

impl Program {
    pub fn eval(&self, ctx: &RequestContext) -> Result<bool, EvalError>
}
```

**Error handling:**
- `CompileError` with position and type mismatch details
- Examples: "expected bool, got string", "function 'anyOf' requires at least 2 arguments"

**Verification:**
- Type error tests: `method AND path` (AND requires bool, got string)
- Type error tests: `contains("foo", "bar")` (first arg must be []string)
- Valid expression compilation tests
- Arity check tests for variadic functions

---

### Phase 7: Expression Language - Evaluator
**Goal:** Evaluate compiled expressions against RequestContext

**File to create:**
- `/workspace/src/expr/eval.rs` - Expression evaluator

**Value enum:**
```rust
pub enum Value {
    Str(String),
    StrList(Vec<String>),
    Bool(bool),
}
```

**Evaluator implementation:**
- Implement `Program::eval(&self, ctx: &RequestContext) -> Result<bool, EvalError>`
- Recursive evaluation of AST nodes
- Pattern match on `Expr` variants (exhaustive matching enforced by Rust compiler)

**Function implementations:**
- `header(name)` → `ctx.header(name)` (returns empty string if missing)
- `headerValues(name)` → `ctx.header_values(name)` (returns empty vec if missing)
- `headerList(name)` → `ctx.header_list(name)` (split by comma, trim whitespace)
- `contains(list, item)` → `list.contains(item)`
- `anyOf(list, items...)` → `items.iter().any(|item| list.contains(item))`
- `allOf(list, items...)` → `items.iter().all(|item| list.contains(item))`

**Comparison operators:**
- `==`, `!=` → Direct string comparison
- `startsWith`, `endsWith` → String prefix/suffix check
- `contains` (infix) → Substring check
- `matches` → Regex match (compile regex on demand, or cache in Program for string literals)

**Regex handling:**
- Use `regex::Regex::new()` for pattern compilation
- Return `EvalError` on invalid regex pattern
- Note: Could optimize by precompiling string-literal regexes during compilation phase

**Error handling:**
- `EvalError` for runtime errors (primarily regex compilation failures)
- Should be rare since type-checking happens at compile time

**Verification:**
- Unit tests for all built-in functions
- Unit tests for all comparison operators
- Unit test regex matching with valid/invalid patterns
- Integration test: compile and evaluate full expressions against mock RequestContext

---

### Phase 8: HTTP Request Handler and Plugin Entrypoint
**Goal:** Integrate with http-wasm guest library and implement request handling

**Files to modify:**
- `/workspace/src/main.rs` - Complete plugin implementation

**Main implementation:**
```rust
use http_wasm_guest::{Guest, host::{Bytes, Request, Response}, register};

struct AuthzPlugin {
    program: Program,
    config: Config,
}

impl Guest for AuthzPlugin {
    fn handle_request(&self, request: Request, response: Response) -> (bool, i32) {
        // 1. Build RequestContext from request
        let ctx = RequestContext::from_request(&request);

        // 2. Evaluate expression
        match self.program.eval(&ctx) {
            Err(e) => {
                // Fail closed: return 500 on eval error
                log_error(&format!("expression eval error: {e}"));
                response.set_status_code(500);
                response.body().write(&Bytes::from("Internal Server Error"));
                (false, 0)
            }
            Ok(false) => {
                // Deny: return configured status and body
                response.set_status_code(self.config.deny_status_code);
                response.body().write(&Bytes::from(self.config.deny_body.as_str()));
                (false, 0)
            }
            Ok(true) => {
                // Allow: pass to next middleware
                (true, 0)
            }
        }
    }
}

fn main() {
    // 1. Load configuration
    let config_bytes = http_wasm_guest::host::config();
    let config: Config = serde_json::from_slice(&config_bytes)
        .unwrap_or_else(|e| {
            log_error(&format!("invalid config JSON: {e}"));
            std::process::exit(1);
        });

    // 2. Compile expression
    let program = Program::compile(&config.expression)
        .unwrap_or_else(|e| {
            log_error(&format!("invalid expression: {e}"));
            std::process::exit(1);
        });

    // 3. Run test cases
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

    // 4. Register plugin
    register(AuthzPlugin { program, config });
}

fn log_error(msg: &str) {
    eprintln!("[ERROR] {}", msg);
}

fn log_info(msg: &str) {
    eprintln!("[INFO] {}", msg);
}
```

**Verification:**
- Build WASM binary: `cargo build --target wasm32-wasip1 --release`
- Binary size check (should be 50-200 KB after release build)

---

### Phase 9: Plugin Manifest and Documentation
**Goal:** Create Traefik plugin manifest and user documentation

**Files to create/verify:**
- `/workspace/.traefik.yml` - Plugin manifest
- `/workspace/README.md` - Comprehensive documentation

**`.traefik.yml` content:**
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

**README.md sections:**
- Project overview and motivation
- Quick start / installation
- Configuration schema reference
- Expression language reference (operators, functions, examples)
- Test framework usage
- Build instructions
- Example Traefik configurations
- Troubleshooting

**Verification:**
- README includes all expression language features from DESIGN.md Section 4
- Examples match those in DESIGN.md Section 4.7

---

### Phase 10: Build Infrastructure and Examples
**Goal:** Set up build automation and example configurations

**Files to create:**
- `/workspace/Makefile` - Build, test, clean targets
- `/workspace/examples/` directory with sample Traefik configs
- `/workspace/.gitignore` - Rust/WASM ignore patterns

**Makefile targets:**
```makefile
.PHONY: build test clean release

build:
	cargo build --target wasm32-wasip1

release:
	cargo build --target wasm32-wasip1 --release
	cp target/wasm32-wasip1/release/traefik_authz_wasm.wasm plugin.wasm

test:
	cargo test

clean:
	cargo clean
	rm -f plugin.wasm

check:
	cargo clippy --target wasm32-wasip1
	cargo fmt --check
```

**Example configs to create:**
- `/workspace/examples/team-based-access.yml` - Team membership example
- `/workspace/examples/path-restrictions.yml` - Path-based access control
- `/workspace/examples/combined-rules.yml` - Complex boolean logic example

**`.gitignore`:**
```
/target/
**/*.rs.bk
*.pdb
plugin.wasm
Cargo.lock
```

**Verification:**
- `make build` succeeds
- `make test` runs all unit tests
- `make release` produces `plugin.wasm` at project root

---

### Phase 11: Testing and Validation
**Goal:** Comprehensive testing of all components

**Test coverage areas:**

1. **Unit tests (already embedded in phases above):**
   - Lexer tokenization
   - Parser AST construction
   - Type checker validation
   - Evaluator correctness
   - RequestContext header access

2. **Integration tests to add:**
   - End-to-end: config JSON → compiled program → evaluation
   - Test framework: mock test cases validated at startup
   - Error propagation: invalid config/expression handling

3. **Manual testing scenarios:**
   - Build WASM binary with release profile
   - Verify binary size (target: < 200 KB)
   - Test with sample Traefik configuration (if Traefik available)

**Files to create:**
- `/workspace/src/expr/tests.rs` - Integration tests for expression engine
- `/workspace/tests/integration_test.rs` - Full plugin lifecycle tests

**Verification:**
- All tests pass: `cargo test`
- No clippy warnings: `cargo clippy --target wasm32-wasip1`
- Code formatted: `cargo fmt --check`
- Release binary builds: `make release`
- Release binary size acceptable: `ls -lh plugin.wasm`

---

### Phase 12: Docker-Based Traefik Integration Test
**Goal:** Validate plugin in real Traefik environment with end-to-end request flow

**Why this matters:**
- Verifies the plugin works in actual Traefik runtime (not just unit tests)
- Tests http-wasm ABI integration correctly
- Validates request/response handling through the full middleware chain
- Ensures compatibility as Traefik versions evolve
- Provides confidence before publishing to plugin registry

**Files to create:**
- `/workspace/integration-test/docker-compose.yml` - Traefik + test backend
- `/workspace/integration-test/traefik.yml` - Traefik static config with local WASM plugin
- `/workspace/integration-test/dynamic.yml` - Middleware configuration with test expressions
- `/workspace/integration-test/test.sh` - Test script that sends HTTP requests and validates responses
- `/workspace/integration-test/Dockerfile.backend` - Simple test backend service (optional, could use httpbin)

**Docker Compose Setup:**

**Services:**
1. **Traefik** - Load plugin from local `plugin.wasm` (not plugin registry)
2. **Backend** - Simple HTTP service that echoes headers/request info
3. **Test runner** - Container that executes test requests

**Key configuration details:**

**`traefik.yml` (static config):**
```yaml
experimental:
  wasm:
    enabled: true

entryPoints:
  web:
    address: ":80"

providers:
  file:
    directory: /etc/traefik/dynamic
    watch: true

log:
  level: DEBUG

# Load local WASM plugin
experimental:
  plugins:
    authz:
      moduleName: local
      # Point to local .wasm file
```

**How to load local WASM plugin in Traefik:**
- Mount `plugin.wasm` into Traefik container
- Use Traefik's local plugin loading mechanism (not plugin catalog)
- Reference: Traefik docs on local WASM plugin development

**`dynamic.yml` (middleware config with test scenarios):**
```yaml
http:
  routers:
    test-allowed:
      rule: "Host(`allowed.test`)"
      service: backend
      middlewares:
        - team-check

    test-denied:
      rule: "Host(`denied.test`)"
      service: backend
      middlewares:
        - team-check

  middlewares:
    team-check:
      plugin:
        authz:
          expression: 'contains(headerList("X-Auth-User-Teams"), "platform-eng")'
          denyStatusCode: 403
          denyBody: "Access denied: requires platform-eng team"
          tests:
            - name: "platform-eng allowed"
              request:
                headers:
                  X-Auth-User-Teams: "platform-eng,devops"
              expect: true
            - name: "marketing denied"
              request:
                headers:
                  X-Auth-User-Teams: "marketing"
              expect: false

  services:
    backend:
      loadBalancer:
        servers:
          - url: "http://backend:8080"
```

**`test.sh` test scenarios:**

```bash
#!/bin/bash
set -e

echo "=== Integration Test Suite ==="

# Test 1: Request with correct team header should be allowed (200)
echo "Test 1: Authorized request with platform-eng team"
RESPONSE=$(curl -s -w "%{http_code}" -H "Host: allowed.test" \
  -H "X-Auth-User-Teams: platform-eng,devops" \
  http://traefik)
if [[ "$RESPONSE" == *"200"* ]]; then
  echo "✓ Test 1 passed"
else
  echo "✗ Test 1 failed: expected 200, got $RESPONSE"
  exit 1
fi

# Test 2: Request with wrong team header should be denied (403)
echo "Test 2: Unauthorized request with marketing team"
RESPONSE=$(curl -s -w "%{http_code}" -H "Host: denied.test" \
  -H "X-Auth-User-Teams: marketing" \
  http://traefik)
if [[ "$RESPONSE" == *"403"* ]]; then
  echo "✓ Test 2 passed"
else
  echo "✗ Test 2 failed: expected 403, got $RESPONSE"
  exit 1
fi

# Test 3: Request without team header should be denied (403)
echo "Test 3: Request missing team header"
RESPONSE=$(curl -s -w "%{http_code}" -H "Host: denied.test" \
  http://traefik)
if [[ "$RESPONSE" == *"403"* ]]; then
  echo "✓ Test 3 passed"
else
  echo "✗ Test 3 failed: expected 403, got $RESPONSE"
  exit 1
fi

# Test 4: Verify deny body contains expected message
echo "Test 4: Verify deny response body"
BODY=$(curl -s -H "Host: denied.test" -H "X-Auth-User-Teams: marketing" \
  http://traefik)
if [[ "$BODY" == *"requires platform-eng team"* ]]; then
  echo "✓ Test 4 passed"
else
  echo "✗ Test 4 failed: unexpected body: $BODY"
  exit 1
fi

# Test 5: Complex expression with path and method checks
# (Add router with more complex expression)
echo "Test 5: Complex expression (method AND path)"
# ... additional test cases ...

echo "=== All integration tests passed! ==="
```

**`docker-compose.yml`:**
```yaml
version: '3.8'

services:
  traefik:
    image: traefik:v3.0  # Use latest Traefik v3
    container_name: traefik-test
    ports:
      - "80:80"
    volumes:
      - ./traefik.yml:/etc/traefik/traefik.yml:ro
      - ./dynamic.yml:/etc/traefik/dynamic/dynamic.yml:ro
      - ../plugin.wasm:/etc/traefik/plugins/authz.wasm:ro
    networks:
      - test-network

  backend:
    image: hashicorp/http-echo:latest
    container_name: backend-test
    command:
      - -text="Backend response: OK"
      - -listen=:8080
    networks:
      - test-network

networks:
  test-network:
    driver: bridge
```

**Execution flow:**
1. `make release` - Build release WASM binary
2. `cd integration-test && docker-compose up -d` - Start Traefik + backend
3. Wait for Traefik startup (check logs for plugin initialization)
4. `./integration-test/test.sh` - Run test suite
5. Verify all tests pass
6. `docker-compose logs traefik` - Check for startup test results in logs
7. `docker-compose down` - Cleanup

**Verification criteria:**
- Docker Compose starts without errors
- Traefik logs show plugin loaded successfully
- Traefik logs show startup tests passed (from `tests:` config)
- All HTTP test scenarios pass (authorized = 200, unauthorized = 403)
- Deny body matches configured message
- No errors in Traefik logs during request handling

**GitHub Actions integration:**
Add integration test job to `.github/workflows/ci.yml`:
```yaml
  integration-test:
    runs-on: ubuntu-latest
    needs: build-and-test
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust and build
        # ... (same as build job)
      - name: Build release WASM
        run: make release
      - name: Run integration tests
        run: |
          cd integration-test
          docker-compose up -d
          sleep 10  # Wait for Traefik startup
          chmod +x test.sh
          ./test.sh
          docker-compose logs traefik
          docker-compose down
```

**Documentation updates:**
- Add "Integration Testing" section to README.md
- Document how to run integration tests locally
- Document how to debug plugin issues with Traefik logs
- Include example of Traefik log output showing successful plugin load

**Future enhancements (out of scope for v1):**
- Test multiple Traefik versions (v3.0, v3.1, latest)
- Test different expression patterns (regex, path matching, etc.)
- Performance testing (requests/second with plugin active)
- Test plugin behavior with invalid config (should prevent Traefik startup)

---

### Phase 13: Task Tracking Setup
**Goal:** Create task checklist document for tracking implementation progress

**File to create:**
- `/workspace/docs/TASKS.md` - Markdown checklist of all implementation tasks

**Structure:**
- Group tasks by phase (matching this plan)
- Each task as a checkbox `- [ ]` that can be marked complete `- [x]`
- Include verification criteria for each task
- Update after each task completes

**Format:**
```markdown
# Implementation Tasks

## Phase 1: Project Scaffolding
- [ ] Create Cargo.toml with dependencies
- [ ] Create .traefik.yml manifest
- [ ] Create Makefile
- [ ] Create README.md stub
- [ ] Create .gitignore
- [ ] Create src/main.rs stub
- [ ] Verify: cargo build succeeds

## Phase 2: GitHub Actions CI/CD Workflow
- [ ] Create .github/workflows/ci.yml
- [ ] Configure build, test, lint jobs
- [ ] Verify workflow triggers on commit
- [ ] Add workflow badge to README

## Phase 3: Configuration and Data Structures
- [ ] Create src/config.rs with Config, TestCase, TestRequest structs
- [ ] Create src/context.rs with RequestContext
- [ ] Implement case-insensitive header access
- [ ] Add unit tests for header access
- [ ] Add JSON deserialization tests

...

## Phase 12: Docker-Based Traefik Integration Test
- [ ] Create integration-test/docker-compose.yml
- [ ] Create integration-test/traefik.yml
- [ ] Create integration-test/dynamic.yml
- [ ] Create integration-test/test.sh
- [ ] Build release WASM: make release
- [ ] Run docker-compose up and verify Traefik starts
- [ ] Execute test.sh and verify all tests pass
- [ ] Add integration test job to GitHub Actions

## Phase 13: Task Tracking Setup
- [ ] Create docs/TASKS.md with all phases
- [ ] Ensure all tasks have completion criteria
- [ ] Commit TASKS.md to repository
```

**Verification:**
- TASKS.md created and committed
- All phases from this plan represented
- Each task has clear completion criteria

---

## Critical Files

### Files to Create (in order):
1. `/workspace/Cargo.toml`
2. `/workspace/.traefik.yml`
3. `/workspace/Makefile`
4. `/workspace/.gitignore`
5. `/workspace/README.md`
6. `/workspace/src/main.rs`
7. `/workspace/.github/workflows/ci.yml`
8. `/workspace/src/config.rs`
9. `/workspace/src/context.rs`
10. `/workspace/src/expr/mod.rs`
11. `/workspace/src/expr/lexer.rs`
12. `/workspace/src/expr/ast.rs`
13. `/workspace/src/expr/parser.rs`
14. `/workspace/src/expr/compiler.rs`
15. `/workspace/src/expr/eval.rs`
16. `/workspace/src/expr/tests.rs`
17. `/workspace/tests/integration_test.rs`
18. `/workspace/examples/team-based-access.yml`
19. `/workspace/examples/path-restrictions.yml`
20. `/workspace/examples/combined-rules.yml`
21. `/workspace/integration-test/docker-compose.yml`
22. `/workspace/integration-test/traefik.yml`
23. `/workspace/integration-test/dynamic.yml`
24. `/workspace/integration-test/test.sh`
25. `/workspace/docs/TASKS.md`

### Existing Files (Reference Only):
- `/workspace/docs/DESIGN.md` - Design specification (DO NOT MODIFY)

---

## Dependencies and Prerequisites

**System Requirements:**
- Rust toolchain (stable) with `rustup`
- `wasm32-wasip1` target installed: `rustup target add wasm32-wasip1`

**Rust Dependencies (Cargo.toml):**
- `http-wasm-guest = "0.7.0"` - http-wasm guest library for Traefik
- `serde = { version = "1.0", features = ["derive"] }` - Serialization framework
- `serde_json = "1.0"` - JSON support
- `regex = "1.10"` - RE2-compatible regex engine

---

## Testing Strategy

### Unit Tests
- Each module (`lexer`, `parser`, `compiler`, `eval`) includes comprehensive unit tests
- Test both success and error cases
- Focus on edge cases: empty strings, missing headers, invalid regex

### Integration Tests
- Full expression compilation and evaluation pipeline
- Config JSON parsing with test cases
- Test framework validation (startup-time test execution)

### Manual Validation
- Build release binary: `make release`
- Check binary size: `ls -lh plugin.wasm` (should be 50-200 KB)
- Optionally test with Traefik if available locally

### Success Criteria
- All `cargo test` pass
- No `cargo clippy` warnings
- Code passes `cargo fmt --check`
- Release binary builds successfully
- Binary size within expected range

---

## Implementation Notes

### Expression Language Design Choices

1. **Case-insensitive headers:** All header name lookups are lowercased for HTTP/1.1 and HTTP/2 compatibility

2. **Fail-closed security:** On any unexpected evaluation error, return HTTP 500 (not pass-through)

3. **Startup-time validation:** Expression compilation and test execution happen during plugin initialization - Traefik will refuse to start if the configuration is invalid

4. **RE2 regex:** The `regex` crate provides linear-time matching guarantees, preventing ReDoS attacks

5. **Type safety:** Type checking at compile time (not runtime) catches errors before any requests are processed

### Rust-Specific Implementation Details

1. **Enum exhaustiveness:** Pattern matching on `Expr`, `Token`, `BinOp` enums is checked at compile time

2. **Error handling:** Use `Result<T, E>` with custom error types (`LexError`, `ParseError`, `CompileError`, `EvalError`)

3. **Zero-copy where possible:** Use `&str` for string slices, avoid unnecessary allocations

4. **HashMap for headers:** Case-insensitive lookup by lowercasing keys during RequestContext construction

### Performance Considerations

1. **Compile once:** Expression compiled at startup, reused for all requests
2. **Lazy header access:** Headers read from http-wasm ABI only when referenced by expression
3. **Regex caching:** String-literal regex patterns could be precompiled during compilation phase (future optimization)
4. **Small binary:** Target < 200 KB for fast plugin loading

---

## Execution Approach

After plan approval, implementation will proceed using specialized subagents:

1. **General-purpose agents** for file creation and code implementation
2. **Bash agents** for build verification and testing
3. **Each phase as a separate task** to enable incremental progress and validation

Progress will be tracked in `/workspace/docs/TASKS.md` with checkboxes updated after each task completion.

---

## Open Questions (to be resolved during implementation)

1. **Regex compilation strategy:** Compile regex at evaluation time or precompile string literals during compilation phase? (Recommend: precompile for performance)

2. **Header value encoding:** How to handle non-UTF8 header values? (Recommend: lossy UTF-8 conversion, log warning)

3. **Maximum expression complexity:** Should we limit AST depth to prevent stack overflow? (Recommend: reasonable limit like 100 nodes deep)

4. **Logging verbosity:** How much detail to log at startup vs per-request? (Recommend: verbose at startup for debugging, minimal per-request for performance)

These can be addressed with sensible defaults during implementation and documented for users to configure if needed.
