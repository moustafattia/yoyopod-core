use yoyopod_protocol::ui::UiScreen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub screen: UiScreen,
    pub focus_index: usize,
    pub selected_id: Option<String>,
}

impl HistoryEntry {
    pub fn new(screen: UiScreen, focus_index: usize, selected_id: Option<String>) -> Self {
        Self {
            screen,
            focus_index,
            selected_id,
        }
    }
}

pub fn push(
    stack: &mut Vec<HistoryEntry>,
    active: &mut UiScreen,
    active_focus_index: usize,
    selected_id: Option<String>,
    next: UiScreen,
) {
    if *active != next {
        stack.push(HistoryEntry::new(*active, active_focus_index, selected_id));
    }
    *active = next;
}

pub fn pop_or_hub(stack: &mut Vec<HistoryEntry>, active: &mut UiScreen) -> Option<HistoryEntry> {
    let entry = stack.pop();
    *active = entry
        .as_ref()
        .map(|entry| entry.screen)
        .unwrap_or(UiScreen::Hub);
    entry
}

pub fn pop_until<F>(
    stack: &mut Vec<HistoryEntry>,
    active: &mut UiScreen,
    should_pop: F,
) -> Option<HistoryEntry>
where
    F: Fn(UiScreen) -> bool,
{
    let mut entry = None;
    while should_pop(*active) {
        entry = pop_or_hub(stack, active);
    }
    entry
}
