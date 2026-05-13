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
    pub battery_percent: u8,
    pub signal_strength: u8,
    pub network_online: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FooterBar {
    pub text: String,
    pub accent: Option<u32>,
}

impl HudScene {
    pub fn element(&self) -> Element {
        Element::new(ElementKind::Container, Some("hud"))
            .key(Key::Static("hud"))
            .child(self.status.element())
            .child(
                FooterBar {
                    text: self.footer_text.clone(),
                    accent: None,
                }
                .element(),
            )
    }
}

impl HudStatus {
    pub fn element(&self) -> Element {
        let mut status_bar = Element::new(ElementKind::Container, Some("status_bar"))
            .key(Key::Static("status_bar"))
            .region(RegionId::StatusBar);

        for bar in signal_bars(self.signal_strength) {
            status_bar = status_bar.child(bar);
        }

        status_bar
            .child(Element::new(ElementKind::Label, Some("status_wifi")).icon("network"))
            .child(gps_ring(self.network_online))
            .child(gps_center(self.network_online))
            .child(gps_tail(self.network_online))
            .child(voip_indicator(self.network_online))
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
            .child(battery_outline(self.battery_percent))
            .child(
                Element::new(ElementKind::Container, Some("status_battery_tip"))
                    .key(Key::Static("battery_tip")),
            )
    }
}

fn signal_bars(strength: u8) -> [Element; 4] {
    core::array::from_fn(|index| {
        Element::new(ElementKind::Container, Some(signal_bar_role(index)))
            .key(Key::Indexed(index))
            .visible(index < usize::from(strength.min(4)))
    })
}

fn gps_ring(active: bool) -> Element {
    Element::new(ElementKind::Container, Some("status_gps_ring"))
        .key(Key::Static("gps_ring"))
        .selected(active)
}

fn gps_center(active: bool) -> Element {
    Element::new(ElementKind::Container, Some("status_gps_center"))
        .key(Key::Static("gps_center"))
        .visible(active)
}

fn gps_tail(active: bool) -> Element {
    Element::new(ElementKind::Container, Some("status_gps_tail"))
        .key(Key::Static("gps_tail"))
        .visible(active)
}

fn voip_indicator(active: bool) -> Element {
    Element::new(ElementKind::Container, Some("status_voip_dot_after_gps"))
        .key(Key::Static("voip_dot"))
        .visible(active)
}

fn battery_outline(percent: u8) -> Element {
    Element::new(ElementKind::Container, Some("status_battery_outline"))
        .key(Key::Static("battery_outline"))
        .child(
            Element::new(ElementKind::Container, Some("status_battery_fill"))
                .key(Key::Static("battery_fill"))
                .progress(i32::from(percent.min(100))),
        )
}

const fn signal_bar_role(index: usize) -> &'static str {
    match index {
        0 => "status_signal_bar_0",
        1 => "status_signal_bar_1",
        2 => "status_signal_bar_2",
        _ => "status_signal_bar_3",
    }
}

impl FooterBar {
    pub fn element(&self) -> Element {
        let mut label = Element::new(ElementKind::Label, Some("footer_label"))
            .key(Key::Static("footer_label"))
            .text(&self.text);
        if let Some(accent) = self.accent {
            label = label.accent(accent);
        }
        Element::new(ElementKind::Container, Some("footer_bar"))
            .key(Key::Static("footer_bar"))
            .region(RegionId::Footer)
            .child(label)
    }
}
