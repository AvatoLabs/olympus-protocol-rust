//! RPC methods

/// JSON-RPC request
#[derive(serde::Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: serde_json::Value,
}

/// JSON-RPC response
#[derive(serde::Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
    pub id: serde_json::Value,
}

/// JSON-RPC error
#[derive(serde::Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

/// RPC method handler
pub struct RpcMethods {
    // TODO: Add method handlers
}

impl RpcMethods {
    /// Create new RPC methods
    pub fn new() -> Self {
        Self {}
    }

    /// Handle RPC request
    pub fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "eth_blockNumber" => self.get_block_number(request.id),
            "eth_getBalance" => self.get_balance(request.params, request.id),
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_string(),
                }),
                id: request.id,
            },
        }
    }

    /// Get current block number
    fn get_block_number(&self, id: serde_json::Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::Value::String("0x0".to_string())),
            error: None,
            id,
        }
    }

    /// Get account balance
    fn get_balance(&self, _params: serde_json::Value, id: serde_json::Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::Value::String("0x0".to_string())),
            error: None,
            id,
        }
    }
}
