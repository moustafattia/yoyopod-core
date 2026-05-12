use yoyopod_protocol::ui::AnimationRequest;

use crate::app::UiScreen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionTarget {
    Screen(UiScreen),
    Selection { screen: UiScreen, index: usize },
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionProperty {
    Opacity,
    OffsetY,
    ScalePermille,
    SelectionOffset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    Linear,
    EaseOut,
    EaseInOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub id: String,
    pub target: TransitionTarget,
    pub property: TransitionProperty,
    pub easing: Easing,
    pub from: i32,
    pub to: i32,
    pub duration_ms: u64,
    pub started_at_ms: u64,
}

impl Transition {
    pub fn from_request(
        request: AnimationRequest,
        screen: UiScreen,
        focus_index: usize,
        started_at_ms: u64,
    ) -> Self {
        let id = if request.transition_id.trim().is_empty() {
            "screen_enter".to_string()
        } else {
            request.transition_id
        };
        let target = if id.starts_with("selection") {
            TransitionTarget::Selection {
                screen,
                index: focus_index,
            }
        } else if id.starts_with("runtime") {
            TransitionTarget::Runtime
        } else {
            TransitionTarget::Screen(screen)
        };
        let property = if id.contains("selection") {
            TransitionProperty::SelectionOffset
        } else if id.contains("scale") {
            TransitionProperty::ScalePermille
        } else if id.contains("fade") {
            TransitionProperty::Opacity
        } else {
            TransitionProperty::OffsetY
        };
        let (from, to) = match property {
            TransitionProperty::Opacity => (0, 255),
            TransitionProperty::ScalePermille => (960, 1000),
            TransitionProperty::OffsetY | TransitionProperty::SelectionOffset => (8, 0),
        };
        Self {
            id,
            target,
            property,
            easing: Easing::EaseOut,
            from,
            to,
            duration_ms: request.duration_ms.max(120),
            started_at_ms,
        }
    }

    pub fn is_complete(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.started_at_ms) >= self.duration_ms
    }

    pub fn interpolated_value(&self, now_ms: u64) -> i32 {
        let elapsed = now_ms
            .saturating_sub(self.started_at_ms)
            .min(self.duration_ms);
        let progress_permille = if self.duration_ms == 0 {
            1000
        } else {
            ((elapsed * 1000) / self.duration_ms) as i32
        };
        let eased = eased_progress(progress_permille, self.easing);
        self.from + (((self.to - self.from) * eased) / 1000)
    }
}

fn eased_progress(progress_permille: i32, easing: Easing) -> i32 {
    let t = progress_permille.clamp(0, 1000);
    match easing {
        Easing::Linear => t,
        Easing::EaseOut => 1000 - (((1000 - t) * (1000 - t)) / 1000),
        Easing::EaseInOut => {
            if t < 500 {
                (2 * t * t) / 1000
            } else {
                1000 - (2 * (1000 - t) * (1000 - t)) / 1000
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_creates_typed_selection_transition() {
        let transition = Transition::from_request(
            AnimationRequest {
                transition_id: "selection_move".to_string(),
                duration_ms: 200,
            },
            UiScreen::Listen,
            2,
            1000,
        );
        assert_eq!(
            transition.target,
            TransitionTarget::Selection {
                screen: UiScreen::Listen,
                index: 2
            }
        );
        assert_eq!(transition.property, TransitionProperty::SelectionOffset);
        assert_eq!(transition.duration_ms, 200);
    }

    #[test]
    fn transition_interpolates_and_completes() {
        let transition = Transition::from_request(
            AnimationRequest {
                transition_id: "screen_fade".to_string(),
                duration_ms: 200,
            },
            UiScreen::Hub,
            0,
            1000,
        );
        assert_eq!(transition.interpolated_value(1000), 0);
        assert!(transition.interpolated_value(1100) > 0);
        assert_eq!(transition.interpolated_value(1200), 255);
        assert!(transition.is_complete(1200));
    }
}
