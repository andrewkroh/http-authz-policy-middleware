// Configuration structures for the authorization plugin

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main plugin configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Authorization expression to evaluate
    pub expression: String,

    /// HTTP status code to return when authorization fails
    #[serde(default = "default_deny_status_code")]
    pub deny_status_code: u16,

    /// Response body to return when authorization fails
    #[serde(default = "default_deny_body")]
    pub deny_body: String,

    /// Test cases to validate at startup
    #[serde(default)]
    pub tests: Vec<TestCase>,
}

fn default_deny_status_code() -> u16 {
    403
}

fn default_deny_body() -> String {
    "Forbidden".to_string()
}

/// Test case for validating expressions at startup
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestCase {
    /// Descriptive name for the test
    pub name: String,

    /// Mock request to test against
    pub request: TestRequest,

    /// Expected authorization result (true = allow, false = deny)
    pub expect: bool,
}

/// Mock HTTP request for testing
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TestRequest {
    /// HTTP method (GET, POST, etc.)
    #[serde(default)]
    pub method: String,

    /// Request path
    #[serde(default)]
    pub path: String,

    /// Request host
    #[serde(default)]
    pub host: String,

    /// Request headers (case-insensitive keys)
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization_minimal() {
        let json = r#"{"expression": "method == \"GET\""}"#;
        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.expression, "method == \"GET\"");
        assert_eq!(config.deny_status_code, 403);
        assert_eq!(config.deny_body, "Forbidden");
        assert_eq!(config.tests.len(), 0);
    }

    #[test]
    fn test_config_deserialization_full() {
        let json = r#"{
            "expression": "method == \"POST\"",
            "denyStatusCode": 401,
            "denyBody": "Unauthorized",
            "tests": [
                {
                    "name": "POST allowed",
                    "request": {
                        "method": "POST",
                        "path": "/api",
                        "host": "example.com",
                        "headers": {
                            "X-Test": "value"
                        }
                    },
                    "expect": true
                }
            ]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.expression, "method == \"POST\"");
        assert_eq!(config.deny_status_code, 401);
        assert_eq!(config.deny_body, "Unauthorized");
        assert_eq!(config.tests.len(), 1);
        assert_eq!(config.tests[0].name, "POST allowed");
        assert_eq!(config.tests[0].request.method, "POST");
        assert_eq!(config.tests[0].request.path, "/api");
        assert_eq!(config.tests[0].request.host, "example.com");
        assert_eq!(
            config.tests[0].request.headers.get("X-Test"),
            Some(&"value".to_string())
        );
        assert_eq!(config.tests[0].expect, true);
    }

    #[test]
    fn test_test_request_default() {
        let req = TestRequest::default();
        assert_eq!(req.method, "");
        assert_eq!(req.path, "");
        assert_eq!(req.host, "");
        assert_eq!(req.headers.len(), 0);
    }
}
