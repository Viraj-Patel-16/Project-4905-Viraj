use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPayload {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

impl NetworkPayload {
    pub fn new(bytes: Vec<u8>, content_type: &str) -> Self {
        Self {
            bytes,
            content_type: content_type.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingMetadata {
    pub timestamp_ms: u64,
    pub request_sequence: u64,
}

impl TrackingMetadata {
    pub fn new(timestamp_ms: u64, request_sequence: u64) -> Self {
        Self {
            timestamp_ms,
            request_sequence,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub tenant_id: String,
    pub payload: NetworkPayload,
    pub metadata: TrackingMetadata,
}

impl Task {
    pub fn new(tenant_id: &str, payload: NetworkPayload, metadata: TrackingMetadata) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id: tenant_id.to_string(),
            payload,
            metadata,
        }
    }
}
