# Implementation Tasks

This document tracks the implementation progress of the Traefik WASM Authorization Middleware Plugin.

## Phase 1: Project Scaffolding ✅
- [x] Create Cargo.toml with dependencies (http-wasm-guest, serde, serde_json, regex)
- [x] Create .traefik.yml plugin manifest
- [x] Create Makefile with build, test, clean, release targets
- [x] Create README.md with project overview
- [x] Create .gitignore for Rust/WASM projects
- [x] Create src/lib.rs with minimal stub (renamed from main.rs)
- [x] Verify: `cargo build --target wasm32-wasip1` succeeds
- [x] Commit Phase 1 changes

## Phase 2: GitHub Actions CI/CD Workflow ✅
- [x] Create .github/workflows/ci.yml
- [x] Configure build-and-test job (checkout, install Rust, cache, fmt, clippy, test, build)
- [x] Configure workflow triggers (push to main, PRs, manual dispatch)
- [x] Add binary size reporting
- [x] Verify workflow is valid YAML
- [x] Commit and verify workflow triggers successfully
- [x] Add workflow badge to README.md

## Phase 3: Configuration and Data Structures ✅
- [x] Create src/config.rs
- [x] Implement Config struct (expression, deny_status_code, deny_body, tests)
- [x] Implement TestCase struct (name, request, expect)
- [x] Implement TestRequest struct (method, path, host, headers)
- [x] Add serde derives and default values
- [x] Create src/context.rs
- [x] Implement RequestContext struct with method, path, host fields
- [x] Implement headers storage (case-insensitive HashMap)
- [x] Implement from_request() method (http-wasm Request) - stub for now
- [x] Implement from_test() method (TestRequest)
- [x] Implement header() method (get first value, case-insensitive)
- [x] Implement header_values() method (get all values)
- [x] Implement header_list() method (comma-separated parsing)
- [x] Add unit tests for RequestContext::from_test()
- [x] Add unit tests for case-insensitive header access
- [x] Add unit tests for Config JSON deserialization
- [x] Verify: All tests pass (10 tests passing)
- [x] Commit Phase 3 changes

## Phase 4: Expression Language - Lexer ✅
- [x] Create src/expr/mod.rs module declaration
- [x] Create src/expr/lexer.rs
- [x] Define Token enum (String, Ident, LParen, RParen, Comma, operators, keywords, Eof)
- [x] Define LexError struct with position and message
- [x] Implement Lexer struct (input, pos, current_char)
- [x] Implement Lexer::new(input)
- [x] Implement advance() helper
- [x] Implement peek() helper
- [x] Implement skip_whitespace() helper
- [x] Implement read_string() helper (handle escaping)
- [x] Implement read_ident_or_keyword() helper
- [x] Implement next_token() method
- [x] Add keyword recognition (AND, OR, NOT, startsWith, endsWith, contains, matches)
- [x] Add unit test: tokenize `method == "GET"`
- [x] Add unit test: string literals with escaping
- [x] Add unit test: all operators (==, !=, startsWith, endsWith, contains, matches)
- [x] Add unit test: all keywords (AND, OR, NOT)
- [x] Add unit test: error cases (unterminated strings, invalid chars)
- [x] Verify: All tests pass (19 tests passing)
- [x] Commit Phase 4 changes

## Phase 5: Expression Language - Parser and AST ✅
- [x] Create src/expr/ast.rs
- [x] Define Expr enum (BoolLiteral, StringLiteral, Ident, FuncCall, BinaryOp, Not, And, Or)
- [x] Define Ident enum (Method, Path, Host)
- [x] Define BinOp enum (Eq, Neq, StartsWith, EndsWith, Contains, Matches)
- [x] Create src/expr/parser.rs
- [x] Define ParseError struct with position and message
- [x] Implement Parser struct (lexer, current_token, peek_token)
- [x] Implement Parser::new() and advance() helper
- [x] Implement parse() entry point
- [x] Implement parse_expr() → parse_or_expr()
- [x] Implement parse_or_expr() (left-associative OR)
- [x] Implement parse_and_expr() (left-associative AND)
- [x] Implement parse_not_expr() (NOT prefix)
- [x] Implement parse_comparison() (binary operators + function-style syntax)
- [x] Implement parse_value() (string, func_call, ident, parentheses)
- [x] Implement parse_func_call() (ident, args)
- [x] Add unit test: parse `method == "GET"`
- [x] Add unit test: parse `contains(headerList("X-Auth-User-Teams"), "platform-eng")`
- [x] Add unit test: parse `path startsWith "/api" AND method == "GET"`
- [x] Add unit test: complex nested expressions
- [x] Add unit test: error cases (unclosed paren, invalid syntax)
- [x] Verify: All tests pass (28 tests passing)
- [x] Commit Phase 5 changes

## Phase 6: Expression Language - Type Checker and Compiler ✅
- [x] Create src/expr/compiler.rs
- [x] Define Type enum (Str, StrList, Bool)
- [x] Define CompileError struct with position and type mismatch details
- [x] Define Program struct (root: Expr)
- [x] Implement compile(input) entry point
- [x] Implement type_check() recursive function
- [x] Add type checking for identifiers (method/path/host → Str)
- [x] Add type checking for binary operators (all require Str, Str → Bool)
- [x] Add type checking for boolean operators (AND/OR require Bool → Bool)
- [x] Add type checking for NOT (requires Bool → Bool)
- [x] Add type checking for header(string) → string
- [x] Add type checking for headerValues(string) → []string
- [x] Add type checking for headerList(string) → []string
- [x] Add type checking for contains([]string, string) → bool
- [x] Add type checking for anyOf([]string, ...string) → bool (variadic)
- [x] Add type checking for allOf([]string, ...string) → bool (variadic)
- [x] Verify top-level expression is bool
- [x] Add unit test: type error `method AND path`
- [x] Add unit test: type error `contains("foo", "bar")`
- [x] Add unit test: valid expression compilation
- [x] Add unit test: arity checks for variadic functions
- [x] Verify: All tests pass (38 tests passing)
- [x] Commit Phase 6 changes

## Phase 7: Expression Language - Evaluator ✅
- [x] Create src/expr/eval.rs
- [x] Define Value enum (Str, StrList, Bool)
- [x] Define EvalError struct
- [x] Implement Program::eval(&self, ctx: &RequestContext) → Result<bool, EvalError>
- [x] Implement eval_expr() recursive function
- [x] Add evaluation for Ident::Method → ctx.method
- [x] Add evaluation for Ident::Path → ctx.path
- [x] Add evaluation for Ident::Host → ctx.host
- [x] Add evaluation for BinOp::Eq (string equality)
- [x] Add evaluation for BinOp::Neq (string inequality)
- [x] Add evaluation for BinOp::StartsWith
- [x] Add evaluation for BinOp::EndsWith
- [x] Add evaluation for BinOp::Contains (substring)
- [x] Add evaluation for BinOp::Matches (regex - compile on demand)
- [x] Add evaluation for And/Or/Not boolean operators
- [x] Implement header(name) function
- [x] Implement headerValues(name) function
- [x] Implement headerList(name) function (comma-separated split)
- [x] Implement contains(list, item) function
- [x] Implement anyOf(list, ...items) function
- [x] Implement allOf(list, ...items) function
- [x] Add unit test: all built-in functions
- [x] Add unit test: all comparison operators
- [x] Add unit test: regex matching (valid and invalid patterns)
- [x] Add integration test: compile + eval full expressions
- [x] Verify: All tests pass
- [x] Commit Phase 7 changes

## Phase 8: HTTP Request Handler and Plugin Entrypoint ✅
- [x] Update src/main.rs with complete implementation
- [x] Define AuthzPlugin struct (program, config)
- [x] Implement Guest trait for AuthzPlugin
- [x] Implement handle_request() method
- [x] Build RequestContext from http-wasm Request
- [x] Evaluate expression via program.eval()
- [x] Handle eval errors (return 500, fail closed)
- [x] Handle deny case (return configured status + body)
- [x] Handle allow case (pass to next middleware)
- [x] Implement main() function
- [x] Load configuration from http_wasm_guest::host::config()
- [x] Parse JSON config (exit on error)
- [x] Compile expression (exit on error)
- [x] Run all test cases at startup
- [x] Validate test results (exit on failure)
- [x] Register plugin with http-wasm
- [x] Add log_error() and log_info() helpers
- [x] Verify: `cargo build --target wasm32-wasip1 --release` succeeds
- [x] Verify: Binary size is 50-200 KB
- [x] Commit Phase 8 changes

## Phase 9: Plugin Manifest and Documentation ✅
- [x] Update .traefik.yml with complete manifest
- [x] Add displayName, type, runtime fields
- [x] Add summary description
- [x] Add testData with sample expression and tests
- [x] Create comprehensive README.md
- [x] Add project overview and motivation
- [x] Add quick start / installation instructions
- [x] Add configuration schema reference
- [x] Add expression language reference (operators)
- [x] Add built-in functions reference
- [x] Add expression examples
- [x] Add test framework usage documentation
- [x] Add build instructions
- [x] Add example Traefik configurations
- [x] Add troubleshooting section
- [x] Verify: README includes all DESIGN.md Section 4 features
- [x] Verify: Examples match DESIGN.md Section 4.7
- [x] Commit Phase 9 changes

## Phase 10: Build Infrastructure and Examples ✅
- [x] Update Makefile with all targets
- [x] Add build target (debug build)
- [x] Add release target (release build + copy to plugin.wasm)
- [x] Add test target (cargo test)
- [x] Add clean target (cargo clean + rm plugin.wasm)
- [x] Add check target (clippy + fmt --check)
- [x] Create examples/ directory
- [x] Create examples/team-based-access.yml
- [x] Create examples/path-restrictions.yml
- [x] Create examples/combined-rules.yml
- [x] Update .gitignore with Rust patterns
- [x] Add /target/ to .gitignore
- [x] Add *.rs.bk to .gitignore
- [x] Add plugin.wasm to .gitignore
- [x] Add Cargo.lock to .gitignore
- [x] Verify: `make build` succeeds
- [x] Verify: `make test` runs all tests
- [x] Verify: `make release` produces plugin.wasm
- [x] Commit Phase 10 changes

## Phase 11: Testing and Validation ✅
- [x] Create src/expr/tests.rs for integration tests
- [x] Add integration tests for full expression pipeline
- [x] Create tests/integration_test.rs
- [x] Add tests for config parsing with test cases
- [x] Add tests for test framework validation
- [x] Run all unit tests: `cargo test`
- [x] Run clippy: `cargo clippy --target wasm32-wasip1`
- [x] Run formatter check: `cargo fmt --check`
- [x] Build release: `make release`
- [x] Verify binary size: `ls -lh plugin.wasm` (should be < 200 KB)
- [x] Verify: No clippy warnings
- [x] Verify: Code is formatted
- [x] Verify: All tests pass
- [x] Commit Phase 11 changes

## Phase 12: Docker-Based Traefik Integration Test ✅
- [x] Create integration-test/ directory
- [x] Create integration-test/docker-compose.yml
- [x] Configure Traefik service (v3.0, mount plugin.wasm)
- [x] Configure backend service (http-echo)
- [x] Configure test network
- [x] Create integration-test/traefik.yml (static config)
- [x] Enable experimental WASM support
- [x] Configure entrypoints, providers, logging
- [x] Configure local plugin loading
- [x] Create integration-test/dynamic.yml (middleware config)
- [x] Add test routers (allowed.test, denied.test)
- [x] Add team-check middleware with expression
- [x] Add startup test cases in config
- [x] Configure backend service
- [x] Create integration-test/test.sh
- [x] Add Test 1: Authorized request (expect 200)
- [x] Add Test 2: Unauthorized request (expect 403)
- [x] Add Test 3: Missing header (expect 403)
- [x] Add Test 4: Verify deny body message
- [x] Add Test 5: Complex expression test
- [x] Make test.sh executable
- [x] Build release: `make release`
- [x] Start services: `cd integration-test && docker-compose up -d`
- [x] Wait for Traefik startup
- [x] Run tests: `./integration-test/test.sh`
- [x] Verify: All tests pass
- [x] Check Traefik logs: `docker-compose logs traefik`
- [x] Verify: Plugin loaded successfully in logs
- [x] Verify: Startup tests passed in logs
- [x] Cleanup: `docker-compose down`
- [x] Update .github/workflows/ci.yml with integration test job
- [x] Add integration-test job (depends on build-and-test)
- [x] Configure job to build, start docker-compose, run tests
- [x] Update README.md with integration testing section
- [x] Document how to run integration tests locally
- [x] Document how to debug with Traefik logs
- [x] Commit Phase 12 changes

## Phase 13: Task Tracking Setup
- [x] Create docs/TASKS.md with all phases
- [x] Include all tasks from implementation plan
- [x] Add verification criteria for each task
- [x] Commit TASKS.md to repository

---

## Progress Summary
- **Total Phases:** 13
- **Completed Phases:** 12
- **Current Phase:** Complete! All phases finished
- **Overall Progress:** 13/13 phases complete (100%)

---

## Notes
- Update checkboxes as tasks complete: `- [x]` for done
- Commit after each major phase completion
- Verify CI passes before moving to next phase
- Update progress summary after each phase
