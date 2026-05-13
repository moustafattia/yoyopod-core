use yoyopod_protocol::ui::UiScreen;

pub fn push(stack: &mut Vec<UiScreen>, active: &mut UiScreen, next: UiScreen) {
    if *active != next {
        stack.push(*active);
    }
    *active = next;
}

pub fn pop_or_hub(stack: &mut Vec<UiScreen>, active: &mut UiScreen) {
    *active = stack.pop().unwrap_or(UiScreen::Hub);
}

pub fn pop_until<F>(stack: &mut Vec<UiScreen>, active: &mut UiScreen, should_pop: F)
where
    F: Fn(UiScreen) -> bool,
{
    while should_pop(*active) {
        *active = stack.pop().unwrap_or(UiScreen::Hub);
    }
}
