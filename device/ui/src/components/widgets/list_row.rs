use crate::components::primitives::{container, image, label};
use crate::engine::{Element, Key};
use crate::scene::deck::RowModel;
use crate::scene::roles;

pub fn list_row(row: &RowModel, selected: bool, key: Key) -> Element {
    container(roles::LIST_ROW)
        .key(key)
        .selected(row.selected || selected)
        .child(image(roles::LIST_ROW_ICON).icon(&row.icon_key))
        .child(label(roles::LIST_ROW_TITLE).text(&row.title))
        .child(label(roles::LIST_ROW_SUBTITLE).text(&row.subtitle))
}
