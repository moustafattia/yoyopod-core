use std::io::Write;
use std::time::Duration;

use anyhow::Result;
use yoyopod_protocol::ui::{
    DisplayInfo, UiError, UiErrorCode, UiEvent, UiReady, UI_SCHEMA_VERSION,
};

use crate::router;

use super::outbound::emit_event;

pub(super) fn emit_ready<W: Write>(output: &mut W, width: usize, height: usize) -> Result<()> {
    emit_event(
        output,
        UiEvent::Ready(UiReady {
            display: DisplayInfo { width, height },
            schema_version: UI_SCHEMA_VERSION,
            screens: router::screen_capabilities(),
        }),
    )
}

pub(super) fn emit_shutdown_complete<W: Write>(output: &mut W) -> Result<()> {
    emit_event(output, UiEvent::ShutdownComplete)
}

pub(super) fn emit_manager_timeout<W, E>(
    output: &mut W,
    errors: &mut E,
    timeout: Duration,
) -> Result<()>
where
    W: Write,
    E: Write,
{
    let message = format!(
        "runtime manager heartbeat timed out after {}ms",
        timeout.as_millis()
    );
    writeln!(errors, "{message}")?;
    emit_event(
        output,
        UiEvent::Error(UiError::new(UiErrorCode::ManagerTimeout, message)),
    )
}
