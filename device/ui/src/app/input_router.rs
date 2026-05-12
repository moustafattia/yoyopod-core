use yoyopod_protocol::ui::InputAction;

use super::UiScreen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputRouteState {
    pub active_screen: UiScreen,
    pub voice_note_phase: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppCommand {
    AdvanceFocus,
    SelectFocused,
    GoBack,
    PttPress,
    PttRelease,
}

pub fn route(action: InputAction, _state: &InputRouteState) -> AppCommand {
    match action {
        InputAction::Advance => AppCommand::AdvanceFocus,
        InputAction::Select => AppCommand::SelectFocused,
        InputAction::Back => AppCommand::GoBack,
        InputAction::PttPress => AppCommand::PttPress,
        InputAction::PttRelease => AppCommand::PttRelease,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_protocol_input_to_app_command() {
        let state = InputRouteState {
            active_screen: UiScreen::Hub,
            voice_note_phase: "ready".to_string(),
        };

        assert_eq!(
            route(InputAction::Advance, &state),
            AppCommand::AdvanceFocus
        );
        assert_eq!(
            route(InputAction::Select, &state),
            AppCommand::SelectFocused
        );
        assert_eq!(route(InputAction::Back, &state), AppCommand::GoBack);
        assert_eq!(route(InputAction::PttPress, &state), AppCommand::PttPress);
        assert_eq!(
            route(InputAction::PttRelease, &state),
            AppCommand::PttRelease
        );
    }
}
