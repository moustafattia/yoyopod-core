use crate::engine::Element;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudScene {
    root: Element,
}

impl HudScene {
    pub fn new(root: Element) -> Self {
        Self { root }
    }

    pub fn element(&self) -> Element {
        self.root.clone()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HudStatus {
    pub time: String,
    pub battery_label: String,
    pub battery_percent: u8,
    pub signal_strength: u8,
    pub network_online: bool,
}
