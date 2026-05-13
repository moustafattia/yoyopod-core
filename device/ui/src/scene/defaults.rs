use std::collections::BTreeSet;
use std::sync::OnceLock;

use serde::Deserialize;
use thiserror::Error;
use yoyopod_protocol::ui::UiScreen;

use crate::animation::{presets, ActorRef, Timeline};

use super::{Backdrop, FxLayer, GlowBloom, Halo, ParticleField, PulseRing, RegionId, Stage};

const SCENES_RON: &str = include_str!("../../assets/scenes.ron");
const DEFAULT_SOLID_RGB: u32 = 0x2a2d35;
const DEFAULT_ACCENT_DRIFT_MS: u32 = 800;

static SCENE_DEFAULTS: OnceLock<SceneDefaultsCatalog> = OnceLock::new();

#[derive(Debug, Error)]
pub enum SceneDefaultsError {
    #[error("failed to parse scenes.ron: {0}")]
    Parse(#[source] ron::error::SpannedError),
    #[error("scenes.ron missing screen coverage: {0:?}")]
    MissingScreens(Vec<&'static str>),
    #[error("scenes.ron has unknown screens: {0:?}")]
    UnknownScreens(Vec<String>),
    #[error("scenes.ron has duplicate screens: {0:?}")]
    DuplicateScreens(Vec<String>),
    #[error("scenes.ron screen {screen} has unknown backdrop {value}")]
    UnknownBackdrop { screen: String, value: String },
    #[error("scenes.ron screen {screen} has unknown stage {value}")]
    UnknownStage { screen: String, value: String },
    #[error("scenes.ron screen {screen} has unknown fx preset {value}")]
    UnknownFx { screen: String, value: String },
    #[error("scenes.ron screen {screen} has unknown timeline preset {value}")]
    UnknownTimeline { screen: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneDefaultsCatalog {
    scenes: Vec<SceneDefaults>,
}

impl SceneDefaultsCatalog {
    pub fn for_screen(&self, screen: UiScreen) -> Option<&SceneDefaults> {
        self.scenes.iter().find(|scene| scene.screen == screen)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneDefaults {
    pub screen: UiScreen,
    pub backdrop: BackdropPreset,
    pub stage: Stage,
    pub fx: Vec<FxPreset>,
    pub on_enter: Option<TimelinePreset>,
}

impl SceneDefaults {
    pub fn backdrop(&self, accent: u32) -> Backdrop {
        match self.backdrop {
            BackdropPreset::Solid => Backdrop::Solid(DEFAULT_SOLID_RGB),
            BackdropPreset::AccentDrift => Backdrop::AccentDrift {
                accent,
                speed_ms: DEFAULT_ACCENT_DRIFT_MS,
            },
        }
    }

    pub fn fx_layer(&self, accent: u32) -> FxLayer {
        let mut layer = FxLayer::default();
        for preset in &self.fx {
            preset.apply(&mut layer, accent);
        }
        layer
    }

    pub fn timelines(&self) -> Vec<Timeline> {
        self.on_enter
            .map(|preset| vec![preset.timeline()])
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackdropPreset {
    Solid,
    AccentDrift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FxPreset {
    HeroHalo,
    ProgressSweep,
    VoiceMeter,
    CallPulse,
    Spinner,
}

impl FxPreset {
    fn apply(self, layer: &mut FxLayer, accent: u32) {
        match self {
            Self::HeroHalo => layer.halos.push(Halo {
                target: ActorRef::Region(RegionId::HeroIcon),
                color: accent,
                period_ms: 1_400,
                min_opacity: 48,
                max_opacity: 128,
            }),
            Self::ProgressSweep => layer.glows.push(GlowBloom {
                target: ActorRef::Region(RegionId::Progress),
                blur: 4,
                intensity: 96,
            }),
            Self::VoiceMeter => layer.particles.push(ParticleField {
                region: RegionId::ButtonRow,
                count: 6,
                color: accent,
                drift_speed_ms: 900,
            }),
            Self::CallPulse => layer.pulses.push(PulseRing {
                target: ActorRef::Region(RegionId::HeroIcon),
                color: accent,
                duration_ms: 900,
                max_radius: 56,
            }),
            Self::Spinner => layer.glows.push(GlowBloom {
                target: ActorRef::Screen,
                blur: 3,
                intensity: 80,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelinePreset {
    SceneEnter,
    StaggerEnter,
}

impl TimelinePreset {
    fn timeline(self) -> Timeline {
        match self {
            Self::SceneEnter => presets::scene_enter(),
            Self::StaggerEnter => presets::stagger_enter(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct SceneAsset {
    scenes: Vec<RawSceneDefaults>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RawSceneDefaults {
    screen: String,
    backdrop: String,
    stage: String,
    #[serde(default)]
    fx: Vec<String>,
    #[serde(default)]
    on_enter: Option<String>,
}

pub fn load_scene_defaults() -> Result<&'static SceneDefaultsCatalog, SceneDefaultsError> {
    if let Some(defaults) = SCENE_DEFAULTS.get() {
        return Ok(defaults);
    }

    let defaults = parse_scene_defaults()?;
    Ok(SCENE_DEFAULTS.get_or_init(|| defaults))
}

pub fn defaults_for(screen: UiScreen) -> SceneDefaults {
    load_scene_defaults()
        .expect("scenes.ron must be valid before rendering")
        .for_screen(screen)
        .unwrap_or_else(|| panic!("scenes.ron missing defaults for {}", screen.as_str()))
        .clone()
}

fn parse_scene_defaults() -> Result<SceneDefaultsCatalog, SceneDefaultsError> {
    let asset: SceneAsset = ron::from_str(SCENES_RON).map_err(SceneDefaultsError::Parse)?;
    validate_screen_coverage(&asset)?;
    let mut scenes = Vec::with_capacity(asset.scenes.len());
    for raw in asset.scenes {
        scenes.push(SceneDefaults {
            screen: screen_from_key(&raw.screen)
                .expect("coverage validation rejects unknown screens"),
            backdrop: backdrop_from_key(&raw.screen, &raw.backdrop)?,
            stage: stage_from_key(&raw.screen, &raw.stage)?,
            fx: raw
                .fx
                .iter()
                .map(|preset| fx_from_key(&raw.screen, preset))
                .collect::<Result<Vec<_>, _>>()?,
            on_enter: raw
                .on_enter
                .as_deref()
                .map(|preset| timeline_from_key(&raw.screen, preset))
                .transpose()?,
        });
    }
    Ok(SceneDefaultsCatalog { scenes })
}

fn validate_screen_coverage(asset: &SceneAsset) -> Result<(), SceneDefaultsError> {
    let mut screens: BTreeSet<&str> = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for scene in &asset.scenes {
        if !screens.insert(scene.screen.as_str()) {
            duplicates.insert(scene.screen.clone());
        }
    }
    if !duplicates.is_empty() {
        return Err(SceneDefaultsError::DuplicateScreens(
            duplicates.into_iter().collect(),
        ));
    }

    let required = UiScreen::ALL
        .iter()
        .map(|screen| screen.as_str())
        .collect::<BTreeSet<_>>();
    let missing = required
        .iter()
        .copied()
        .filter(|screen| !screens.contains(screen))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(SceneDefaultsError::MissingScreens(missing));
    }

    let unknown = screens
        .into_iter()
        .filter(|screen| !required.contains(screen))
        .map(str::to_string)
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(SceneDefaultsError::UnknownScreens(unknown));
    }

    Ok(())
}

fn screen_from_key(key: &str) -> Option<UiScreen> {
    UiScreen::ALL
        .iter()
        .copied()
        .find(|screen| screen.as_str() == key)
}

fn backdrop_from_key(screen: &str, key: &str) -> Result<BackdropPreset, SceneDefaultsError> {
    match key {
        "solid" => Ok(BackdropPreset::Solid),
        "accent_drift" => Ok(BackdropPreset::AccentDrift),
        value => Err(SceneDefaultsError::UnknownBackdrop {
            screen: screen.to_string(),
            value: value.to_string(),
        }),
    }
}

fn stage_from_key(screen: &str, key: &str) -> Result<Stage, SceneDefaultsError> {
    match key {
        "centered_hero_icon" => Ok(Stage::CenteredHeroIcon),
        "list_with_chrome" => Ok(Stage::ListWithChrome),
        "now_playing_panel" => Ok(Stage::NowPlayingPanel),
        "call_panel" => Ok(Stage::CallPanel),
        "talk_actions_grid" => Ok(Stage::TalkActionsGrid),
        "paged_detail" => Ok(Stage::PagedDetail),
        "overlay_center" => Ok(Stage::OverlayCenter),
        value => Err(SceneDefaultsError::UnknownStage {
            screen: screen.to_string(),
            value: value.to_string(),
        }),
    }
}

fn fx_from_key(screen: &str, key: &str) -> Result<FxPreset, SceneDefaultsError> {
    match key {
        "hero_halo" => Ok(FxPreset::HeroHalo),
        "progress_sweep" => Ok(FxPreset::ProgressSweep),
        "voice_meter" => Ok(FxPreset::VoiceMeter),
        "call_pulse" => Ok(FxPreset::CallPulse),
        "spinner" => Ok(FxPreset::Spinner),
        value => Err(SceneDefaultsError::UnknownFx {
            screen: screen.to_string(),
            value: value.to_string(),
        }),
    }
}

fn timeline_from_key(screen: &str, key: &str) -> Result<TimelinePreset, SceneDefaultsError> {
    match key {
        "scene_enter" => Ok(TimelinePreset::SceneEnter),
        "stagger_enter" => Ok(TimelinePreset::StaggerEnter),
        value => Err(SceneDefaultsError::UnknownTimeline {
            screen: screen.to_string(),
            value: value.to_string(),
        }),
    }
}
