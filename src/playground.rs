// Copyright (c) 2025 Andrew Kroh
// SPDX-License-Identifier: MIT

// Browser playground bridge via wasm-bindgen
//
// Exposes expression compilation and evaluation to JavaScript
// using the same code path as Traefik's startup test validation.

use crate::config::TestRequest;
use crate::context::RequestContext;
use crate::expr::compiler::Program;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Compile an expression and return JSON result.
/// Returns {"ok": true} on success or {"error": "..."} on failure.
#[wasm_bindgen]
pub fn playground_compile(expression: &str) -> String {
    match Program::compile(expression) {
        Ok(_) => r#"{"ok":true}"#.to_string(),
        Err(e) => {
            let msg = e.message.replace('\\', "\\\\").replace('"', "\\\"");
            format!(r#"{{"error":"{}"}}"#, msg)
        }
    }
}

/// Evaluate an expression against a mock request.
/// Input JSON: {"expression": "...", "request": {"method": "GET", "path": "/...", "host": "...", "headers": {...}}}
/// Returns {"result": true/false} or {"error": "..."}.
#[wasm_bindgen]
pub fn playground_eval(input_json: &str) -> String {
    let input: EvalInput = match serde_json::from_str(input_json) {
        Ok(v) => v,
        Err(e) => {
            return format!(
                r#"{{"error":"Invalid input JSON: {}"}}"#,
                escape(&e.to_string())
            )
        }
    };

    let program = match Program::compile(&input.expression) {
        Ok(p) => p,
        Err(e) => return format!(r#"{{"error":"{}"}}"#, escape(&e.message)),
    };

    let test_req = TestRequest {
        method: input.request.method,
        path: input.request.path,
        host: input.request.host,
        headers: input.request.headers.unwrap_or_default(),
    };

    let ctx = RequestContext::from_test(&test_req);

    match program.eval(&ctx) {
        Ok(result) => format!(r#"{{"result":{}}}"#, result),
        Err(e) => format!(r#"{{"error":"{}"}}"#, escape(&e.message)),
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[derive(serde::Deserialize)]
struct EvalInput {
    expression: String,
    request: EvalRequest,
}

#[derive(serde::Deserialize)]
struct EvalRequest {
    #[serde(default)]
    method: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    host: String,
    #[serde(default)]
    headers: Option<HashMap<String, String>>,
}
