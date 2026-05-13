use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use thiserror::Error;
use yoyopod_protocol::ui::UiScreen;

use crate::renderer::widgets::roles;

const LAYOUTS_RON: &str = include_str!("../../assets/layouts.ron");
const SCENES_RON: &str = include_str!("../../assets/scenes.ron");
const THEME_RON: &str = include_str!("../../assets/theme.ron");

#[derive(Debug, Error)]
pub enum RenderAssetError {
    #[error("failed to parse {asset}: {source}")]
    Parse {
        asset: &'static str,
        #[source]
        source: ron::error::SpannedError,
    },
    #[error("{asset} missing role coverage: {roles:?}")]
    MissingRoles {
        asset: &'static str,
        roles: Vec<&'static str>,
    },
    #[error("{asset} has unknown roles: {roles:?}")]
    UnknownRoles {
        asset: &'static str,
        roles: Vec<String>,
    },
    #[error("{asset} has duplicate roles: {roles:?}")]
    DuplicateRoles {
        asset: &'static str,
        roles: Vec<String>,
    },
    #[error("{asset} missing screen coverage: {screens:?}")]
    MissingScreens {
        asset: &'static str,
        screens: Vec<&'static str>,
    },
    #[error("{asset} has unknown screens: {screens:?}")]
    UnknownScreens {
        asset: &'static str,
        screens: Vec<String>,
    },
    #[error("{asset} has duplicate screens: {screens:?}")]
    DuplicateScreens {
        asset: &'static str,
        screens: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LayoutAsset {
    pub roles: Vec<LayoutRole>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LayoutRole {
    pub role: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    #[serde(default)]
    pub repeat_x: Option<i32>,
    #[serde(default)]
    pub repeat_y: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ThemeAsset {
    pub roles: Vec<ThemeRole>,
    #[serde(default)]
    pub selected_roles: Vec<ThemeRole>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ThemeRole {
    pub role: String,
    #[serde(default)]
    pub fill_rgb: Option<u32>,
    #[serde(default)]
    pub text_rgb: Option<u32>,
    #[serde(default)]
    pub opacity: Option<u8>,
    #[serde(default)]
    pub border_rgb: Option<u32>,
    #[serde(default)]
    pub border_width: i32,
    #[serde(default)]
    pub radius: i32,
    #[serde(default)]
    pub outline_width: i32,
    #[serde(default)]
    pub shadow_width: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SceneAsset {
    pub scenes: Vec<SceneDefaults>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SceneDefaults {
    pub screen: String,
    pub backdrop: String,
    pub stage: String,
    #[serde(default)]
    pub fx: Vec<String>,
    #[serde(default)]
    pub on_enter: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderAssets {
    layouts: BTreeMap<String, LayoutRole>,
    scenes: BTreeMap<String, SceneDefaults>,
    theme: BTreeMap<String, ThemeRole>,
    selected_theme: BTreeMap<String, ThemeRole>,
}

impl RenderAssets {
    pub fn layout_role(&self, role: &str) -> Option<&LayoutRole> {
        self.layouts.get(role)
    }

    pub fn theme_role(&self, role: &str) -> Option<&ThemeRole> {
        self.theme.get(role)
    }

    pub fn selected_theme_role(&self, role: &str) -> Option<&ThemeRole> {
        self.selected_theme.get(role)
    }

    pub fn scene_defaults(&self, screen: UiScreen) -> Option<&SceneDefaults> {
        self.scenes.get(screen.as_str())
    }
}

pub fn load_render_assets() -> Result<RenderAssets, RenderAssetError> {
    let layouts = parse_layout_asset()?;
    let scenes = parse_scene_asset()?;
    let theme = parse_theme_asset()?;
    Ok(RenderAssets {
        layouts: layouts
            .roles
            .into_iter()
            .map(|role| (role.role.clone(), role))
            .collect(),
        scenes: scenes
            .scenes
            .into_iter()
            .map(|scene| (scene.screen.clone(), scene))
            .collect(),
        theme: theme
            .roles
            .into_iter()
            .map(|role| (role.role.clone(), role))
            .collect(),
        selected_theme: theme
            .selected_roles
            .into_iter()
            .map(|role| (role.role.clone(), role))
            .collect(),
    })
}

pub fn parse_layout_asset() -> Result<LayoutAsset, RenderAssetError> {
    let asset = ron::from_str(LAYOUTS_RON).map_err(|source| RenderAssetError::Parse {
        asset: "layouts.ron",
        source,
    })?;
    validate_layout_asset(&asset)?;
    Ok(asset)
}

pub fn parse_scene_asset() -> Result<SceneAsset, RenderAssetError> {
    let asset = ron::from_str(SCENES_RON).map_err(|source| RenderAssetError::Parse {
        asset: "scenes.ron",
        source,
    })?;
    validate_scene_asset(&asset)?;
    Ok(asset)
}

pub fn parse_theme_asset() -> Result<ThemeAsset, RenderAssetError> {
    let asset = ron::from_str(THEME_RON).map_err(|source| RenderAssetError::Parse {
        asset: "theme.ron",
        source,
    })?;
    validate_theme_asset(&asset)?;
    Ok(asset)
}

pub fn validate_scene_asset(asset: &SceneAsset) -> Result<(), RenderAssetError> {
    let mut screens: BTreeSet<&str> = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for scene in &asset.scenes {
        if !screens.insert(scene.screen.as_str()) {
            duplicates.insert(scene.screen.clone());
        }
    }
    if !duplicates.is_empty() {
        return Err(RenderAssetError::DuplicateScreens {
            asset: "scenes.ron",
            screens: duplicates.into_iter().collect(),
        });
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
        return Err(RenderAssetError::MissingScreens {
            asset: "scenes.ron",
            screens: missing,
        });
    }

    let unknown = screens
        .into_iter()
        .filter(|screen| !required.contains(screen))
        .map(str::to_string)
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(RenderAssetError::UnknownScreens {
            asset: "scenes.ron",
            screens: unknown,
        });
    }

    Ok(())
}

pub fn validate_layout_asset(asset: &LayoutAsset) -> Result<(), RenderAssetError> {
    validate_role_coverage(
        "layouts.ron",
        required_layout_roles(),
        asset.roles.iter().map(|role| role.role.as_str()),
    )
}

pub fn validate_theme_asset(asset: &ThemeAsset) -> Result<(), RenderAssetError> {
    validate_role_coverage(
        "theme.ron",
        required_theme_roles(),
        asset.roles.iter().map(|role| role.role.as_str()),
    )?;
    validate_role_coverage(
        "theme.ron selected_roles",
        required_selected_theme_roles(),
        asset.selected_roles.iter().map(|role| role.role.as_str()),
    )
}

fn validate_role_coverage<'a>(
    asset: &'static str,
    required_roles: Vec<&'static str>,
    role_iter: impl IntoIterator<Item = &'a str>,
) -> Result<(), RenderAssetError> {
    let mut roles: BTreeSet<&str> = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for role in role_iter {
        if !roles.insert(role) {
            duplicates.insert(role.to_string());
        }
    }
    if !duplicates.is_empty() {
        return Err(RenderAssetError::DuplicateRoles {
            asset,
            roles: duplicates.into_iter().collect(),
        });
    }

    let required = required_roles.into_iter().collect::<BTreeSet<_>>();
    let missing = required
        .iter()
        .copied()
        .into_iter()
        .filter(|role| !roles.contains(role))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(RenderAssetError::MissingRoles {
            asset,
            roles: missing,
        });
    }

    let unknown = roles
        .into_iter()
        .filter(|role| !required.contains(role))
        .map(str::to_string)
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(RenderAssetError::UnknownRoles {
            asset,
            roles: unknown,
        });
    }

    Ok(())
}

fn required_layout_roles() -> Vec<&'static str> {
    let mut roles = vec![
        roles::ASK_ICON,
        roles::ASK_ICON_GLOW,
        roles::ASK_ICON_HALO,
        roles::ASK_SUBTITLE,
        roles::ASK_TITLE,
        roles::BUTTON,
        roles::BUTTON_ICON,
        roles::BUTTON_TITLE,
        roles::CALL_ICON_HALO,
        roles::CALL_MUTE_BADGE,
        roles::CALL_MUTE_LABEL,
        roles::CALL_PANEL,
        roles::CALL_STATE_CHIP,
        roles::CALL_STATE_ICON,
        roles::CALL_STATE_LABEL,
        roles::CALL_TITLE,
        roles::CARD,
        roles::CARD_ICON,
        roles::CARD_SUBTITLE,
        roles::CARD_TITLE,
        roles::CURSOR_DOTS,
        roles::CURSOR_ROW_GLOW,
        roles::DECK_BUTTONS,
        roles::DECK_CARD_ROW,
        roles::DECK_GRID,
        roles::DECK_LIST,
        roles::DECK_PAGE,
        roles::DECK_REGION,
        roles::FOOTER_BAR,
        roles::FOOTER_LABEL,
        roles::HUD,
        roles::HUB_CARD_PANEL,
        roles::HUB_DOT,
        roles::HUB_ICON,
        roles::HUB_ICON_GLOW,
        roles::HUB_SUBTITLE,
        roles::HUB_TITLE,
        roles::LIST_ROW,
        roles::LIST_ROW_ICON,
        roles::LIST_ROW_SUBTITLE,
        roles::LIST_ROW_TITLE,
        roles::LIST_SUBTITLE,
        roles::LIST_TITLE,
        roles::LISTEN_EMPTY_ICON,
        roles::LISTEN_EMPTY_PANEL,
        roles::LISTEN_EMPTY_SUBTITLE,
        roles::LISTEN_EMPTY_TITLE,
        roles::LISTEN_PANEL,
        roles::LISTEN_ROW,
        roles::LISTEN_ROW_ICON,
        roles::LISTEN_ROW_SUBTITLE,
        roles::LISTEN_ROW_TITLE,
        roles::LISTEN_SUBTITLE,
        roles::LISTEN_TITLE,
        roles::NOW_PLAYING_ARTIST,
        roles::NOW_PLAYING_ICON_HALO,
        roles::NOW_PLAYING_ICON_LABEL,
        roles::NOW_PLAYING_PANEL,
        roles::NOW_PLAYING_PROGRESS_FILL,
        roles::NOW_PLAYING_PROGRESS_TRACK,
        roles::NOW_PLAYING_STATE_CHIP,
        roles::NOW_PLAYING_STATE_LABEL,
        roles::NOW_PLAYING_TITLE,
        roles::OVERLAY_TITLE,
        roles::OVERLAY_SUBTITLE,
        roles::MODAL,
        roles::MODAL_MESSAGE,
        roles::MODAL_STACK,
        roles::MODAL_TITLE,
        roles::PAGE,
        roles::PAGE_BODY,
        roles::PAGE_TITLE,
        roles::PLAYLIST_EMPTY_ICON,
        roles::PLAYLIST_EMPTY_PANEL,
        roles::PLAYLIST_EMPTY_SUBTITLE,
        roles::PLAYLIST_EMPTY_TITLE,
        roles::PLAYLIST_PANEL,
        roles::PLAYLIST_ROW,
        roles::PLAYLIST_ROW_ICON,
        roles::PLAYLIST_ROW_SUBTITLE,
        roles::PLAYLIST_ROW_TITLE,
        roles::PLAYLIST_TITLE,
        roles::PLAYLIST_UNDERLINE,
        roles::POWER_DOT,
        roles::POWER_ICON,
        roles::POWER_ICON_HALO,
        roles::POWER_ROW,
        roles::POWER_ROW_TITLE,
        roles::POWER_TITLE,
        roles::PROGRESS_SWEEP,
        roles::SCENE_BACKDROP,
        roles::SCENE_DECKS,
        roles::SCENE_GRAPH,
        roles::SCENE_ROOT,
        roles::SCENE_STAGE,
        roles::STATUS_BAR,
        roles::STATUS_WIFI,
        roles::STATUS_GPS_RING,
        roles::STATUS_GPS_CENTER,
        roles::STATUS_GPS_TAIL,
        roles::STATUS_VOIP_DOT_LEFT,
        roles::STATUS_VOIP_DOT_AFTER_GPS,
        roles::STATUS_TIME,
        roles::STATUS_BATTERY_OUTLINE,
        roles::STATUS_BATTERY_FILL,
        roles::STATUS_BATTERY_TIP,
        roles::STATUS_BATTERY_LABEL,
        roles::STATUS_NETWORK,
        roles::STATUS_SIGNAL,
        roles::TALK_ACTIONS_BUTTON_LABEL,
        roles::TALK_ACTIONS_HEADER_BOX,
        roles::TALK_ACTIONS_HEADER_LABEL,
        roles::TALK_ACTIONS_HEADER_NAME,
        roles::TALK_ACTIONS_PRIMARY_BUTTON,
        roles::TALK_ACTIONS_STATUS_LABEL,
        roles::TALK_ACTIONS_TITLE_LABEL,
        roles::TALK_CARD_GLOW,
        roles::TALK_CARD_LABEL,
        roles::TALK_CARD_PANEL,
        roles::TALK_DOT,
        roles::TALK_TITLE,
        roles::VOICE_METER,
    ];
    roles.extend(roles::STATUS_SIGNAL_BARS);
    roles
}

fn required_theme_roles() -> Vec<&'static str> {
    let mut roles = required_layout_roles();
    roles.push(roles::ROOT);
    roles.sort_unstable();
    roles.dedup();
    roles
}

fn required_selected_theme_roles() -> Vec<&'static str> {
    vec![
        roles::HUB_DOT,
        roles::LIST_ROW,
        roles::LIST_ROW_SUBTITLE,
        roles::LIST_ROW_TITLE,
        roles::LISTEN_ROW,
        roles::LISTEN_ROW_SUBTITLE,
        roles::LISTEN_ROW_TITLE,
        roles::PLAYLIST_ROW,
        roles::PLAYLIST_ROW_SUBTITLE,
        roles::PLAYLIST_ROW_TITLE,
        roles::POWER_DOT,
        roles::TALK_DOT,
    ]
}
