use crate::components::primitives::{container, label};
use crate::engine::{Element, Key};
use crate::scene::deck::RowModel;

pub fn list_row(row: &RowModel, accent: u32) -> Element {
    container("list_row")
        .key(Key::String(row.id.clone()))
        .selected(row.selected)
        .accent(accent)
        .child(label("list_row_icon").icon(&row.icon_key).accent(accent))
        .child(label("list_row_title").text(&row.title))
        .child(label("list_row_subtitle").text(&row.subtitle))
}
