//! RPC server

use crate::Result;
use warp::Filter;

/// RPC server
pub struct RpcServer {
    /// Server address
    pub address: String,
    /// Server port
    pub port: u16,
}

impl RpcServer {
    /// Create new RPC server
    pub fn new(address: String, port: u16) -> Self {
        Self { address, port }
    }

    /// Start RPC server
    pub async fn start(&self) -> Result<()> {
        let routes = warp::path("rpc")
            .and(warp::post())
            .and(warp::body::json())
            .map(|_body: serde_json::Value| {
                warp::reply::json(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": "Hello from Olympus RPC!"
                }))
            });

        let addr = format!("{}:{}", self.address, self.port);
        let addr: std::net::SocketAddr = addr.parse().unwrap();
        warp::serve(routes)
            .run(addr)
            .await;

        Ok(())
    }
}
