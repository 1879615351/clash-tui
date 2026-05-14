use serde::{Deserialize, Serialize};

/// IPC message types for Client ↔ Daemon communication.
/// Uses JSON encoding over TCP (127.0.0.1 loopback).

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcRequest {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcResponse {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl IpcResponse {
    pub fn success(id: u64, ok: serde_json::Value) -> Self {
        Self {
            id,
            ok: Some(ok),
            error: None,
        }
    }

    pub fn failure(id: u64, error: String) -> Self {
        Self {
            id,
            ok: None,
            error: Some(error),
        }
    }
}
