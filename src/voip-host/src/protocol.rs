use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

pub const SUPPORTED_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvelopeKind {
    Command,
    Event,
    Result,
    Error,
    Heartbeat,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerEnvelope {
    #[serde(default = "default_schema_version")]
    pub schema_version: u16,
    pub kind: EnvelopeKind,
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub timestamp_ms: u64,
    #[serde(default)]
    pub deadline_ms: u64,
    #[serde(default = "empty_payload")]
    pub payload: Value,
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("invalid JSON worker envelope: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("unsupported schema_version {actual}; expected {expected}")]
    UnsupportedSchema { actual: u16, expected: u16 },
    #[error("invalid worker envelope: {0}")]
    InvalidEnvelope(String),
}

fn default_schema_version() -> u16 {
    SUPPORTED_SCHEMA_VERSION
}

fn empty_payload() -> Value {
    json!({})
}

impl WorkerEnvelope {
    pub fn decode(line: &[u8]) -> Result<Self, ProtocolError> {
        let envelope: WorkerEnvelope = serde_json::from_slice(line)?;
        envelope.validate()?;
        Ok(envelope)
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        self.validate()?;
        let mut encoded = serde_json::to_vec(self)?;
        encoded.push(b'\n');
        Ok(encoded)
    }

    pub fn event(message_type: impl Into<String>, payload: Value) -> Self {
        Self {
            schema_version: SUPPORTED_SCHEMA_VERSION,
            kind: EnvelopeKind::Event,
            message_type: message_type.into(),
            request_id: None,
            timestamp_ms: 0,
            deadline_ms: 0,
            payload,
        }
    }

    pub fn result(
        message_type: impl Into<String>,
        request_id: Option<String>,
        payload: Value,
    ) -> Self {
        Self {
            schema_version: SUPPORTED_SCHEMA_VERSION,
            kind: EnvelopeKind::Result,
            message_type: message_type.into(),
            request_id,
            timestamp_ms: 0,
            deadline_ms: 0,
            payload,
        }
    }

    pub fn error(
        message_type: impl Into<String>,
        request_id: Option<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: SUPPORTED_SCHEMA_VERSION,
            kind: EnvelopeKind::Error,
            message_type: message_type.into(),
            request_id,
            timestamp_ms: 0,
            deadline_ms: 0,
            payload: json!({
                "code": code.into(),
                "message": message.into(),
            }),
        }
    }

    pub fn validate(&self) -> Result<(), ProtocolError> {
        if self.schema_version != SUPPORTED_SCHEMA_VERSION {
            return Err(ProtocolError::UnsupportedSchema {
                actual: self.schema_version,
                expected: SUPPORTED_SCHEMA_VERSION,
            });
        }
        if self.message_type.trim().is_empty() {
            return Err(ProtocolError::InvalidEnvelope(
                "type must be a non-empty string".to_string(),
            ));
        }
        if !self.payload.is_object() {
            return Err(ProtocolError::InvalidEnvelope(
                "payload must be an object".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_accepts_voip_health_command() {
        let raw = br#"{"schema_version":1,"kind":"command","type":"voip.health","request_id":"r1","payload":{}}"#;

        let envelope = WorkerEnvelope::decode(raw).expect("decode");

        assert_eq!(envelope.schema_version, SUPPORTED_SCHEMA_VERSION);
        assert_eq!(envelope.kind, EnvelopeKind::Command);
        assert_eq!(envelope.message_type, "voip.health");
        assert_eq!(envelope.request_id.as_deref(), Some("r1"));
    }

    #[test]
    fn encode_ready_event_has_newline() {
        let encoded = WorkerEnvelope::event("voip.ready", json!({"capabilities":["calls"]}))
            .encode()
            .expect("encode");

        assert!(encoded.ends_with(b"\n"));
        assert!(std::str::from_utf8(&encoded)
            .unwrap()
            .contains("\"type\":\"voip.ready\""));
    }

    #[test]
    fn rejects_array_payload() {
        let err = WorkerEnvelope::decode(
            br#"{"schema_version":1,"kind":"command","type":"voip.health","payload":[]}"#,
        )
        .expect_err("payload must be rejected");

        assert!(err.to_string().contains("payload must be an object"));
    }
}
