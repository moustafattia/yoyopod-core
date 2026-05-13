use crate::engine::Element;
use crate::scene::HudStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarProps {
    pub time: String,
    pub battery_label: String,
    pub signal_strength: u8,
    pub network_online: bool,
}

pub fn status_bar(props: &StatusBarProps) -> Element {
    HudStatus {
        time: props.time.clone(),
        battery_label: props.battery_label.clone(),
        signal_strength: props.signal_strength,
        network_online: props.network_online,
    }
    .element()
}
