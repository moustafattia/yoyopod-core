#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    CenteredHeroIcon,
    ListWithChrome,
    NowPlayingPanel,
    CallPanel,
    TalkActionsGrid,
    PagedDetail,
    OverlayCenter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionId {
    Auto,
    Backdrop,
    Header,
    HeroIcon,
    ListBody,
    ButtonRow,
    Dots,
    Progress,
    Title,
    Subtitle,
    Footer,
    StatusBar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub const fn region_rect(stage: Stage, region: RegionId) -> Option<LayoutRect> {
    match (stage, region) {
        (_, RegionId::Backdrop) => Some(LayoutRect {
            x: 0,
            y: 0,
            w: 240,
            h: 280,
        }),
        (_, RegionId::StatusBar) => Some(LayoutRect {
            x: 14,
            y: 10,
            w: 212,
            h: 16,
        }),
        (_, RegionId::Footer) => Some(LayoutRect {
            x: 14,
            y: 244,
            w: 212,
            h: 24,
        }),
        (Stage::CenteredHeroIcon, RegionId::HeroIcon) => Some(LayoutRect {
            x: 62,
            y: 54,
            w: 116,
            h: 116,
        }),
        (Stage::ListWithChrome, RegionId::ListBody) => Some(LayoutRect {
            x: 12,
            y: 48,
            w: 216,
            h: 184,
        }),
        _ => None,
    }
}
