use crate::animation::{ClockSource, Timeline, TimelineSampler};
use crate::render_contract::{DirtyRegion, Mutation, RenderMode};
use crate::scene::{SceneGraph, SceneId};

use super::{flatten, NodeIdAlloc, Reconciler, TreeCache};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameOutcome<'a> {
    pub mode: RenderMode,
    pub mutations: &'a [Mutation],
}

#[derive(Debug, Default)]
pub struct Engine {
    pub tree_cache: TreeCache,
    pub node_alloc: NodeIdAlloc,
    pub mutations: Vec<Mutation>,
    pub timelines: Vec<Timeline>,
    animation_signature: Option<u64>,
    active_scene: Option<SceneId>,
    reconciler: Reconciler,
}

impl Engine {
    pub fn tick(
        &mut self,
        graph: &SceneGraph,
        dirty_region: Option<DirtyRegion>,
        hud_region: DirtyRegion,
        now_ms: u64,
    ) -> FrameOutcome<'_> {
        self.tick_clocks(now_ms);
        let mode = render_mode_for_dirty_region(dirty_region, hud_region);
        let mutations = self.render(graph, now_ms);
        FrameOutcome { mode, mutations }
    }

    pub fn render(&mut self, graph: &SceneGraph, now_ms: u64) -> &[Mutation] {
        self.mutations.clear();
        self.sync_scene_timelines(graph, now_ms);
        let global_ms = graph
            .global_clock
            .now_ms
            .saturating_sub(graph.global_clock.started_ms);
        let sampler = TimelineSampler::new(&self.timelines, now_ms, global_ms);
        let animation_signature = sampler.quantized_signature();
        let new_tree = flatten::flatten(graph);
        let next_ids = self.reconciler.diff(
            self.tree_cache.previous(),
            self.tree_cache.ids(),
            &new_tree,
            graph.active.stage,
            &mut self.node_alloc,
            &sampler,
            &mut self.mutations,
        );
        self.tree_cache.replace(new_tree, next_ids);
        self.animation_signature = animation_signature;
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

    pub fn animation_frame_dirty(&mut self, now_ms: u64) -> bool {
        if self.timelines.is_empty() {
            self.animation_signature = None;
            return false;
        }
        let sampler = TimelineSampler::new(&self.timelines, now_ms, now_ms);
        let next_signature = sampler.quantized_signature();
        let dirty = next_signature.is_some() && next_signature != self.animation_signature;
        self.animation_signature = next_signature;
        dirty
    }

    fn sync_scene_timelines(&mut self, graph: &SceneGraph, now_ms: u64) {
        let scene_changed = self.active_scene != Some(graph.active.id);
        if scene_changed {
            self.timelines.clear();
            self.animation_signature = None;
            self.active_scene = Some(graph.active.id);
        }

        self.timelines.retain(|active| {
            graph
                .active
                .timelines
                .iter()
                .any(|timeline| timeline.id == active.id)
        });

        for mut timeline in graph.active.timelines.clone() {
            if self.timelines.iter().any(|active| active.id == timeline.id) {
                continue;
            }
            if matches!(timeline.clock, ClockSource::SceneTime) {
                timeline.started_ms = now_ms;
            }
            self.schedule_timeline(timeline);
        }
    }
}

fn render_mode_for_dirty_region(
    region: Option<DirtyRegion>,
    hud_region: DirtyRegion,
) -> RenderMode {
    match region {
        Some(region) if region == hud_region => RenderMode::HudRegion,
        Some(region) => RenderMode::Region(region),
        None => RenderMode::FullFrame,
    }
}
