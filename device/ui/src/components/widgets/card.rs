use crate::components::primitives::{container, image, label};
use crate::engine::Element;
use crate::scene::CardModel;

pub fn card(model: &CardModel) -> Element {
    container("card")
        .accent(model.accent)
        .child(
            image("card_icon")
                .icon(&model.icon_key)
                .accent(model.accent),
        )
        .child(label("card_title").text(&model.title))
        .child(label("card_subtitle").text(&model.subtitle))
}
