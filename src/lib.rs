// Copyright (c) 2025 Andrew Kroh
// SPDX-License-Identifier: MIT

// Traefik WASM Authorization Middleware Plugin
//
// This plugin performs attribute-based authorization on HTTP requests
// by evaluating expressions against request attributes.

pub mod config;
pub mod context;
pub mod expr;

#[cfg(feature = "playground")]
pub mod playground;

#[cfg(all(target_arch = "wasm32", feature = "traefik-plugin"))]
mod plugin {
    use crate::config::Config;
    use crate::context::RequestContext;
    use crate::expr::compiler::Program;
    use http_wasm_guest::{host, Guest, Request, Response};

    /// Authorization plugin implementation
    pub struct AuthzPlugin {
        program: Program,
        config: Config,
    }

    impl Guest for AuthzPlugin {
        fn handle_request(&self, request: Request, response: Response) -> (bool, i32) {
            // Build RequestContext from http-wasm Request
            let ctx = RequestContext::from_request(&request);

            // Evaluate expression
            match self.program.eval(&ctx) {
                Err(e) => {
                    // Fail closed: return 500 on eval error
                    log_error(&format!("Expression evaluation error: {}", e));
                    response.set_status(500);
                    response.body().write(b"Internal Server Error");
                    (false, 0)
                }
                Ok(false) => {
                    // Deny: return configured status and body
                    response.set_status(self.config.deny_status_code as i32);
                    response.body().write(self.config.deny_body.as_bytes());
                    (false, 0)
                }
                Ok(true) => {
                    // Allow: pass to next middleware
                    (true, 0)
                }
            }
        }
    }

    /// Plugin initialization
    #[no_mangle]
    pub extern "C" fn _start() {
        // 1. Load configuration
        let config_bytes = host::config();
        let config: Config = serde_json::from_slice(&config_bytes).unwrap_or_else(|e| {
            log_error(&format!("Invalid config JSON: {}", e));
            std::process::abort();
        });

        // 2. Compile expression
        let program = Program::compile(&config.expression).unwrap_or_else(|e| {
            log_error(&format!("Invalid expression: {}", e));
            std::process::abort();
        });

        log_info(&format!(
            "Expression compiled successfully: {}",
            config.expression
        ));

        // 3. Run test cases
        for tc in &config.tests {
            let ctx = RequestContext::from_test(&tc.request);
            match program.eval(&ctx) {
                Err(e) => {
                    log_error(&format!("Test '{}' evaluation error: {}", tc.name, e));
                    std::process::abort();
                }
                Ok(result) if result != tc.expect => {
                    log_error(&format!(
                        "Test '{}' failed: got {}, expected {}",
                        tc.name, result, tc.expect
                    ));
                    std::process::abort();
                }
                Ok(_) => {
                    log_info(&format!("Test '{}' passed", tc.name));
                }
            }
        }

        log_info(&format!("All {} test(s) passed", config.tests.len()));

        // 4. Register plugin
        http_wasm_guest::register(AuthzPlugin { program, config });
    }

    fn log_error(msg: &str) {
        eprintln!("[traefik-authz-wasm ERROR] {}", msg);
    }

    fn log_info(msg: &str) {
        eprintln!("[traefik-authz-wasm INFO] {}", msg);
    }
}
