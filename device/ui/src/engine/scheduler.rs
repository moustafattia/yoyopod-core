use crate::animation::{Timeline, TimelineSampler};
use crate::scene::SceneGraph;

use super::{Mutation, NodeIdAlloc, Reconciler, TreeCache};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    FullFrame,
    HudRegion,
    Region(super::DirtyRegion),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameOutcome {
    pub mode: RenderMode,
    pub mutations: Vec<Mutation>,
}

#[derive(Debug, Default)]
pub struct Engine {
    pub tree_cache: TreeCache,
    pub node_alloc: NodeIdAlloc,
    pub mutations: Vec<Mutation>,
    pub timelines: Vec<Timeline>,
    reconciler: Reconciler,
}

impl Engine {
    pub fn render(&mut self, _graph: &SceneGraph, now_ms: u64) -> &[Mutation] {
        self.mutations.clear();
        let sampler = TimelineSampler::new(&self.timelines, now_ms, now_ms);
        let _ = sampler;
        &self.mutations
    }

    pub fn schedule_timeline(&mut self, timeline: Timeline) {
        assert!(
            self.timelines.len() < 16,
            "active UI timelines must stay within the Whisplay frame budget"
        );
        self.timelines.push(timeline);
    }

    pub fn tick_clocks(&mut self, _now_ms: u64) {
        let _ = &mut self.reconciler;
    }
}
