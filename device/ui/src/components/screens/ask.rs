use yoyopod_protocol::ui::UiScreen;

use crate::scene::Scene;

pub fn scene(focus: usize) -> Scene {
    super::common::hero_scene(UiScreen::Ask, 0xc79bff, 1, focus)
}
