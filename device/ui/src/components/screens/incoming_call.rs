use yoyopod_protocol::ui::UiScreen;

use crate::scene::Scene;

pub fn scene() -> Scene {
    super::common::call_scene(UiScreen::IncomingCall)
}
