/// JSON-RPC 2.0 request message.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response message.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    pub fn ok(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: serde_json::Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    pub fn parse_error(id: serde_json::Value) -> Self {
        Self::error(id, -32700, "Parse error")
    }

    pub fn method_not_found(id: serde_json::Value, method: &str) -> Self {
        Self::error(id, -32601, format!("Method not found: {}", method))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Internal error"}}"#
                .to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_ok_serializes_with_result_field() {
        let resp = JsonRpcResponse::ok(json!(1), json!({"foo": "bar"}));
        let s = resp.to_json();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert!(v.get("result").is_some());
        assert!(v.get("error").is_none());
        assert_eq!(v["result"]["foo"], json!("bar"));
    }

    #[test]
    fn test_error_serializes_with_error_field() {
        let resp = JsonRpcResponse::error(json!(1), -32600, "Invalid Request");
        let s = resp.to_json();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert!(v.get("error").is_some());
        assert!(v.get("result").is_none());
        assert_eq!(v["error"]["code"], json!(-32600));
    }

    #[test]
    fn test_method_not_found_has_code_minus_32601() {
        let resp = JsonRpcResponse::method_not_found(json!(1), "foo/bar");
        let s = resp.to_json();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["error"]["code"], json!(-32601));
    }
}
