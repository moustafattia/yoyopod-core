use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

pub const SUPPORTED_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvelopeKind {
    Command,
    Event,
    Error,
    Heartbeat,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    #[serde(default = "default_schema_version")]
    pub schema_version: u16,
    pub kind: EnvelopeKind,
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(default)]
    pub request_id: String,
    #[serde(default)]
    pub timestamp_ms: u64,
    #[serde(default)]
    pub deadline_ms: u64,
    #[serde(default = "empty_payload")]
    pub payload: Value,
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("invalid JSON UI envelope: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("unsupported schema_version {actual}; expected {expected}")]
    UnsupportedSchema { actual: u16, expected: u16 },
    #[error("invalid envelope kind or payload: {0}")]
    InvalidEnvelope(String),
}

fn default_schema_version() -> u16 {
    SUPPORTED_SCHEMA_VERSION
}

fn empty_payload() -> Value {
    json!({})
}

impl Envelope {
    pub fn decode(line: &[u8]) -> Result<Self, ProtocolError> {
        let envelope: Envelope = serde_json::from_slice(line)?;
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
            request_id: String::new(),
            timestamp_ms: monotonic_millis(),
            deadline_ms: 0,
            payload,
        }
    }

    #[allow(dead_code)]
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::event(
            "ui.error",
            json!({
                "code": code.into(),
                "message": message.into(),
            }),
        )
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
                "envelope type must be a non-empty string".to_string(),
            ));
        }
        if !self.payload.is_object() {
            return Err(ProtocolError::InvalidEnvelope(
                "payload must be a JSON object".to_string(),
            ));
        }
        Ok(())
    }
}

pub fn monotonic_millis() -> u64 {
    use std::sync::OnceLock;
    use std::time::Instant;

    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decode_accepts_spec_style_command_without_schema_version() {
        let line = br#"{"kind":"command","type":"ui.show_test_scene","payload":{"counter":7}}"#;

        let envelope = Envelope::decode(line).expect("decode");

        assert_eq!(envelope.schema_version, SUPPORTED_SCHEMA_VERSION);
        assert_eq!(envelope.kind, EnvelopeKind::Command);
        assert_eq!(envelope.message_type, "ui.show_test_scene");
        assert_eq!(envelope.payload["counter"], json!(7));
    }

    #[test]
    fn encode_ready_event_terminates_with_newline() {
        let encoded = Envelope::event("ui.ready", json!({"width": 240, "height": 280}))
            .encode()
            .expect("encode");

        assert!(encoded.ends_with(b"\n"));
        assert!(std::str::from_utf8(&encoded)
            .unwrap()
            .contains("\"type\":\"ui.ready\""));
    }

    #[test]
    fn rejects_unknown_kind() {
        let err = Envelope::decode(br#"{"kind":"bogus","type":"ui.ready","payload":{}}"#)
            .expect_err("must reject invalid kind");

        assert!(err.to_string().contains("invalid JSON UI envelope"));
    }
}
