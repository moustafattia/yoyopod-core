use yoyopod_protocol::ui::{ListItemSnapshot, UiScreen};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiView {
    pub screen: UiScreen,
    pub title: String,
    pub subtitle: String,
    pub footer: String,
    pub items: Vec<ListItemSnapshot>,
    pub focus_index: usize,
}
