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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRegistryEntry {
    pub screen: UiScreen,
    pub render_scene: RenderScene,
    pub native_scene: NativeRenderScene,
}

pub const fn screen_entry(screen: UiScreen) -> ScreenRegistryEntry {
    ScreenRegistryEntry {
        screen,
        render_scene: render_scene(screen),
        native_scene: native_scene(screen),
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
}
