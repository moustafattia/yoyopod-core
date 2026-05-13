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
