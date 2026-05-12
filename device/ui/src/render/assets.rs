use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use thiserror::Error;

use crate::render::lvgl::roles;

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
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ThemeAsset {
    pub roles: Vec<ThemeRole>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ThemeRole {
    pub role: String,
    pub fill_rgb: Option<u32>,
    pub text_rgb: Option<u32>,
    pub opacity: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderAssets {
    layouts: BTreeMap<String, LayoutRole>,
    theme: BTreeMap<String, ThemeRole>,
}

impl RenderAssets {
    pub fn layout_role(&self, role: &str) -> Option<&LayoutRole> {
        self.layouts.get(role)
    }

    pub fn theme_role(&self, role: &str) -> Option<&ThemeRole> {
        self.theme.get(role)
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
        asset.roles.iter().map(|role| role.role.as_str()),
    )
}

pub fn validate_theme_asset(asset: &ThemeAsset) -> Result<(), RenderAssetError> {
    validate_role_coverage(
        "theme.ron",
        asset.roles.iter().map(|role| role.role.as_str()),
    )
}

fn validate_role_coverage<'a>(
    asset: &'static str,
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

    let required = required_roles().into_iter().collect::<BTreeSet<_>>();
    let missing = required_roles()
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

fn required_roles() -> Vec<&'static str> {
    let mut roles = vec![
        roles::FOOTER_BAR,
        roles::OVERLAY_TITLE,
        roles::OVERLAY_SUBTITLE,
        roles::OVERLAY_FOOTER,
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
    ];
    roles.extend(roles::STATUS_SIGNAL_BARS);
    roles
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
            Some(crate::lvgl::theme::MUTED_RGB)
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
