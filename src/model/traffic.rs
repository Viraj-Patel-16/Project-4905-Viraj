use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficEvent {
    pub timestamp_ms: u64,
    pub tenant_id: String,
    pub request_id: u64,
    pub payload_size_bytes: u32,
    pub target_system: String,
}

impl TrafficEvent {
    pub fn new(
        timestamp_ms: u64,
        tenant_id: &str,
        request_id: u64,
        payload_size_bytes: u32,
        target_system: &str,
    ) -> Self {
        Self {
            timestamp_ms,
            tenant_id: tenant_id.to_string(),
            request_id,
            payload_size_bytes,
            target_system: target_system.to_string(),
        }
    }
}
