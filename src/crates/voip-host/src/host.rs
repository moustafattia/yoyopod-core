use crate::config::VoipConfig;
use serde_json::json;

#[derive(Debug, Default)]
pub struct VoipHost {
    config: Option<VoipConfig>,
    registered: bool,
    active_call_id: Option<String>,
}

impl VoipHost {
    pub fn configure(&mut self, config: VoipConfig) {
        self.config = Some(config);
        self.registered = false;
        self.active_call_id = None;
    }

    pub fn mark_registered(&mut self, registered: bool) {
        self.registered = registered;
    }

    pub fn set_active_call_id(&mut self, call_id: Option<String>) {
        self.active_call_id = call_id;
    }

    pub fn health_payload(&self) -> serde_json::Value {
        json!({
            "configured": self.config.is_some(),
            "registered": self.registered,
            "active_call_id": self.active_call_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn config() -> VoipConfig {
        VoipConfig::from_payload(&json!({
            "sip_server": "sip.example.com",
            "sip_identity": "sip:alice@example.com"
        }))
        .unwrap()
    }

    #[test]
    fn health_reports_configured_registered_and_call_id() {
        let mut host = VoipHost::default();
        host.configure(config());
        host.mark_registered(true);
        host.set_active_call_id(Some("call-1".to_string()));

        let payload = host.health_payload();

        assert_eq!(payload["configured"], true);
        assert_eq!(payload["registered"], true);
        assert_eq!(payload["active_call_id"], "call-1");
    }
}
