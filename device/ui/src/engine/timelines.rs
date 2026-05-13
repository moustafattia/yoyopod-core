use crate::animation::{Timeline, TimelineId};

const MAX_ACTIVE_TIMELINES: usize = 16;

#[derive(Debug, Default)]
pub struct ActiveTimelines {
    items: Vec<Timeline>,
}

impl ActiveTimelines {
    pub fn as_slice(&self) -> &[Timeline] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&Timeline) -> bool) {
        self.items.retain(|timeline| keep(timeline));
    }

    pub fn contains_id(&self, id: TimelineId) -> bool {
        self.items.iter().any(|timeline| timeline.id == id)
    }

    pub fn schedule(&mut self, timeline: Timeline) {
        assert!(
            self.items.len() < MAX_ACTIVE_TIMELINES,
            "active UI timelines must stay within the Whisplay frame budget"
        );
        self.items.push(timeline);
    }
}
