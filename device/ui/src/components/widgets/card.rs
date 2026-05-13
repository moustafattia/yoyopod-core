use crate::components::primitives::label;
use crate::engine::Element;
use crate::scene::roles;
use crate::scene::CardModel;

use super::{icon_halo, IconHaloProps};

pub fn card(model: &CardModel) -> Element {
    icon_halo(&IconHaloProps {
        halo_role: roles::CARD,
        icon_role: roles::CARD_ICON,
        icon_key: model.icon_key.clone(),
        accent: model.accent,
    })
    .child(label(roles::CARD_TITLE).text(&model.title))
    .child(label(roles::CARD_SUBTITLE).text(&model.subtitle))
}
