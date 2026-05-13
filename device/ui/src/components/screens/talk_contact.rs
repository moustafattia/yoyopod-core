use yoyopod_protocol::ui::UiScreen;

use crate::scene::Scene;

pub fn scene(focus: usize) -> Scene {
    super::common::action_scene(UiScreen::TalkContact, focus)
}
