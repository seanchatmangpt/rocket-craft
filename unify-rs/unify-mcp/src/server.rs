use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use crate::resource::{ResourceDescriptor, ResourceRegistry};
use crate::tool::{ToolDescriptor, ToolRegistry};
use serde_json::json;

/// Information about the MCP server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// The main MCP server that dispatches JSON-RPC requests.
pub struct McpServer {
    tools: ToolRegistry,
    resources: ResourceRegistry,
    server_info: ServerInfo,
}

impl McpServer {
    pub fn new(info: ServerInfo) -> Self {
        Self {
            tools: ToolRegistry::new(),
            resources: ResourceRegistry::new(),
            server_info: info,
        }
    }

    /// Builder-style method to add a tool.
    pub fn with_tool(
        mut self,
        desc: ToolDescriptor,
        handler: impl Fn(serde_json::Value) -> Result<serde_json::Value, String>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.tools.register(desc, handler);
        self
    }

    /// Builder-style method to add a resource.
    pub fn with_resource(
        mut self,
        desc: ResourceDescriptor,
        handler: impl Fn(&str) -> Result<serde_json::Value, String> + Send + Sync + 'static,
    ) -> Self {
        self.resources.register(desc, handler);
        self
    }

    /// Handle a single JSON-RPC request string, return response string.
    pub fn handle(&self, request_json: &str) -> String {
        let req: JsonRpcRequest = match serde_json::from_str(request_json) {
            Ok(r) => r,
            Err(_) => return JsonRpcResponse::parse_error(json!(null)).to_json(),
        };
        self.dispatch(req).to_json()
    }

    /// Dispatch a parsed JSON-RPC request to the appropriate handler.
    fn dispatch(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        match req.method.as_str() {
            "initialize" => {
                JsonRpcResponse::ok(
                    req.id,
                    json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {},
                            "resources": {}
                        },
                        "serverInfo": {
                            "name": self.server_info.name,
                            "version": self.server_info.version
                        }
                    }),
                )
            }
            "tools/list" => {
                let tools: Vec<serde_json::Value> = self
                    .tools
                    .list()
                    .iter()
                    .map(|t| {
                        json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": t.input_schema
                        })
                    })
                    .collect();
                JsonRpcResponse::ok(req.id, json!({ "tools": tools }))
            }
            "tools/call" => {
                let params = req.params.unwrap_or(json!({}));
                let tool_name = match params.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n.to_string(),
                    None => {
                        return JsonRpcResponse::error(
                            req.id,
                            -32602,
                            "Invalid params: missing tool name",
                        )
                    }
                };
                let tool_params = params.get("arguments").cloned().unwrap_or(json!({}));

                if !self.tools.has(&tool_name) {
                    return JsonRpcResponse::error(
                        req.id,
                        -32601,
                        format!("Tool not found: {}", tool_name),
                    );
                }

                match self.tools.call(&tool_name, tool_params) {
                    Ok(result) => JsonRpcResponse::ok(
                        req.id,
                        json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": serde_json::to_string(&result).unwrap_or_default()
                                }
                            ]
                        }),
                    ),
                    Err(e) => JsonRpcResponse::error(req.id, -32603, e),
                }
            }
            "resources/list" => {
                let resources: Vec<serde_json::Value> = self
                    .resources
                    .list()
                    .iter()
                    .map(|r| {
                        json!({
                            "uri": r.uri,
                            "name": r.name,
                            "mimeType": r.mime_type,
                            "description": r.description
                        })
                    })
                    .collect();
                JsonRpcResponse::ok(req.id, json!({ "resources": resources }))
            }
            "resources/read" => {
                let params = req.params.unwrap_or(json!({}));
                let uri = match params.get("uri").and_then(|v| v.as_str()) {
                    Some(u) => u.to_string(),
                    None => {
                        return JsonRpcResponse::error(
                            req.id,
                            -32602,
                            "Invalid params: missing uri",
                        )
                    }
                };
                match self.resources.read(&uri) {
                    Ok(content) => JsonRpcResponse::ok(
                        req.id,
                        json!({
                            "contents": [
                                {
                                    "uri": uri,
                                    "mimeType": "application/json",
                                    "text": serde_json::to_string(&content).unwrap_or_default()
                                }
                            ]
                        }),
                    ),
                    Err(e) => JsonRpcResponse::error(req.id, -32602, e),
                }
            }
            method => JsonRpcResponse::method_not_found(req.id, method),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_server() -> McpServer {
        McpServer::new(ServerInfo {
            name: "test-server".into(),
            version: "0.0.1".into(),
        })
    }

    #[test]
    fn test_initialize_returns_server_info() {
        let server = make_server();
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let resp = server.handle(req);
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(v["result"]["serverInfo"]["name"] == json!("test-server"));
        assert!(v.get("error").is_none());
    }

    #[test]
    fn test_tools_list_returns_registered_tools() {
        let desc = ToolDescriptor {
            name: "my_tool".into(),
            description: "does stuff".into(),
            input_schema: json!({"type": "object"}),
        };
        let server = make_server().with_tool(desc, |_| Ok(json!("ok")));
        let req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
        let resp = server.handle(req);
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        let tools = &v["result"]["tools"];
        assert!(tools.is_array());
        assert_eq!(tools.as_array().unwrap().len(), 1);
        assert_eq!(tools[0]["name"], json!("my_tool"));
    }

    #[test]
    fn test_tools_call_known_tool_returns_result() {
        let desc = ToolDescriptor {
            name: "greet".into(),
            description: "greets".into(),
            input_schema: json!({"type": "object"}),
        };
        let server = make_server().with_tool(desc, |params| {
            let name = params["name"].as_str().unwrap_or("world");
            Ok(json!(format!("Hello, {}!", name)))
        });
        let req = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"greet","arguments":{"name":"Alice"}}}"#;
        let resp = server.handle(req);
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(v.get("error").is_none());
        assert!(v["result"]["content"].is_array());
    }

    #[test]
    fn test_tools_call_unknown_tool_returns_error() {
        let server = make_server();
        let req = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"nonexistent","arguments":{}}}"#;
        let resp = server.handle(req);
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(v.get("error").is_some());
        assert_eq!(v["error"]["code"], json!(-32601));
    }

    #[test]
    fn test_invalid_json_returns_parse_error() {
        let server = make_server();
        let resp = server.handle("{not valid json");
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(v.get("error").is_some());
        assert_eq!(v["error"]["code"], json!(-32700));
    }

    #[test]
    fn test_resources_list_returns_registered_resources() {
        use crate::resource::ResourceDescriptor;
        let desc = ResourceDescriptor {
            uri: "unify://test".into(),
            name: "Test".into(),
            mime_type: "application/json".into(),
            description: "A test resource".into(),
        };
        let server = make_server().with_resource(desc, |_| Ok(json!({"data": 42})));
        let req = r#"{"jsonrpc":"2.0","id":5,"method":"resources/list","params":{}}"#;
        let resp = server.handle(req);
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        let resources = &v["result"]["resources"];
        assert!(resources.is_array());
        assert_eq!(resources.as_array().unwrap().len(), 1);
        assert_eq!(resources[0]["uri"], json!("unify://test"));
    }

    #[test]
    fn test_unknown_method_returns_method_not_found() {
        let server = make_server();
        let req = r#"{"jsonrpc":"2.0","id":6,"method":"unknown/method","params":{}}"#;
        let resp = server.handle(req);
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(v.get("error").is_some());
        assert_eq!(v["error"]["code"], json!(-32601));
    }
}
