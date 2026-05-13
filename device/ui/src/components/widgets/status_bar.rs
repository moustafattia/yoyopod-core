use crate::components::primitives::{container, label};
use crate::engine::Element;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarProps {
    pub time: String,
    pub battery_label: String,
    pub signal_strength: u8,
    pub network_online: bool,
}

pub fn status_bar(props: &StatusBarProps) -> Element {
    container("status_bar")
        .child(label("status_time").text(&props.time))
        .child(label("status_battery_label").text(&props.battery_label))
        .child(label("status_signal").text(props.signal_strength.to_string()))
        .child(label("status_network").selected(props.network_online))
}
