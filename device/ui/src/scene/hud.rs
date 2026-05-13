use crate::engine::{Element, Key};
use crate::render_contract::ElementKind;
use crate::roles;
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
        Element::new(ElementKind::Container, Some(roles::HUD))
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
        let mut status_bar = Element::new(ElementKind::Container, Some(roles::STATUS_BAR))
            .key(Key::Static("status_bar"))
            .region(RegionId::StatusBar);

        for bar in signal_bars(self.signal_strength) {
            status_bar = status_bar.child(bar);
        }

        status_bar
            .child(Element::new(ElementKind::Label, Some(roles::STATUS_WIFI)).icon("network"))
            .child(gps_ring(self.network_online))
            .child(gps_center(self.network_online))
            .child(gps_tail(self.network_online))
            .child(voip_indicator(self.network_online))
            .child(
                Element::new(ElementKind::Label, Some(roles::STATUS_TIME))
                    .key(Key::Static("status_time"))
                    .text(&self.time),
            )
            .child(
                Element::new(ElementKind::Label, Some(roles::STATUS_BATTERY_LABEL))
                    .key(Key::Static("status_battery_label"))
                    .text(&self.battery_label),
            )
            .child(battery_outline(self.battery_percent))
            .child(
                Element::new(ElementKind::Container, Some(roles::STATUS_BATTERY_TIP))
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
    Element::new(ElementKind::Container, Some(roles::STATUS_GPS_RING))
        .key(Key::Static("gps_ring"))
        .selected(active)
}

fn gps_center(active: bool) -> Element {
    Element::new(ElementKind::Container, Some(roles::STATUS_GPS_CENTER))
        .key(Key::Static("gps_center"))
        .visible(active)
}

fn gps_tail(active: bool) -> Element {
    Element::new(ElementKind::Container, Some(roles::STATUS_GPS_TAIL))
        .key(Key::Static("gps_tail"))
        .visible(active)
}

fn voip_indicator(active: bool) -> Element {
    Element::new(
        ElementKind::Container,
        Some(roles::STATUS_VOIP_DOT_AFTER_GPS),
    )
    .key(Key::Static("voip_dot"))
    .visible(active)
}

fn battery_outline(percent: u8) -> Element {
    Element::new(ElementKind::Container, Some(roles::STATUS_BATTERY_OUTLINE))
        .key(Key::Static("battery_outline"))
        .child(
            Element::new(ElementKind::Container, Some(roles::STATUS_BATTERY_FILL))
                .key(Key::Static("battery_fill"))
                .progress(i32::from(percent.min(100))),
        )
}

const fn signal_bar_role(index: usize) -> &'static str {
    match index {
        0 => roles::STATUS_SIGNAL_BAR_0,
        1 => roles::STATUS_SIGNAL_BAR_1,
        2 => roles::STATUS_SIGNAL_BAR_2,
        _ => roles::STATUS_SIGNAL_BAR_3,
    }
}

impl FooterBar {
    pub fn element(&self) -> Element {
        let mut label = Element::new(ElementKind::Label, Some(roles::FOOTER_LABEL))
            .key(Key::Static("footer_label"))
            .text(&self.text);
        if let Some(accent) = self.accent {
            label = label.accent(accent);
        }
        Element::new(ElementKind::Container, Some(roles::FOOTER_BAR))
            .key(Key::Static("footer_bar"))
            .region(RegionId::Footer)
            .child(label)
    }
}
