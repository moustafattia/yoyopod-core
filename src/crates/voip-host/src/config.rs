use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoipConfig {
    #[serde(default = "default_sip_server")]
    pub sip_server: String,
    #[serde(default)]
    pub sip_username: String,
    #[serde(default)]
    pub sip_password: String,
    #[serde(default)]
    pub sip_password_ha1: String,
    #[serde(default)]
    pub sip_identity: String,
    #[serde(default)]
    pub factory_config_path: String,
    #[serde(default = "default_transport")]
    pub transport: String,
    #[serde(default)]
    pub stun_server: String,
    #[serde(default)]
    pub conference_factory_uri: String,
    #[serde(default)]
    pub file_transfer_server_url: String,
    #[serde(default)]
    pub lime_server_url: String,
    #[serde(default = "default_iterate_interval_ms")]
    pub iterate_interval_ms: u64,
    #[serde(default)]
    pub voice_note_store_dir: String,
    #[serde(default)]
    pub auto_download_incoming_voice_recordings: bool,
    #[serde(default = "default_audio_device")]
    pub playback_dev_id: String,
    #[serde(default = "default_audio_device")]
    pub ringer_dev_id: String,
    #[serde(default = "default_audio_device")]
    pub capture_dev_id: String,
    #[serde(default = "default_audio_device")]
    pub media_dev_id: String,
    #[serde(default = "default_mic_gain")]
    pub mic_gain: i32,
    #[serde(default = "default_output_volume")]
    pub output_volume: i32,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid voip config payload: {0}")]
    InvalidPayload(#[from] serde_json::Error),
    #[error("sip_identity is required for Rust VoIP host registration")]
    MissingSipIdentity,
    #[error("sip_server is required for Rust VoIP host registration")]
    MissingSipServer,
}

impl VoipConfig {
    pub fn from_payload(payload: &Value) -> Result<Self, ConfigError> {
        let config: Self = serde_json::from_value(payload.clone())?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.sip_server.trim().is_empty() {
            return Err(ConfigError::MissingSipServer);
        }
        if self.sip_identity.trim().is_empty() {
            return Err(ConfigError::MissingSipIdentity);
        }
        Ok(())
    }
}

fn default_sip_server() -> String {
    "sip.linphone.org".to_string()
}

fn default_transport() -> String {
    "tcp".to_string()
}

fn default_iterate_interval_ms() -> u64 {
    20
}

fn default_audio_device() -> String {
    "ALSA: wm8960-soundcard".to_string()
}

fn default_mic_gain() -> i32 {
    80
}

fn default_output_volume() -> i32 {
    100
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn config_from_payload_accepts_python_voip_config_shape() {
        let payload = json!({
            "sip_server": "sip.example.com",
            "sip_username": "alice",
            "sip_password": "secret",
            "sip_identity": "sip:alice@example.com",
            "transport": "tcp",
            "iterate_interval_ms": 20,
            "playback_dev_id": "ALSA: wm8960-soundcard",
            "ringer_dev_id": "ALSA: wm8960-soundcard",
            "capture_dev_id": "ALSA: wm8960-soundcard",
            "media_dev_id": "ALSA: wm8960-soundcard",
            "mic_gain": 80,
            "output_volume": 100
        });

        let config = VoipConfig::from_payload(&payload).expect("config");

        assert_eq!(config.sip_server, "sip.example.com");
        assert_eq!(config.sip_identity, "sip:alice@example.com");
        assert_eq!(config.iterate_interval_ms, 20);
        assert_eq!(config.transport, "tcp");
    }

    #[test]
    fn config_rejects_missing_identity() {
        let payload = json!({"sip_server":"sip.example.com"});

        let error = VoipConfig::from_payload(&payload).expect_err("must reject");

        assert!(error.to_string().contains("sip_identity"));
    }
}
