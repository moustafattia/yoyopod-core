use crate::engine::{Element, Key};
use crate::render_contract::ElementKind;
use crate::scene::RegionId;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HudScene {
    pub status: HudStatus,
    pub footer_text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HudStatus {
    pub time: String,
    pub battery_label: String,
    pub signal_strength: u8,
    pub network_online: bool,
}

impl HudScene {
    pub fn element(&self) -> Element {
        Element::new(ElementKind::Container, Some("hud"))
            .key(Key::Static("hud"))
            .child(self.status.element())
            .child(self.footer_element())
    }

    fn footer_element(&self) -> Element {
        Element::new(ElementKind::Container, Some("footer_bar"))
            .key(Key::Static("footer_bar"))
            .region(RegionId::Footer)
            .child(
                Element::new(ElementKind::Label, Some("footer_label"))
                    .key(Key::Static("footer_label"))
                    .text(&self.footer_text),
            )
    }
}

impl HudStatus {
    fn element(&self) -> Element {
        Element::new(ElementKind::Container, Some("status_bar"))
            .key(Key::Static("status_bar"))
            .region(RegionId::StatusBar)
            .child(
                Element::new(ElementKind::Label, Some("status_signal"))
                    .key(Key::Static("status_signal"))
                    .text(self.signal_strength.to_string()),
            )
            .child(
                Element::new(ElementKind::Label, Some("status_network"))
                    .key(Key::Static("status_network"))
                    .selected(self.network_online),
            )
            .child(
                Element::new(ElementKind::Label, Some("status_time"))
                    .key(Key::Static("status_time"))
                    .text(&self.time),
            )
            .child(
                Element::new(ElementKind::Label, Some("status_battery_label"))
                    .key(Key::Static("status_battery_label"))
                    .text(&self.battery_label),
            )
    }
}
