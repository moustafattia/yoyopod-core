use crate::app::UiScreen;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderScene {
    Hub,
    List,
    NowPlaying,
    Ask,
    TalkActions,
    Call,
    Power,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeRenderScene {
    Hub,
    Listen,
    Playlist,
    NowPlaying,
    Talk,
    TalkActions,
    IncomingCall,
    OutgoingCall,
    InCall,
    Ask,
    Power,
    Overlay,
}

impl NativeRenderScene {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hub => "hub",
            Self::Listen => "listen",
            Self::Playlist => "playlist",
            Self::NowPlaying => "now_playing",
            Self::Talk => "talk",
            Self::TalkActions => "talk_actions",
            Self::IncomingCall => "incoming_call",
            Self::OutgoingCall => "outgoing_call",
            Self::InCall => "in_call",
            Self::Ask => "ask",
            Self::Power => "power",
            Self::Overlay => "overlay",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenModelKind {
    Hub,
    List,
    NowPlaying,
    Ask,
    TalkActions,
    Call,
    Power,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerKind {
    Hub,
    List,
    Listen,
    Playlist,
    NowPlaying,
    Ask,
    Talk,
    TalkActions,
    Call,
    Power,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPolicy {
    None,
    Wrap,
    Clamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationPolicy {
    Root,
    Stack,
    Overlay,
    Call,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRegistryEntry {
    pub screen: UiScreen,
    pub model_kind: ScreenModelKind,
    pub controller_kind: ControllerKind,
    pub native_controller_kind: ControllerKind,
    pub focus_policy: FocusPolicy,
    pub navigation_policy: NavigationPolicy,
    pub render_scene: RenderScene,
    pub native_scene: NativeRenderScene,
}

pub const fn screen_entry(screen: UiScreen) -> ScreenRegistryEntry {
    ScreenRegistryEntry {
        screen,
        model_kind: model_kind(screen),
        controller_kind: controller_kind(screen),
        native_controller_kind: native_controller_kind(screen),
        focus_policy: focus_policy(screen),
        navigation_policy: navigation_policy(screen),
        render_scene: render_scene(screen),
        native_scene: native_scene(screen),
    }
}

const fn model_kind(screen: UiScreen) -> ScreenModelKind {
    match screen {
        UiScreen::Hub => ScreenModelKind::Hub,
        UiScreen::Listen
        | UiScreen::Playlists
        | UiScreen::RecentTracks
        | UiScreen::Talk
        | UiScreen::Contacts
        | UiScreen::CallHistory => ScreenModelKind::List,
        UiScreen::NowPlaying => ScreenModelKind::NowPlaying,
        UiScreen::Ask => ScreenModelKind::Ask,
        UiScreen::TalkContact | UiScreen::VoiceNote => ScreenModelKind::TalkActions,
        UiScreen::IncomingCall | UiScreen::OutgoingCall | UiScreen::InCall => ScreenModelKind::Call,
        UiScreen::Power => ScreenModelKind::Power,
        UiScreen::Loading | UiScreen::Error => ScreenModelKind::Overlay,
    }
}

const fn controller_kind(screen: UiScreen) -> ControllerKind {
    match screen {
        UiScreen::Hub => ControllerKind::Hub,
        UiScreen::Listen
        | UiScreen::Playlists
        | UiScreen::RecentTracks
        | UiScreen::Talk
        | UiScreen::Contacts
        | UiScreen::CallHistory => ControllerKind::List,
        UiScreen::NowPlaying => ControllerKind::NowPlaying,
        UiScreen::Ask => ControllerKind::Ask,
        UiScreen::TalkContact | UiScreen::VoiceNote => ControllerKind::TalkActions,
        UiScreen::IncomingCall | UiScreen::OutgoingCall | UiScreen::InCall => ControllerKind::Call,
        UiScreen::Power => ControllerKind::Power,
        UiScreen::Loading | UiScreen::Error => ControllerKind::Overlay,
    }
}

const fn native_controller_kind(screen: UiScreen) -> ControllerKind {
    match screen {
        UiScreen::Hub => ControllerKind::Hub,
        UiScreen::Listen => ControllerKind::Listen,
        UiScreen::Playlists
        | UiScreen::RecentTracks
        | UiScreen::Contacts
        | UiScreen::CallHistory => ControllerKind::Playlist,
        UiScreen::NowPlaying => ControllerKind::NowPlaying,
        UiScreen::Ask => ControllerKind::Ask,
        UiScreen::Talk => ControllerKind::Talk,
        UiScreen::TalkContact | UiScreen::VoiceNote => ControllerKind::TalkActions,
        UiScreen::IncomingCall | UiScreen::OutgoingCall | UiScreen::InCall => ControllerKind::Call,
        UiScreen::Power => ControllerKind::Power,
        UiScreen::Loading | UiScreen::Error => ControllerKind::Overlay,
    }
}

pub const fn controller_kind_for_native_scene(scene: NativeRenderScene) -> ControllerKind {
    match scene {
        NativeRenderScene::Hub => ControllerKind::Hub,
        NativeRenderScene::Listen => ControllerKind::Listen,
        NativeRenderScene::Playlist => ControllerKind::Playlist,
        NativeRenderScene::NowPlaying => ControllerKind::NowPlaying,
        NativeRenderScene::Talk => ControllerKind::Talk,
        NativeRenderScene::TalkActions => ControllerKind::TalkActions,
        NativeRenderScene::IncomingCall
        | NativeRenderScene::OutgoingCall
        | NativeRenderScene::InCall => ControllerKind::Call,
        NativeRenderScene::Ask => ControllerKind::Ask,
        NativeRenderScene::Power => ControllerKind::Power,
        NativeRenderScene::Overlay => ControllerKind::Overlay,
    }
}

const fn focus_policy(screen: UiScreen) -> FocusPolicy {
    match screen {
        UiScreen::Hub
        | UiScreen::Listen
        | UiScreen::Talk
        | UiScreen::TalkContact
        | UiScreen::VoiceNote
        | UiScreen::Power => FocusPolicy::Wrap,
        UiScreen::Playlists
        | UiScreen::RecentTracks
        | UiScreen::Contacts
        | UiScreen::CallHistory => FocusPolicy::Clamp,
        UiScreen::NowPlaying
        | UiScreen::Ask
        | UiScreen::IncomingCall
        | UiScreen::OutgoingCall
        | UiScreen::InCall
        | UiScreen::Loading
        | UiScreen::Error => FocusPolicy::None,
    }
}

const fn navigation_policy(screen: UiScreen) -> NavigationPolicy {
    match screen {
        UiScreen::Hub => NavigationPolicy::Root,
        UiScreen::Loading | UiScreen::Error => NavigationPolicy::Overlay,
        UiScreen::IncomingCall | UiScreen::OutgoingCall | UiScreen::InCall => {
            NavigationPolicy::Call
        }
        _ => NavigationPolicy::Stack,
    }
}

const fn render_scene(screen: UiScreen) -> RenderScene {
    match screen {
        UiScreen::Hub => RenderScene::Hub,
        UiScreen::Listen
        | UiScreen::Playlists
        | UiScreen::RecentTracks
        | UiScreen::Talk
        | UiScreen::Contacts
        | UiScreen::CallHistory => RenderScene::List,
        UiScreen::NowPlaying => RenderScene::NowPlaying,
        UiScreen::Ask => RenderScene::Ask,
        UiScreen::TalkContact | UiScreen::VoiceNote => RenderScene::TalkActions,
        UiScreen::IncomingCall | UiScreen::OutgoingCall | UiScreen::InCall => RenderScene::Call,
        UiScreen::Power => RenderScene::Power,
        UiScreen::Loading | UiScreen::Error => RenderScene::Overlay,
    }
}

const fn native_scene(screen: UiScreen) -> NativeRenderScene {
    match screen {
        UiScreen::Hub => NativeRenderScene::Hub,
        UiScreen::Listen => NativeRenderScene::Listen,
        UiScreen::Playlists
        | UiScreen::RecentTracks
        | UiScreen::Contacts
        | UiScreen::CallHistory => NativeRenderScene::Playlist,
        UiScreen::NowPlaying => NativeRenderScene::NowPlaying,
        UiScreen::Talk => NativeRenderScene::Talk,
        UiScreen::TalkContact | UiScreen::VoiceNote => NativeRenderScene::TalkActions,
        UiScreen::IncomingCall => NativeRenderScene::IncomingCall,
        UiScreen::OutgoingCall => NativeRenderScene::OutgoingCall,
        UiScreen::InCall => NativeRenderScene::InCall,
        UiScreen::Ask => NativeRenderScene::Ask,
        UiScreen::Power => NativeRenderScene::Power,
        UiScreen::Loading | UiScreen::Error => NativeRenderScene::Overlay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_covers_every_screen() {
        for screen in UiScreen::ALL {
            let entry = screen_entry(screen);
            assert_eq!(entry.screen, screen);
        }
    }

    #[test]
    fn registry_keeps_overlay_model_and_controller_aligned() {
        for screen in [UiScreen::Loading, UiScreen::Error] {
            let entry = screen_entry(screen);
            assert_eq!(entry.model_kind, ScreenModelKind::Overlay);
            assert_eq!(entry.controller_kind, ControllerKind::Overlay);
            assert_eq!(entry.native_controller_kind, ControllerKind::Overlay);
            assert_eq!(entry.render_scene, RenderScene::Overlay);
            assert_eq!(entry.native_scene, NativeRenderScene::Overlay);
        }
    }

    #[test]
    fn registry_declares_focus_policy_for_all_focusable_screens() {
        for screen in UiScreen::ALL {
            let entry = screen_entry(screen);
            match entry.focus_policy {
                FocusPolicy::Wrap | FocusPolicy::Clamp | FocusPolicy::None => {}
            }
        }
    }

    #[test]
    fn registry_distinguishes_generic_and_native_list_controllers() {
        let listen = screen_entry(UiScreen::Listen);
        assert_eq!(listen.controller_kind, ControllerKind::List);
        assert_eq!(listen.native_controller_kind, ControllerKind::Listen);

        let playlists = screen_entry(UiScreen::Playlists);
        assert_eq!(playlists.controller_kind, ControllerKind::List);
        assert_eq!(playlists.native_controller_kind, ControllerKind::Playlist);

        let talk = screen_entry(UiScreen::Talk);
        assert_eq!(talk.controller_kind, ControllerKind::List);
        assert_eq!(talk.native_controller_kind, ControllerKind::Talk);
    }

    #[test]
    fn native_scene_controller_mapping_matches_screen_entries() {
        for screen in UiScreen::ALL {
            let entry = screen_entry(screen);
            assert_eq!(
                controller_kind_for_native_scene(entry.native_scene),
                entry.native_controller_kind
            );
        }
    }
}
