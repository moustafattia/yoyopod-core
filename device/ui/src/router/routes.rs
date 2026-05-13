pub use yoyopod_protocol::ui::UiScreen;
use yoyopod_protocol::ui::{
    InputAction, IntentKind, RuntimeSnapshotDomain, ScreenCapabilities, UiIntent,
};

use crate::engine::DirtyRegion;

use super::route::{
    BackPolicy, ControllerKind, DynamicActionKind, FocusPolicy, IntentTemplate, ListKind,
    NativeRenderScene, NavigationPolicy, PassthroughPolicy, Persistence, RenderScene, Route,
    ScreenModelKind, SelectionTarget, SnapshotCondition,
};

const STATUS_BAR_REGION: DirtyRegion = DirtyRegion {
    x: 0,
    y: 0,
    w: 240,
    h: 32,
};

pub const fn route_for(screen: UiScreen) -> Route {
    Route {
        screen,
        title: screen.as_str(),
        model_kind: model_kind(screen),
        controller_kind: controller_kind(screen),
        native_controller_kind: native_controller_kind(screen),
        focus_policy: focus_policy(screen),
        nav_policy: navigation_policy(screen),
        persistence: persistence(screen),
        render_scene: render_scene(screen),
        native_scene: native_scene(screen),
        select: select_targets(screen),
        passthrough: passthrough_policies(screen),
        back: back_policies(screen),
    }
}

pub const fn dirty_region_for(
    screen: UiScreen,
    domain: RuntimeSnapshotDomain,
) -> Option<DirtyRegion> {
    match (screen, domain) {
        (UiScreen::Power, RuntimeSnapshotDomain::Power) => None,
        (_, RuntimeSnapshotDomain::Power | RuntimeSnapshotDomain::Network) => {
            Some(STATUS_BAR_REGION)
        }
        _ => None,
    }
}

pub fn screen_capabilities() -> Vec<ScreenCapabilities> {
    UiScreen::ALL
        .iter()
        .copied()
        .map(|screen| {
            let entry = route_for(screen);
            let mut supported_intents = Vec::new();
            for target in entry.select {
                add_selection_intent(*target, &mut supported_intents);
            }
            for passthrough in entry.passthrough {
                add_intent_kind(
                    template_intent_kind(passthrough.intent),
                    &mut supported_intents,
                );
            }
            for back in entry.back {
                add_intent_kind(template_intent_kind(back.intent), &mut supported_intents);
            }
            ScreenCapabilities {
                screen,
                supported_intents,
                passthrough: entry.passthrough.first().map(|policy| policy.trigger),
            }
        })
        .collect()
}

fn add_selection_intent(target: SelectionTarget, supported_intents: &mut Vec<IntentKind>) {
    match target {
        SelectionTarget::EmitIntent(template)
        | SelectionTarget::PushWithIntent {
            intent: template, ..
        } => add_intent_kind(template_intent_kind(template), supported_intents),
        SelectionTarget::DynamicListItem { kind } => {
            add_intent_kind(dynamic_list_intent_kind(kind), supported_intents);
        }
        SelectionTarget::DynamicAction { kind } => {
            for intent in dynamic_action_intent_kinds(kind) {
                add_intent_kind((*intent).into(), supported_intents);
            }
        }
        SelectionTarget::PushScreen(_) | SelectionTarget::AdvanceFocus | SelectionTarget::Noop => {}
    }
}

fn add_intent_kind(intent: IntentKind, supported_intents: &mut Vec<IntentKind>) {
    if !supported_intents.contains(&intent) {
        supported_intents.push(intent);
    }
}

fn template_intent_kind(template: IntentTemplate) -> IntentKind {
    let (domain, action) = match template {
        IntentTemplate::MusicShuffleAll => ("music", "shuffle_all"),
        IntentTemplate::MusicPlayPause => ("music", "play_pause"),
        IntentTemplate::VoiceAskStart => ("voice", "ask_start"),
        IntentTemplate::VoiceAskStop => ("voice", "ask_stop"),
        IntentTemplate::VoiceCaptureStartRecipient => ("voice", "capture_start"),
        IntentTemplate::VoiceCaptureStop => ("voice", "capture_stop"),
        IntentTemplate::VoiceCaptureCancel => ("voice", "capture_cancel"),
        IntentTemplate::VoiceDiscard => ("voice", "discard"),
        IntentTemplate::CallAnswer => ("call", "answer"),
        IntentTemplate::CallReject => ("call", "reject"),
        IntentTemplate::CallHangup => ("call", "hangup"),
        IntentTemplate::CallToggleMute => ("call", "toggle_mute"),
    };
    IntentKind {
        domain: domain.to_string(),
        action: action.to_string(),
    }
}

fn dynamic_list_intent_kind(kind: ListKind) -> IntentKind {
    match kind {
        ListKind::Playlists => IntentKind {
            domain: "music".to_string(),
            action: "load_playlist".to_string(),
        },
        ListKind::RecentTracks => IntentKind {
            domain: "music".to_string(),
            action: "play_recent_track".to_string(),
        },
        ListKind::Contacts | ListKind::CallHistory => IntentKind {
            domain: "call".to_string(),
            action: "start".to_string(),
        },
    }
}

fn dynamic_action_intent_kinds(kind: DynamicActionKind) -> &'static [IntentKindLiteral] {
    const TALK_CONTACT_INTENTS: &[IntentKindLiteral] = &[
        IntentKindLiteral::new("call", "start"),
        IntentKindLiteral::new("voice", "capture_start"),
        IntentKindLiteral::new("voice", "play_latest"),
        IntentKindLiteral::new("voice", "mark_seen"),
    ];
    const VOICE_NOTE_INTENTS: &[IntentKindLiteral] = &[
        IntentKindLiteral::new("voice", "capture_start"),
        IntentKindLiteral::new("voice", "capture_stop"),
        IntentKindLiteral::new("voice", "capture_cancel"),
        IntentKindLiteral::new("voice", "send"),
        IntentKindLiteral::new("voice", "play"),
        IntentKindLiteral::new("voice", "discard"),
    ];

    match kind {
        DynamicActionKind::TalkContact => TALK_CONTACT_INTENTS,
        DynamicActionKind::VoiceNote => VOICE_NOTE_INTENTS,
    }
}

#[derive(Debug, Clone, Copy)]
struct IntentKindLiteral {
    domain: &'static str,
    action: &'static str,
}

impl IntentKindLiteral {
    const fn new(domain: &'static str, action: &'static str) -> Self {
        Self { domain, action }
    }
}

impl From<IntentKindLiteral> for IntentKind {
    fn from(value: IntentKindLiteral) -> Self {
        Self {
            domain: value.domain.to_string(),
            action: value.action.to_string(),
        }
    }
}

const HUB_SELECT: &[SelectionTarget] = &[
    SelectionTarget::PushScreen(UiScreen::Listen),
    SelectionTarget::PushScreen(UiScreen::Talk),
    SelectionTarget::PushScreen(UiScreen::Ask),
    SelectionTarget::PushScreen(UiScreen::Power),
];
const LISTEN_SELECT: &[SelectionTarget] = &[
    SelectionTarget::PushScreen(UiScreen::Playlists),
    SelectionTarget::PushScreen(UiScreen::RecentTracks),
    SelectionTarget::PushWithIntent {
        screen: UiScreen::NowPlaying,
        intent: IntentTemplate::MusicShuffleAll,
    },
];
const TALK_SELECT: &[SelectionTarget] = &[
    SelectionTarget::PushScreen(UiScreen::Contacts),
    SelectionTarget::PushScreen(UiScreen::CallHistory),
    SelectionTarget::PushScreen(UiScreen::VoiceNote),
];
const PLAYLISTS_SELECT: &[SelectionTarget] = &[SelectionTarget::DynamicListItem {
    kind: ListKind::Playlists,
}];
const RECENT_TRACKS_SELECT: &[SelectionTarget] = &[SelectionTarget::DynamicListItem {
    kind: ListKind::RecentTracks,
}];
const CONTACTS_SELECT: &[SelectionTarget] = &[SelectionTarget::DynamicListItem {
    kind: ListKind::Contacts,
}];
const CALL_HISTORY_SELECT: &[SelectionTarget] = &[SelectionTarget::DynamicListItem {
    kind: ListKind::CallHistory,
}];
const NOW_PLAYING_SELECT: &[SelectionTarget] =
    &[SelectionTarget::EmitIntent(IntentTemplate::MusicPlayPause)];
const ASK_SELECT: &[SelectionTarget] =
    &[SelectionTarget::EmitIntent(IntentTemplate::VoiceAskStart)];
const TALK_CONTACT_SELECT: &[SelectionTarget] = &[SelectionTarget::DynamicAction {
    kind: DynamicActionKind::TalkContact,
}];
const VOICE_NOTE_SELECT: &[SelectionTarget] = &[SelectionTarget::DynamicAction {
    kind: DynamicActionKind::VoiceNote,
}];
const INCOMING_SELECT: &[SelectionTarget] =
    &[SelectionTarget::EmitIntent(IntentTemplate::CallAnswer)];
const IN_CALL_SELECT: &[SelectionTarget] =
    &[SelectionTarget::EmitIntent(IntentTemplate::CallToggleMute)];
const POWER_SELECT: &[SelectionTarget] = &[SelectionTarget::AdvanceFocus];
const NO_SELECT: &[SelectionTarget] = &[SelectionTarget::Noop];

const ASK_PASSTHROUGH: &[PassthroughPolicy] = &[
    PassthroughPolicy {
        trigger: InputAction::PttPress,
        when: SnapshotCondition::Always,
        intent: IntentTemplate::VoiceAskStart,
        captures_button: false,
    },
    PassthroughPolicy {
        trigger: InputAction::PttRelease,
        when: SnapshotCondition::Always,
        intent: IntentTemplate::VoiceAskStop,
        captures_button: false,
    },
];
const VOICE_NOTE_PASSTHROUGH: &[PassthroughPolicy] = &[
    PassthroughPolicy {
        trigger: InputAction::PttPress,
        when: SnapshotCondition::VoiceReady,
        intent: IntentTemplate::VoiceCaptureStartRecipient,
        captures_button: true,
    },
    PassthroughPolicy {
        trigger: InputAction::PttRelease,
        when: SnapshotCondition::VoiceRecording,
        intent: IntentTemplate::VoiceCaptureStop,
        captures_button: true,
    },
];
const NO_PASSTHROUGH: &[PassthroughPolicy] = &[];

const VOICE_NOTE_BACK: &[BackPolicy] = &[
    BackPolicy {
        when: SnapshotCondition::VoiceRecording,
        intent: IntentTemplate::VoiceCaptureCancel,
        pop_screen: true,
    },
    BackPolicy {
        when: SnapshotCondition::VoiceReviewOrFailedOrSent,
        intent: IntentTemplate::VoiceDiscard,
        pop_screen: true,
    },
];
const NO_BACK: &[BackPolicy] = &[];

const fn select_targets(screen: UiScreen) -> &'static [SelectionTarget] {
    match screen {
        UiScreen::Hub => HUB_SELECT,
        UiScreen::Listen => LISTEN_SELECT,
        UiScreen::Talk => TALK_SELECT,
        UiScreen::Playlists => PLAYLISTS_SELECT,
        UiScreen::RecentTracks => RECENT_TRACKS_SELECT,
        UiScreen::NowPlaying => NOW_PLAYING_SELECT,
        UiScreen::Ask => ASK_SELECT,
        UiScreen::VoiceNote => VOICE_NOTE_SELECT,
        UiScreen::Contacts => CONTACTS_SELECT,
        UiScreen::TalkContact => TALK_CONTACT_SELECT,
        UiScreen::CallHistory => CALL_HISTORY_SELECT,
        UiScreen::IncomingCall => INCOMING_SELECT,
        UiScreen::InCall => IN_CALL_SELECT,
        UiScreen::Power => POWER_SELECT,
        UiScreen::OutgoingCall | UiScreen::Loading | UiScreen::Error => NO_SELECT,
    }
}

const fn passthrough_policies(screen: UiScreen) -> &'static [PassthroughPolicy] {
    match screen {
        UiScreen::Ask => ASK_PASSTHROUGH,
        UiScreen::VoiceNote => VOICE_NOTE_PASSTHROUGH,
        _ => NO_PASSTHROUGH,
    }
}

const fn back_policies(screen: UiScreen) -> &'static [BackPolicy] {
    match screen {
        UiScreen::VoiceNote => VOICE_NOTE_BACK,
        _ => NO_BACK,
    }
}

pub fn static_intent_template(template: IntentTemplate) -> Option<UiIntent> {
    match template {
        IntentTemplate::MusicShuffleAll => Some(UiIntent::Music(
            yoyopod_protocol::ui::MusicIntent::ShuffleAll,
        )),
        IntentTemplate::MusicPlayPause => Some(UiIntent::Music(
            yoyopod_protocol::ui::MusicIntent::PlayPause,
        )),
        IntentTemplate::VoiceAskStart => {
            Some(UiIntent::Voice(yoyopod_protocol::ui::VoiceIntent::AskStart))
        }
        IntentTemplate::VoiceAskStop => {
            Some(UiIntent::Voice(yoyopod_protocol::ui::VoiceIntent::AskStop))
        }
        IntentTemplate::VoiceCaptureStartRecipient => None,
        IntentTemplate::VoiceCaptureStop => Some(UiIntent::Voice(
            yoyopod_protocol::ui::VoiceIntent::CaptureStop,
        )),
        IntentTemplate::VoiceCaptureCancel => Some(UiIntent::Voice(
            yoyopod_protocol::ui::VoiceIntent::CaptureCancel,
        )),
        IntentTemplate::VoiceDiscard => {
            Some(UiIntent::Voice(yoyopod_protocol::ui::VoiceIntent::Discard))
        }
        IntentTemplate::CallAnswer => {
            Some(UiIntent::Call(yoyopod_protocol::ui::CallIntent::Answer))
        }
        IntentTemplate::CallReject => {
            Some(UiIntent::Call(yoyopod_protocol::ui::CallIntent::Reject))
        }
        IntentTemplate::CallHangup => {
            Some(UiIntent::Call(yoyopod_protocol::ui::CallIntent::Hangup))
        }
        IntentTemplate::CallToggleMute => {
            Some(UiIntent::Call(yoyopod_protocol::ui::CallIntent::ToggleMute))
        }
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

const fn persistence(screen: UiScreen) -> Persistence {
    match screen {
        UiScreen::NowPlaying => Persistence::KeepAlive,
        UiScreen::Loading | UiScreen::Error => Persistence::Singleton,
        _ => Persistence::Ephemeral,
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
