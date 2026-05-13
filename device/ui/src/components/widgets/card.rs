use crate::components::primitives::{container, image, label};
use crate::engine::Element;
use crate::scene::roles;
use crate::scene::CardModel;

pub fn card(model: &CardModel) -> Element {
    container(roles::CARD)
        .accent(model.accent)
        .child(
            image(roles::CARD_ICON)
                .icon(&model.icon_key)
                .accent(model.accent),
        )
        .child(label(roles::CARD_TITLE).text(&model.title))
        .child(label(roles::CARD_SUBTITLE).text(&model.subtitle))
}
