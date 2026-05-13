use yoyopod_protocol::ui::UiScreen;

use crate::scene::Scene;

pub fn scene(focus: usize) -> Scene {
    super::common::hero_scene(UiScreen::Talk, 0x00d4ff, 3, focus)
}
