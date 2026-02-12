// Request context for expression evaluation

use crate::config::TestRequest;
use std::collections::HashMap;

/// Context containing HTTP request attributes for expression evaluation
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// HTTP method (GET, POST, etc.)
    pub method: String,

    /// Request path
    pub path: String,

    /// Request host
    pub host: String,

    /// Headers map (lowercase key -> first value)
    /// Used by header() function
    headers: HashMap<String, String>,

    /// All headers map (lowercase key -> all values)
    /// Used by headerValues() and headerList() functions
    all_headers: HashMap<String, Vec<String>>,
}

impl RequestContext {
    /// Create a RequestContext from an http-wasm Request
    #[cfg(target_arch = "wasm32")]
    pub fn from_request(request: &http_wasm_guest::host::Request) -> Self {
        let headers = HashMap::new();
        let all_headers = HashMap::new();

        // Extract method
        let method = String::from_utf8_lossy(&request.method()).to_string();

        // Extract URI
        let uri = String::from_utf8_lossy(&request.uri()).to_string();
        let path = uri.split('?').next().unwrap_or(&uri).to_string();

        // Extract host - simplified for now
        // TODO: Properly extract host from headers
        let host = String::new();

        // TODO: Properly extract all headers
        // http-wasm-guest 0.7 API doesn't provide easy header iteration
        // This is a limitation that needs to be addressed

        RequestContext {
            method,
            path,
            host,
            headers,
            all_headers,
        }
    }

    /// Create a RequestContext from a test request
    pub fn from_test(test_req: &TestRequest) -> Self {
        let mut headers = HashMap::new();
        let mut all_headers = HashMap::new();

        // Normalize header names to lowercase for case-insensitive access
        for (name, value) in &test_req.headers {
            let lowercase_name = name.to_lowercase();

            // Store first value
            headers
                .entry(lowercase_name.clone())
                .or_insert_with(|| value.clone());

            // Store all values
            all_headers
                .entry(lowercase_name)
                .or_insert_with(Vec::new)
                .push(value.clone());
        }

        RequestContext {
            method: test_req.method.clone(),
            path: test_req.path.clone(),
            host: test_req.host.clone(),
            headers,
            all_headers,
        }
    }

    /// Get the first value of a header (case-insensitive)
    /// Returns empty string if header not found
    pub fn header(&self, name: &str) -> &str {
        self.headers
            .get(&name.to_lowercase())
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Get all values of a header (case-insensitive)
    /// Returns empty slice if header not found
    pub fn header_values(&self, name: &str) -> &[String] {
        self.all_headers
            .get(&name.to_lowercase())
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get header value split by comma into a list
    /// Trims whitespace from each value
    /// Returns empty vec if header not found
    pub fn header_list(&self, name: &str) -> Vec<String> {
        let value = self.header(name);
        if value.is_empty() {
            return Vec::new();
        }

        value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_test_basic() {
        let test_req = TestRequest {
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            host: "example.com".to_string(),
            headers: HashMap::new(),
        };

        let ctx = RequestContext::from_test(&test_req);
        assert_eq!(ctx.method, "GET");
        assert_eq!(ctx.path, "/api/users");
        assert_eq!(ctx.host, "example.com");
    }

    #[test]
    fn test_header_case_insensitive() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-Auth-User".to_string(), "alice".to_string());

        let test_req = TestRequest {
            method: "POST".to_string(),
            path: "/api".to_string(),
            host: "example.com".to_string(),
            headers,
        };

        let ctx = RequestContext::from_test(&test_req);

        // Case-insensitive lookups
        assert_eq!(ctx.header("content-type"), "application/json");
        assert_eq!(ctx.header("Content-Type"), "application/json");
        assert_eq!(ctx.header("CONTENT-TYPE"), "application/json");
        assert_eq!(ctx.header("x-auth-user"), "alice");
        assert_eq!(ctx.header("X-Auth-User"), "alice");
    }

    #[test]
    fn test_header_missing() {
        let test_req = TestRequest::default();
        let ctx = RequestContext::from_test(&test_req);

        assert_eq!(ctx.header("missing"), "");
    }

    #[test]
    fn test_header_values() {
        let mut headers = HashMap::new();
        headers.insert("X-Team".to_string(), "platform-eng".to_string());

        let test_req = TestRequest {
            headers,
            ..Default::default()
        };

        let ctx = RequestContext::from_test(&test_req);

        let values = ctx.header_values("x-team");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], "platform-eng");
    }

    #[test]
    fn test_header_list_single() {
        let mut headers = HashMap::new();
        headers.insert("X-Teams".to_string(), "platform-eng,devops,sre".to_string());

        let test_req = TestRequest {
            headers,
            ..Default::default()
        };

        let ctx = RequestContext::from_test(&test_req);

        let list = ctx.header_list("x-teams");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0], "platform-eng");
        assert_eq!(list[1], "devops");
        assert_eq!(list[2], "sre");
    }

    #[test]
    fn test_header_list_with_spaces() {
        let mut headers = HashMap::new();
        headers.insert(
            "X-Teams".to_string(),
            "platform-eng , devops , sre".to_string(),
        );

        let test_req = TestRequest {
            headers,
            ..Default::default()
        };

        let ctx = RequestContext::from_test(&test_req);

        let list = ctx.header_list("x-teams");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0], "platform-eng");
        assert_eq!(list[1], "devops");
        assert_eq!(list[2], "sre");
    }

    #[test]
    fn test_header_list_empty() {
        let test_req = TestRequest::default();
        let ctx = RequestContext::from_test(&test_req);

        let list = ctx.header_list("missing");
        assert_eq!(list.len(), 0);
    }
}
