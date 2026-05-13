use crate::components::primitives::{container, label};
use crate::engine::{Element, Key};
use crate::roles;
use crate::scene::deck::RowModel;

pub fn list_row(row: &RowModel, accent: u32) -> Element {
    container(roles::LIST_ROW)
        .key(Key::String(row.id.clone()))
        .selected(row.selected)
        .accent(accent)
        .child(
            label(roles::LIST_ROW_ICON)
                .icon(&row.icon_key)
                .accent(accent),
        )
        .child(label(roles::LIST_ROW_TITLE).text(&row.title))
        .child(label(roles::LIST_ROW_SUBTITLE).text(&row.subtitle))
}
