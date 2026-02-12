// Configuration structures for the authorization plugin
//
// Traefik passes plugin configuration as JSON where all values from YAML
// are serialized as strings. Custom deserializers handle both native JSON
// types (u16, bool, map) and Traefik's string-based representations.

use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Main plugin configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Authorization expression to evaluate
    pub expression: String,

    /// HTTP status code to return when authorization fails
    #[serde(
        default = "default_deny_status_code",
        deserialize_with = "deserialize_u16_from_any"
    )]
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
    #[serde(deserialize_with = "deserialize_bool_from_any")]
    pub expect: bool,
}

/// Mock HTTP request for testing
#[derive(Debug, Clone, Serialize, Default)]
pub struct TestRequest {
    /// HTTP method (GET, POST, etc.)
    pub method: String,

    /// Request path
    pub path: String,

    /// Request host
    pub host: String,

    /// Request headers (case-insensitive keys)
    pub headers: HashMap<String, String>,
}

/// Deserialize a u16 from either a number or a string.
/// Traefik serializes YAML numbers as strings (e.g., "403" instead of 403).
fn deserialize_u16_from_any<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    struct U16Visitor;

    impl<'de> Visitor<'de> for U16Visitor {
        type Value = u16;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a u16 integer or a string containing a u16 integer")
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<u16, E> {
            u16::try_from(v).map_err(|_| E::custom(format!("u16 out of range: {}", v)))
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<u16, E> {
            u16::try_from(v).map_err(|_| E::custom(format!("u16 out of range: {}", v)))
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<u16, E> {
            v.parse::<u16>()
                .map_err(|_| E::custom(format!("invalid u16 string: {:?}", v)))
        }
    }

    deserializer.deserialize_any(U16Visitor)
}

/// Deserialize a bool from either a boolean or a string.
/// Traefik serializes YAML booleans as strings (e.g., "true" instead of true).
fn deserialize_bool_from_any<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct BoolVisitor;

    impl<'de> Visitor<'de> for BoolVisitor {
        type Value = bool;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a boolean or a string containing a boolean")
        }

        fn visit_bool<E: de::Error>(self, v: bool) -> Result<bool, E> {
            Ok(v)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<bool, E> {
            match v {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(E::custom(format!("invalid bool string: {:?}", v))),
            }
        }
    }

    deserializer.deserialize_any(BoolVisitor)
}

/// Custom deserializer for TestRequest that handles Traefik's serialization quirks.
/// Traefik serializes empty YAML maps `{}` as empty strings `""`.
impl<'de> Deserialize<'de> for TestRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Method,
            Path,
            Host,
            Headers,
        }

        struct TestRequestVisitor;

        impl<'de> Visitor<'de> for TestRequestVisitor {
            type Value = TestRequest;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a TestRequest object")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<TestRequest, M::Error> {
                let mut method = None;
                let mut path = None;
                let mut host = None;
                let mut headers = None;

                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Method => method = Some(map.next_value()?),
                        Field::Path => path = Some(map.next_value()?),
                        Field::Host => host = Some(map.next_value()?),
                        Field::Headers => {
                            // Traefik serializes empty maps as empty strings.
                            headers = Some(map.next_value::<HeadersOrString>()?);
                        }
                    }
                }

                Ok(TestRequest {
                    method: method.unwrap_or_default(),
                    path: path.unwrap_or_default(),
                    host: host.unwrap_or_default(),
                    headers: headers.map(|h| h.into_map()).unwrap_or_default(),
                })
            }
        }

        deserializer.deserialize_map(TestRequestVisitor)
    }
}

/// Helper to deserialize headers as either a map or an empty string.
enum HeadersOrString {
    Map(HashMap<String, String>),
    Empty,
}

impl HeadersOrString {
    fn into_map(self) -> HashMap<String, String> {
        match self {
            HeadersOrString::Map(m) => m,
            HeadersOrString::Empty => HashMap::new(),
        }
    }
}

impl<'de> Deserialize<'de> for HeadersOrString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HeadersVisitor;

        impl<'de> Visitor<'de> for HeadersVisitor {
            type Value = HeadersOrString;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a map of headers or an empty string")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<HeadersOrString, E> {
                if v.is_empty() {
                    Ok(HeadersOrString::Empty)
                } else {
                    Err(E::custom(format!("unexpected string for headers: {:?}", v)))
                }
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<HeadersOrString, M::Error> {
                let mut headers = HashMap::new();
                while let Some((k, v)) = map.next_entry()? {
                    headers.insert(k, v);
                }
                Ok(HeadersOrString::Map(headers))
            }
        }

        deserializer.deserialize_any(HeadersVisitor)
    }
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
    fn test_config_deserialization_traefik_strings() {
        // Traefik serializes all YAML values as strings
        let json = r#"{
            "expression": "method == \"GET\"",
            "denyStatusCode": "403",
            "denyBody": "Forbidden",
            "tests": [
                {
                    "name": "test1",
                    "request": {
                        "method": "GET",
                        "headers": ""
                    },
                    "expect": "true"
                },
                {
                    "name": "test2",
                    "request": {
                        "headers": {"X-Team": "eng"}
                    },
                    "expect": "false"
                }
            ]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.deny_status_code, 403);
        assert_eq!(config.tests[0].expect, true);
        assert_eq!(config.tests[0].request.headers.len(), 0);
        assert_eq!(config.tests[1].expect, false);
        assert_eq!(
            config.tests[1].request.headers.get("X-Team"),
            Some(&"eng".to_string())
        );
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
