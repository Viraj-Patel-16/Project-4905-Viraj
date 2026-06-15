use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrafficPattern {
    Steady,
    Burst,
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantProfile {
    pub tenant_id: String,
    pub tenant_name: String,
    pub traffic_pattern: TrafficPattern,
    pub requests_per_second: u32,
    pub payload_size_bytes: u32,
    pub priority: u8,
    pub duration_seconds: u32,
}

impl TenantProfile {
    pub fn new(
        tenant_id: &str,
        tenant_name: &str,
        traffic_pattern: TrafficPattern,
        requests_per_second: u32,
        payload_size_bytes: u32,
        priority: u8,
        duration_seconds: u32,
    ) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            tenant_name: tenant_name.to_string(),
            traffic_pattern,
            requests_per_second,
            payload_size_bytes,
            priority,
            duration_seconds,
        }
    }
}