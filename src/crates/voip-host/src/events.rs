use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationState {
    None,
    Progress,
    Ok,
    Cleared,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallState {
    Idle,
    Incoming,
    OutgoingInit,
    OutgoingProgress,
    OutgoingRinging,
    OutgoingEarlyMedia,
    Connected,
    StreamsRunning,
    Paused,
    PausedByRemote,
    UpdatedByRemote,
    Released,
    Error,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Text,
    VoiceNote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDirection {
    Incoming,
    Outgoing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDeliveryState {
    Queued,
    Sending,
    Sent,
    Delivered,
    Failed,
}

impl RegistrationState {
    pub fn from_native(value: i32) -> Self {
        match value {
            1 => Self::Progress,
            2 => Self::Ok,
            3 => Self::Cleared,
            4 => Self::Failed,
            _ => Self::None,
        }
    }

    pub fn as_protocol(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Progress => "progress",
            Self::Ok => "ok",
            Self::Cleared => "cleared",
            Self::Failed => "failed",
        }
    }
}

impl CallState {
    pub fn from_native(value: i32) -> Self {
        match value {
            1 => Self::Incoming,
            2 => Self::OutgoingInit,
            3 => Self::OutgoingProgress,
            4 => Self::OutgoingRinging,
            5 => Self::OutgoingEarlyMedia,
            6 => Self::Connected,
            7 => Self::StreamsRunning,
            8 => Self::Paused,
            9 => Self::PausedByRemote,
            10 => Self::UpdatedByRemote,
            11 => Self::Released,
            12 => Self::Error,
            13 => Self::End,
            _ => Self::Idle,
        }
    }

    pub fn as_protocol(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Incoming => "incoming",
            Self::OutgoingInit => "outgoing_init",
            Self::OutgoingProgress => "outgoing_progress",
            Self::OutgoingRinging => "outgoing_ringing",
            Self::OutgoingEarlyMedia => "outgoing_early_media",
            Self::Connected => "connected",
            Self::StreamsRunning => "streams_running",
            Self::Paused => "paused",
            Self::PausedByRemote => "paused_by_remote",
            Self::UpdatedByRemote => "updated_by_remote",
            Self::Released => "released",
            Self::Error => "error",
            Self::End => "end",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Idle | Self::Released | Self::Error | Self::End)
    }
}

impl MessageKind {
    pub fn from_native(value: i32) -> Self {
        if value == 2 {
            Self::VoiceNote
        } else {
            Self::Text
        }
    }

    pub fn as_protocol(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::VoiceNote => "voice_note",
        }
    }
}

impl MessageDirection {
    pub fn from_native(value: i32) -> Self {
        if value == 2 {
            Self::Outgoing
        } else {
            Self::Incoming
        }
    }

    pub fn as_protocol(self) -> &'static str {
        match self {
            Self::Incoming => "incoming",
            Self::Outgoing => "outgoing",
        }
    }
}

impl MessageDeliveryState {
    pub fn from_native(value: i32) -> Self {
        match value {
            1 => Self::Queued,
            2 => Self::Sending,
            3 => Self::Sent,
            4 => Self::Delivered,
            5 => Self::Failed,
            _ => Self::Failed,
        }
    }

    pub fn as_protocol(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Sending => "sending",
            Self::Sent => "sent",
            Self::Delivered => "delivered",
            Self::Failed => "failed",
        }
    }
}

pub fn registration_payload(state: RegistrationState, reason: &str) -> Value {
    json!({"state": state.as_protocol(), "reason": reason})
}

pub fn call_state_payload(call_id: &str, state: CallState) -> Value {
    json!({"call_id": call_id, "state": state.as_protocol()})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_native_registration_values_to_python_values() {
        assert_eq!(RegistrationState::from_native(1).as_protocol(), "progress");
        assert_eq!(RegistrationState::from_native(2).as_protocol(), "ok");
        assert_eq!(RegistrationState::from_native(4).as_protocol(), "failed");
        assert_eq!(RegistrationState::from_native(99).as_protocol(), "none");
    }

    #[test]
    fn maps_native_call_values_to_python_values() {
        assert_eq!(CallState::from_native(1).as_protocol(), "incoming");
        assert_eq!(CallState::from_native(7).as_protocol(), "streams_running");
        assert_eq!(CallState::from_native(11).as_protocol(), "released");
        assert_eq!(CallState::from_native(99).as_protocol(), "idle");
    }

    #[test]
    fn released_error_end_are_terminal() {
        assert!(CallState::Released.is_terminal());
        assert!(CallState::Error.is_terminal());
        assert!(CallState::End.is_terminal());
        assert!(!CallState::Connected.is_terminal());
    }

    #[test]
    fn maps_native_message_values_to_python_values() {
        assert_eq!(MessageKind::from_native(1).as_protocol(), "text");
        assert_eq!(MessageKind::from_native(2).as_protocol(), "voice_note");
        assert_eq!(MessageDirection::from_native(1).as_protocol(), "incoming");
        assert_eq!(MessageDirection::from_native(2).as_protocol(), "outgoing");
        assert_eq!(MessageDeliveryState::from_native(1).as_protocol(), "queued");
        assert_eq!(
            MessageDeliveryState::from_native(4).as_protocol(),
            "delivered"
        );
        assert_eq!(
            MessageDeliveryState::from_native(99).as_protocol(),
            "failed"
        );
    }

    #[test]
    fn event_payload_helpers_match_worker_protocol() {
        assert_eq!(
            registration_payload(RegistrationState::Ok, ""),
            serde_json::json!({"state": "ok", "reason": ""})
        );
        assert_eq!(
            call_state_payload("call-1", CallState::StreamsRunning),
            serde_json::json!({"call_id": "call-1", "state": "streams_running"})
        );
    }
}
