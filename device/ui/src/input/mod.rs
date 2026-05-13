mod config;
mod machine;

use anyhow::Result;

use crate::hardware::ButtonDevice;

pub use config::ButtonTiming;
pub use machine::{InputEvent, OneButtonMachine};

pub fn poll_button_actions<B>(
    button: &mut B,
    machine: &mut OneButtonMachine,
    ptt_passthrough: bool,
    now_ms: u64,
) -> Result<Vec<InputEvent>>
where
    B: ButtonDevice,
{
    let pressed = button.pressed()?;
    let events = if ptt_passthrough {
        machine.observe_ptt_passthrough(pressed, now_ms)
    } else {
        machine.observe(pressed, now_ms)
    };
    Ok(events)
}
