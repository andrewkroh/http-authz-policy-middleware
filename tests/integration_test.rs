// Integration tests for the full plugin lifecycle
// Note: These tests use the library crate which is available for testing

use traefik_authz_wasm::config::{Config, TestRequest};
use traefik_authz_wasm::context::RequestContext;
use traefik_authz_wasm::expr::compiler::Program;

#[test]
fn test_full_pipeline_simple() {
    let config_json = r#"{
        "expression": "method == \"GET\"",
        "denyStatusCode": 403,
        "denyBody": "Forbidden"
    }"#;

    let config: Config = serde_json::from_str(config_json).unwrap();
    let program = Program::compile(&config.expression).unwrap();

    // Test GET request
    let ctx = RequestContext::from_test(&TestRequest {
        method: "GET".to_string(),
        path: "/api".to_string(),
        host: "example.com".to_string(),
        headers: std::collections::HashMap::new(),
    });
    assert_eq!(program.eval(&ctx).unwrap(), true);

    // Test POST request
    let ctx = RequestContext::from_test(&TestRequest {
        method: "POST".to_string(),
        path: "/api".to_string(),
        host: "example.com".to_string(),
        headers: std::collections::HashMap::new(),
    });
    assert_eq!(program.eval(&ctx).unwrap(), false);
}

#[test]
fn test_full_pipeline_with_headers() {
    let config_json = r#"{
        "expression": "contains(headerList(\"X-Teams\"), \"platform-eng\")",
        "denyStatusCode": 403,
        "denyBody": "Access denied"
    }"#;

    let config: Config = serde_json::from_str(config_json).unwrap();
    let program = Program::compile(&config.expression).unwrap();

    // Test with correct team
    let mut headers = std::collections::HashMap::new();
    headers.insert("X-Teams".to_string(), "platform-eng,devops".to_string());

    let ctx = RequestContext::from_test(&TestRequest {
        method: "GET".to_string(),
        path: "/api".to_string(),
        host: "example.com".to_string(),
        headers: headers.clone(),
    });
    assert_eq!(program.eval(&ctx).unwrap(), true);

    // Test with wrong team
    headers.insert("X-Teams".to_string(), "marketing".to_string());
    let ctx = RequestContext::from_test(&TestRequest {
        method: "GET".to_string(),
        path: "/api".to_string(),
        host: "example.com".to_string(),
        headers,
    });
    assert_eq!(program.eval(&ctx).unwrap(), false);
}

#[test]
fn test_config_with_test_cases() {
    let config_json = r#"{
        "expression": "method == \"GET\" AND path startsWith \"/api\"",
        "denyStatusCode": 403,
        "denyBody": "Forbidden",
        "tests": [
            {
                "name": "GET /api allowed",
                "request": {
                    "method": "GET",
                    "path": "/api/users"
                },
                "expect": true
            },
            {
                "name": "POST /api denied",
                "request": {
                    "method": "POST",
                    "path": "/api/users"
                },
                "expect": false
            },
            {
                "name": "GET /public denied",
                "request": {
                    "method": "GET",
                    "path": "/public"
                },
                "expect": false
            }
        ]
    }"#;

    let config: Config = serde_json::from_str(config_json).unwrap();
    let program = Program::compile(&config.expression).unwrap();

    // Validate all test cases
    for test_case in &config.tests {
        let ctx = RequestContext::from_test(&test_case.request);
        let result = program.eval(&ctx).unwrap();
        assert_eq!(
            result, test_case.expect,
            "Test '{}' failed: expected {}, got {}",
            test_case.name, test_case.expect, result
        );
    }
}

#[test]
fn test_complex_expression_pipeline() {
    let config_json = r#"{
        "expression": "(method == \"GET\" OR method == \"HEAD\") AND (path startsWith \"/api\" OR path startsWith \"/public\")",
        "tests": [
            {
                "name": "GET /api",
                "request": {"method": "GET", "path": "/api/v1"},
                "expect": true
            },
            {
                "name": "HEAD /public",
                "request": {"method": "HEAD", "path": "/public/index.html"},
                "expect": true
            },
            {
                "name": "POST /api",
                "request": {"method": "POST", "path": "/api/v1"},
                "expect": false
            },
            {
                "name": "GET /admin",
                "request": {"method": "GET", "path": "/admin"},
                "expect": false
            }
        ]
    }"#;

    let config: Config = serde_json::from_str(config_json).unwrap();
    let program = Program::compile(&config.expression).unwrap();

    for test_case in &config.tests {
        let ctx = RequestContext::from_test(&test_case.request);
        let result = program.eval(&ctx).unwrap();
        assert_eq!(result, test_case.expect, "Test '{}' failed", test_case.name);
    }
}

#[test]
fn test_regex_in_pipeline() {
    let config_json = r#"{
        "expression": "matches(path, \"^/api/v[0-9]+/.*\")"
    }"#;

    let config: Config = serde_json::from_str(config_json).unwrap();
    let program = Program::compile(&config.expression).unwrap();

    // Test matching path
    let ctx = RequestContext::from_test(&TestRequest {
        method: "GET".to_string(),
        path: "/api/v1/users".to_string(),
        host: "example.com".to_string(),
        headers: std::collections::HashMap::new(),
    });
    assert_eq!(program.eval(&ctx).unwrap(), true);

    // Test non-matching path
    let ctx = RequestContext::from_test(&TestRequest {
        method: "GET".to_string(),
        path: "/api/users".to_string(),
        host: "example.com".to_string(),
        headers: std::collections::HashMap::new(),
    });
    assert_eq!(program.eval(&ctx).unwrap(), false);
}

#[test]
fn test_variadic_functions_pipeline() {
    let config_json = r#"{
        "expression": "anyOf(headerList(\"X-Roles\"), \"admin\", \"moderator\")"
    }"#;

    let config: Config = serde_json::from_str(config_json).unwrap();
    let program = Program::compile(&config.expression).unwrap();

    // Test with admin role
    let mut headers = std::collections::HashMap::new();
    headers.insert("X-Roles".to_string(), "admin,user".to_string());
    let ctx = RequestContext::from_test(&TestRequest {
        method: "GET".to_string(),
        path: "/admin".to_string(),
        host: "example.com".to_string(),
        headers: headers.clone(),
    });
    assert_eq!(program.eval(&ctx).unwrap(), true);

    // Test with moderator role
    headers.insert("X-Roles".to_string(), "moderator,user".to_string());
    let ctx = RequestContext::from_test(&TestRequest {
        method: "GET".to_string(),
        path: "/admin".to_string(),
        host: "example.com".to_string(),
        headers: headers.clone(),
    });
    assert_eq!(program.eval(&ctx).unwrap(), true);

    // Test without required roles
    headers.insert("X-Roles".to_string(), "user".to_string());
    let ctx = RequestContext::from_test(&TestRequest {
        method: "GET".to_string(),
        path: "/admin".to_string(),
        host: "example.com".to_string(),
        headers,
    });
    assert_eq!(program.eval(&ctx).unwrap(), false);
}
