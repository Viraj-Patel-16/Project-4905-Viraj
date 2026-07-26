use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
    pub burst_size: u32,
    pub burst_interval_ms: u64,
    pub random_min_interval_ms: u64,
    pub random_max_interval_ms: u64,
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
        let default_interval_ms = (1000_u64 / u64::from(requests_per_second.max(1))).max(1);
        Self {
            tenant_id: tenant_id.to_string(),
            tenant_name: tenant_name.to_string(),
            traffic_pattern,
            requests_per_second,
            payload_size_bytes,
            priority,
            duration_seconds,
            burst_size: requests_per_second.max(1),
            burst_interval_ms: 1000,
            random_min_interval_ms: default_interval_ms,
            random_max_interval_ms: default_interval_ms.saturating_mul(3),
        }
    }

    pub fn with_burst_config(mut self, burst_size: u32, burst_interval_ms: u64) -> Self {
        self.burst_size = burst_size.max(1);
        self.burst_interval_ms = burst_interval_ms.max(1);
        self
    }

    pub fn with_random_interval_config(
        mut self,
        min_interval_ms: u64,
        max_interval_ms: u64,
    ) -> Self {
        let min_value = min_interval_ms.max(1);
        let max_value = max_interval_ms.max(min_value);
        self.random_min_interval_ms = min_value;
        self.random_max_interval_ms = max_value;
        self
    }
}
