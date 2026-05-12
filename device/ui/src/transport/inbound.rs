use yoyopod_protocol::ui::UiCommand;
use yoyopod_protocol::{ProtocolError, WorkerEnvelope};

pub(super) fn decode_command(envelope: WorkerEnvelope) -> Result<UiCommand, ProtocolError> {
    UiCommand::from_envelope(envelope)
}
