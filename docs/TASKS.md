# Implementation Tasks

This document tracks the implementation progress of the Traefik WASM Authorization Middleware Plugin.

## Phase 1: Project Scaffolding
- [ ] Create Cargo.toml with dependencies (http-wasm-guest, serde, serde_json, regex)
- [ ] Create .traefik.yml plugin manifest
- [ ] Create Makefile with build, test, clean, release targets
- [ ] Create README.md with project overview
- [ ] Create .gitignore for Rust/WASM projects
- [ ] Create src/main.rs with minimal stub
- [ ] Verify: `cargo build --target wasm32-wasip1` succeeds
- [ ] Commit Phase 1 changes

## Phase 2: GitHub Actions CI/CD Workflow
- [ ] Create .github/workflows/ci.yml
- [ ] Configure build-and-test job (checkout, install Rust, cache, fmt, clippy, test, build)
- [ ] Configure workflow triggers (push to main, PRs, manual dispatch)
- [ ] Add binary size reporting
- [ ] Verify workflow is valid YAML
- [ ] Commit and verify workflow triggers successfully
- [ ] Add workflow badge to README.md

## Phase 3: Configuration and Data Structures
- [ ] Create src/config.rs
- [ ] Implement Config struct (expression, deny_status_code, deny_body, tests)
- [ ] Implement TestCase struct (name, request, expect)
- [ ] Implement TestRequest struct (method, path, host, headers)
- [ ] Add serde derives and default values
- [ ] Create src/context.rs
- [ ] Implement RequestContext struct with method, path, host fields
- [ ] Implement headers storage (case-insensitive HashMap)
- [ ] Implement from_request() method (http-wasm Request)
- [ ] Implement from_test() method (TestRequest)
- [ ] Implement header() method (get first value, case-insensitive)
- [ ] Implement header_values() method (get all values)
- [ ] Implement header_list() method (comma-separated parsing)
- [ ] Add unit tests for RequestContext::from_test()
- [ ] Add unit tests for case-insensitive header access
- [ ] Add unit tests for Config JSON deserialization
- [ ] Verify: All tests pass
- [ ] Commit Phase 3 changes

## Phase 4: Expression Language - Lexer
- [ ] Create src/expr/mod.rs module declaration
- [ ] Create src/expr/lexer.rs
- [ ] Define Token enum (String, Ident, LParen, RParen, Comma, operators, keywords, Eof)
- [ ] Define LexError struct with position and message
- [ ] Implement Lexer struct (input, pos, current_char)
- [ ] Implement Lexer::new(input)
- [ ] Implement advance() helper
- [ ] Implement peek() helper
- [ ] Implement skip_whitespace() helper
- [ ] Implement read_string() helper (handle escaping)
- [ ] Implement read_ident_or_keyword() helper
- [ ] Implement next_token() method
- [ ] Add keyword recognition (AND, OR, NOT, startsWith, endsWith, contains, matches)
- [ ] Add unit test: tokenize `method == "GET"`
- [ ] Add unit test: string literals with escaping
- [ ] Add unit test: all operators (==, !=, startsWith, endsWith, contains, matches)
- [ ] Add unit test: all keywords (AND, OR, NOT)
- [ ] Add unit test: error cases (unterminated strings, invalid chars)
- [ ] Verify: All tests pass
- [ ] Commit Phase 4 changes

## Phase 5: Expression Language - Parser and AST
- [ ] Create src/expr/ast.rs
- [ ] Define Expr enum (BoolLiteral, StringLiteral, Ident, FuncCall, BinaryOp, Not, And, Or)
- [ ] Define Ident enum (Method, Path, Host)
- [ ] Define BinOp enum (Eq, Neq, StartsWith, EndsWith, Contains, Matches)
- [ ] Create src/expr/parser.rs
- [ ] Define ParseError struct with position and message
- [ ] Implement Parser struct (lexer, current_token, peek_token)
- [ ] Implement Parser::new() and advance() helper
- [ ] Implement parse() entry point
- [ ] Implement parse_expr() → parse_or_expr()
- [ ] Implement parse_or_expr() (left-associative OR)
- [ ] Implement parse_and_expr() (left-associative AND)
- [ ] Implement parse_not_expr() (NOT prefix)
- [ ] Implement parse_comparison() (binary operators)
- [ ] Implement parse_value() (string, func_call, ident, parentheses)
- [ ] Implement parse_func_call() (ident, args)
- [ ] Add unit test: parse `method == "GET"`
- [ ] Add unit test: parse `contains(headerList("X-Auth-User-Teams"), "platform-eng")`
- [ ] Add unit test: parse `path startsWith "/api" AND method == "GET"`
- [ ] Add unit test: complex nested expressions
- [ ] Add unit test: error cases (unclosed paren, invalid syntax)
- [ ] Verify: All tests pass
- [ ] Commit Phase 5 changes

## Phase 6: Expression Language - Type Checker and Compiler
- [ ] Create src/expr/compiler.rs
- [ ] Define Type enum (Str, StrList, Bool)
- [ ] Define CompileError struct with position and type mismatch details
- [ ] Define Program struct (root: Expr)
- [ ] Implement compile(input) entry point
- [ ] Implement type_check() recursive function
- [ ] Add type checking for identifiers (method/path/host → Str)
- [ ] Add type checking for binary operators (all require Str, Str → Bool)
- [ ] Add type checking for boolean operators (AND/OR require Bool → Bool)
- [ ] Add type checking for NOT (requires Bool → Bool)
- [ ] Add type checking for header(string) → string
- [ ] Add type checking for headerValues(string) → []string
- [ ] Add type checking for headerList(string) → []string
- [ ] Add type checking for contains([]string, string) → bool
- [ ] Add type checking for anyOf([]string, ...string) → bool (variadic)
- [ ] Add type checking for allOf([]string, ...string) → bool (variadic)
- [ ] Verify top-level expression is bool
- [ ] Add unit test: type error `method AND path`
- [ ] Add unit test: type error `contains("foo", "bar")`
- [ ] Add unit test: valid expression compilation
- [ ] Add unit test: arity checks for variadic functions
- [ ] Verify: All tests pass
- [ ] Commit Phase 6 changes

## Phase 7: Expression Language - Evaluator
- [ ] Create src/expr/eval.rs
- [ ] Define Value enum (Str, StrList, Bool)
- [ ] Define EvalError struct
- [ ] Implement Program::eval(&self, ctx: &RequestContext) → Result<bool, EvalError>
- [ ] Implement eval_expr() recursive function
- [ ] Add evaluation for Ident::Method → ctx.method
- [ ] Add evaluation for Ident::Path → ctx.path
- [ ] Add evaluation for Ident::Host → ctx.host
- [ ] Add evaluation for BinOp::Eq (string equality)
- [ ] Add evaluation for BinOp::Neq (string inequality)
- [ ] Add evaluation for BinOp::StartsWith
- [ ] Add evaluation for BinOp::EndsWith
- [ ] Add evaluation for BinOp::Contains (substring)
- [ ] Add evaluation for BinOp::Matches (regex - compile on demand)
- [ ] Add evaluation for And/Or/Not boolean operators
- [ ] Implement header(name) function
- [ ] Implement headerValues(name) function
- [ ] Implement headerList(name) function (comma-separated split)
- [ ] Implement contains(list, item) function
- [ ] Implement anyOf(list, ...items) function
- [ ] Implement allOf(list, ...items) function
- [ ] Add unit test: all built-in functions
- [ ] Add unit test: all comparison operators
- [ ] Add unit test: regex matching (valid and invalid patterns)
- [ ] Add integration test: compile + eval full expressions
- [ ] Verify: All tests pass
- [ ] Commit Phase 7 changes

## Phase 8: HTTP Request Handler and Plugin Entrypoint
- [ ] Update src/main.rs with complete implementation
- [ ] Define AuthzPlugin struct (program, config)
- [ ] Implement Guest trait for AuthzPlugin
- [ ] Implement handle_request() method
- [ ] Build RequestContext from http-wasm Request
- [ ] Evaluate expression via program.eval()
- [ ] Handle eval errors (return 500, fail closed)
- [ ] Handle deny case (return configured status + body)
- [ ] Handle allow case (pass to next middleware)
- [ ] Implement main() function
- [ ] Load configuration from http_wasm_guest::host::config()
- [ ] Parse JSON config (exit on error)
- [ ] Compile expression (exit on error)
- [ ] Run all test cases at startup
- [ ] Validate test results (exit on failure)
- [ ] Register plugin with http-wasm
- [ ] Add log_error() and log_info() helpers
- [ ] Verify: `cargo build --target wasm32-wasip1 --release` succeeds
- [ ] Verify: Binary size is 50-200 KB
- [ ] Commit Phase 8 changes

## Phase 9: Plugin Manifest and Documentation
- [ ] Update .traefik.yml with complete manifest
- [ ] Add displayName, type, runtime fields
- [ ] Add summary description
- [ ] Add testData with sample expression and tests
- [ ] Create comprehensive README.md
- [ ] Add project overview and motivation
- [ ] Add quick start / installation instructions
- [ ] Add configuration schema reference
- [ ] Add expression language reference (operators)
- [ ] Add built-in functions reference
- [ ] Add expression examples
- [ ] Add test framework usage documentation
- [ ] Add build instructions
- [ ] Add example Traefik configurations
- [ ] Add troubleshooting section
- [ ] Verify: README includes all DESIGN.md Section 4 features
- [ ] Verify: Examples match DESIGN.md Section 4.7
- [ ] Commit Phase 9 changes

## Phase 10: Build Infrastructure and Examples
- [ ] Update Makefile with all targets
- [ ] Add build target (debug build)
- [ ] Add release target (release build + copy to plugin.wasm)
- [ ] Add test target (cargo test)
- [ ] Add clean target (cargo clean + rm plugin.wasm)
- [ ] Add check target (clippy + fmt --check)
- [ ] Create examples/ directory
- [ ] Create examples/team-based-access.yml
- [ ] Create examples/path-restrictions.yml
- [ ] Create examples/combined-rules.yml
- [ ] Update .gitignore with Rust patterns
- [ ] Add /target/ to .gitignore
- [ ] Add *.rs.bk to .gitignore
- [ ] Add plugin.wasm to .gitignore
- [ ] Add Cargo.lock to .gitignore
- [ ] Verify: `make build` succeeds
- [ ] Verify: `make test` runs all tests
- [ ] Verify: `make release` produces plugin.wasm
- [ ] Commit Phase 10 changes

## Phase 11: Testing and Validation
- [ ] Create src/expr/tests.rs for integration tests
- [ ] Add integration tests for full expression pipeline
- [ ] Create tests/integration_test.rs
- [ ] Add tests for config parsing with test cases
- [ ] Add tests for test framework validation
- [ ] Run all unit tests: `cargo test`
- [ ] Run clippy: `cargo clippy --target wasm32-wasip1`
- [ ] Run formatter check: `cargo fmt --check`
- [ ] Build release: `make release`
- [ ] Verify binary size: `ls -lh plugin.wasm` (should be < 200 KB)
- [ ] Verify: No clippy warnings
- [ ] Verify: Code is formatted
- [ ] Verify: All tests pass
- [ ] Commit Phase 11 changes

## Phase 12: Docker-Based Traefik Integration Test
- [ ] Create integration-test/ directory
- [ ] Create integration-test/docker-compose.yml
- [ ] Configure Traefik service (v3.0, mount plugin.wasm)
- [ ] Configure backend service (http-echo)
- [ ] Configure test network
- [ ] Create integration-test/traefik.yml (static config)
- [ ] Enable experimental WASM support
- [ ] Configure entrypoints, providers, logging
- [ ] Configure local plugin loading
- [ ] Create integration-test/dynamic.yml (middleware config)
- [ ] Add test routers (allowed.test, denied.test)
- [ ] Add team-check middleware with expression
- [ ] Add startup test cases in config
- [ ] Configure backend service
- [ ] Create integration-test/test.sh
- [ ] Add Test 1: Authorized request (expect 200)
- [ ] Add Test 2: Unauthorized request (expect 403)
- [ ] Add Test 3: Missing header (expect 403)
- [ ] Add Test 4: Verify deny body message
- [ ] Add Test 5: Complex expression test
- [ ] Make test.sh executable
- [ ] Build release: `make release`
- [ ] Start services: `cd integration-test && docker-compose up -d`
- [ ] Wait for Traefik startup
- [ ] Run tests: `./integration-test/test.sh`
- [ ] Verify: All tests pass
- [ ] Check Traefik logs: `docker-compose logs traefik`
- [ ] Verify: Plugin loaded successfully in logs
- [ ] Verify: Startup tests passed in logs
- [ ] Cleanup: `docker-compose down`
- [ ] Update .github/workflows/ci.yml with integration test job
- [ ] Add integration-test job (depends on build-and-test)
- [ ] Configure job to build, start docker-compose, run tests
- [ ] Update README.md with integration testing section
- [ ] Document how to run integration tests locally
- [ ] Document how to debug with Traefik logs
- [ ] Commit Phase 12 changes

## Phase 13: Task Tracking Setup
- [x] Create docs/TASKS.md with all phases
- [x] Include all tasks from implementation plan
- [x] Add verification criteria for each task
- [x] Commit TASKS.md to repository

---

## Progress Summary
- **Total Phases:** 13
- **Completed Phases:** 0
- **Current Phase:** Phase 1 - Project Scaffolding
- **Overall Progress:** 1/13 phases complete (7.7%)

---

## Notes
- Update checkboxes as tasks complete: `- [x]` for done
- Commit after each major phase completion
- Verify CI passes before moving to next phase
- Update progress summary after each phase
