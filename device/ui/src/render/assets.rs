use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use thiserror::Error;

use crate::render::widgets::roles;

const LAYOUTS_RON: &str = include_str!("../../assets/layouts.ron");
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderAssets {
    layouts: BTreeMap<String, LayoutRole>,
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
}

pub fn load_render_assets() -> Result<RenderAssets, RenderAssetError> {
    let layouts = parse_layout_asset()?;
    let theme = parse_theme_asset()?;
    Ok(RenderAssets {
        layouts: layouts
            .roles
            .into_iter()
            .map(|role| (role.role.clone(), role))
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

pub fn parse_theme_asset() -> Result<ThemeAsset, RenderAssetError> {
    let asset = ron::from_str(THEME_RON).map_err(|source| RenderAssetError::Parse {
        asset: "theme.ron",
        source,
    })?;
    validate_theme_asset(&asset)?;
    Ok(asset)
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
        roles::ASK_FOOTER,
        roles::ASK_ICON,
        roles::ASK_ICON_GLOW,
        roles::ASK_ICON_HALO,
        roles::ASK_SUBTITLE,
        roles::ASK_TITLE,
        roles::CALL_FOOTER,
        roles::CALL_ICON_HALO,
        roles::CALL_MUTE_BADGE,
        roles::CALL_MUTE_LABEL,
        roles::CALL_PANEL,
        roles::CALL_STATE_CHIP,
        roles::CALL_STATE_ICON,
        roles::CALL_STATE_LABEL,
        roles::CALL_TITLE,
        roles::FOOTER_BAR,
        roles::HUB_CARD_PANEL,
        roles::HUB_DOT,
        roles::HUB_FOOTER,
        roles::HUB_ICON,
        roles::HUB_ICON_GLOW,
        roles::HUB_SUBTITLE,
        roles::HUB_TITLE,
        roles::LIST_FOOTER,
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
        roles::LISTEN_FOOTER,
        roles::LISTEN_PANEL,
        roles::LISTEN_ROW,
        roles::LISTEN_ROW_ICON,
        roles::LISTEN_ROW_SUBTITLE,
        roles::LISTEN_ROW_TITLE,
        roles::LISTEN_SUBTITLE,
        roles::LISTEN_TITLE,
        roles::NOW_PLAYING_ARTIST,
        roles::NOW_PLAYING_FOOTER,
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
        roles::OVERLAY_FOOTER,
        roles::PLAYLIST_EMPTY_ICON,
        roles::PLAYLIST_EMPTY_PANEL,
        roles::PLAYLIST_EMPTY_SUBTITLE,
        roles::PLAYLIST_EMPTY_TITLE,
        roles::PLAYLIST_FOOTER,
        roles::PLAYLIST_PANEL,
        roles::PLAYLIST_ROW,
        roles::PLAYLIST_ROW_ICON,
        roles::PLAYLIST_ROW_SUBTITLE,
        roles::PLAYLIST_ROW_TITLE,
        roles::PLAYLIST_TITLE,
        roles::PLAYLIST_UNDERLINE,
        roles::POWER_DOT,
        roles::POWER_FOOTER,
        roles::POWER_ICON,
        roles::POWER_ICON_HALO,
        roles::POWER_ROW,
        roles::POWER_ROW_TITLE,
        roles::POWER_TITLE,
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
        roles::TALK_ACTIONS_BUTTON_LABEL,
        roles::TALK_ACTIONS_FOOTER,
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
        roles::TALK_FOOTER,
        roles::TALK_TITLE,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_asset_parses_and_covers_initial_roles() {
        let asset = parse_layout_asset().unwrap();
        assert!(asset
            .roles
            .iter()
            .any(|role| role.role == roles::OVERLAY_TITLE));
    }

    #[test]
    fn theme_asset_parses_and_covers_initial_roles() {
        let asset = parse_theme_asset().unwrap();
        assert!(asset
            .roles
            .iter()
            .any(|role| role.role == roles::OVERLAY_TITLE));
        assert!(asset
            .selected_roles
            .iter()
            .any(|role| role.role == roles::LIST_ROW));
    }

    #[test]
    fn render_assets_provide_role_lookups() {
        let assets = load_render_assets().unwrap();
        assert_eq!(
            assets.layout_role(roles::OVERLAY_TITLE).map(|role| role.y),
            Some(96)
        );
        assert_eq!(
            assets
                .theme_role(roles::OVERLAY_SUBTITLE)
                .and_then(|role| role.text_rgb),
            Some(crate::render::styling::style::MUTED_RGB)
        );
    }

    #[test]
    fn layout_asset_rejects_unknown_roles() {
        let mut asset = parse_layout_asset().unwrap();
        asset.roles.push(LayoutRole {
            role: "not_a_real_role".to_string(),
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            repeat_x: None,
            repeat_y: None,
        });
        assert!(matches!(
            validate_layout_asset(&asset),
            Err(RenderAssetError::UnknownRoles { .. })
        ));
    }

    #[test]
    fn theme_asset_rejects_duplicate_roles() {
        let mut asset = parse_theme_asset().unwrap();
        let duplicate = asset.roles[0].clone();
        asset.roles.push(duplicate);
        assert!(matches!(
            validate_theme_asset(&asset),
            Err(RenderAssetError::DuplicateRoles { .. })
        ));
    }
}
