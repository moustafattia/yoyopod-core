use yoyopod_protocol::ui::InputAction;
pub use yoyopod_protocol::ui::UiScreen;

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
pub enum Persistence {
    Ephemeral,
    KeepAlive,
    Singleton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentTemplate {
    MusicShuffleAll,
    MusicPlayPause,
    VoiceAskStart,
    VoiceAskStop,
    VoiceCaptureStartRecipient,
    VoiceCaptureStop,
    VoiceCaptureCancel,
    VoiceDiscard,
    CallAnswer,
    CallReject,
    CallHangup,
    CallToggleMute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKind {
    Playlists,
    RecentTracks,
    Contacts,
    CallHistory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicActionKind {
    TalkContact,
    VoiceNote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionTarget {
    PushScreen(UiScreen),
    EmitIntent(IntentTemplate),
    PushWithIntent {
        screen: UiScreen,
        intent: IntentTemplate,
    },
    DynamicListItem {
        kind: ListKind,
    },
    DynamicAction {
        kind: DynamicActionKind,
    },
    AdvanceFocus,
    Noop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotCondition {
    Always,
    VoiceReady,
    VoiceRecording,
    VoiceReviewOrFailedOrSent,
    VoiceReadyOrRecording,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassthroughPolicy {
    pub trigger: InputAction,
    pub when: SnapshotCondition,
    pub intent: IntentTemplate,
    pub captures_button: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackPolicy {
    pub when: SnapshotCondition,
    pub intent: IntentTemplate,
    pub pop_screen: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Route {
    pub screen: UiScreen,
    pub title: &'static str,
    pub model_kind: ScreenModelKind,
    pub controller_kind: ControllerKind,
    pub native_controller_kind: ControllerKind,
    pub focus_policy: FocusPolicy,
    pub nav_policy: NavigationPolicy,
    pub persistence: Persistence,
    pub render_scene: RenderScene,
    pub native_scene: NativeRenderScene,
    pub select: &'static [SelectionTarget],
    pub back: &'static [BackPolicy],
    pub passthrough: &'static [PassthroughPolicy],
}
