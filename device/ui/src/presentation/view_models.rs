use yoyopod_protocol::ui::UiScreen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarModel {
    pub network_connected: bool,
    pub network_enabled: bool,
    pub connection_type: String,
    pub signal_strength: i32,
    pub gps_has_fix: bool,
    pub battery_percent: i32,
    pub charging: bool,
    pub power_available: bool,
    pub voip_state: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChromeModel {
    pub status: StatusBarModel,
    pub footer: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubCardModel {
    pub key: String,
    pub title: String,
    pub subtitle: String,
    pub accent: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubViewModel {
    pub chrome: ChromeModel,
    pub cards: Vec<HubCardModel>,
    pub selected_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListRowModel {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub icon_key: String,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListScreenModel {
    pub chrome: ChromeModel,
    pub title: String,
    pub subtitle: String,
    pub rows: Vec<ListRowModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NowPlayingViewModel {
    pub chrome: ChromeModel,
    pub title: String,
    pub artist: String,
    pub state_text: String,
    pub progress_permille: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskViewModel {
    pub chrome: ChromeModel,
    pub title: String,
    pub subtitle: String,
    pub icon_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallViewModel {
    pub chrome: ChromeModel,
    pub title: String,
    pub subtitle: String,
    pub detail: String,
    pub muted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TalkActionButtonModel {
    pub title: String,
    pub icon_key: String,
    pub color_kind: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TalkActionsViewModel {
    pub chrome: ChromeModel,
    pub contact_name: String,
    pub title: String,
    pub status: String,
    pub status_kind: i32,
    pub buttons: Vec<TalkActionButtonModel>,
    pub selected_index: usize,
    pub layout_kind: i32,
    pub button_size_kind: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerViewModel {
    pub chrome: ChromeModel,
    pub title: String,
    pub subtitle: String,
    pub icon_key: String,
    pub rows: Vec<ListRowModel>,
    pub current_page_index: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayViewModel {
    pub chrome: ChromeModel,
    pub title: String,
    pub subtitle: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenModel {
    Hub(HubViewModel),
    Listen(ListScreenModel),
    Playlists(ListScreenModel),
    RecentTracks(ListScreenModel),
    NowPlaying(NowPlayingViewModel),
    Ask(AskViewModel),
    Talk(ListScreenModel),
    Contacts(ListScreenModel),
    CallHistory(ListScreenModel),
    TalkContact(TalkActionsViewModel),
    VoiceNote(TalkActionsViewModel),
    IncomingCall(CallViewModel),
    OutgoingCall(CallViewModel),
    InCall(CallViewModel),
    Power(PowerViewModel),
    Loading(OverlayViewModel),
    Error(OverlayViewModel),
}

impl ScreenModel {
    pub fn screen(&self) -> UiScreen {
        match self {
            Self::Hub(_) => UiScreen::Hub,
            Self::Listen(_) => UiScreen::Listen,
            Self::Playlists(_) => UiScreen::Playlists,
            Self::RecentTracks(_) => UiScreen::RecentTracks,
            Self::NowPlaying(_) => UiScreen::NowPlaying,
            Self::Ask(_) => UiScreen::Ask,
            Self::Talk(_) => UiScreen::Talk,
            Self::Contacts(_) => UiScreen::Contacts,
            Self::CallHistory(_) => UiScreen::CallHistory,
            Self::TalkContact(_) => UiScreen::TalkContact,
            Self::VoiceNote(_) => UiScreen::VoiceNote,
            Self::IncomingCall(_) => UiScreen::IncomingCall,
            Self::OutgoingCall(_) => UiScreen::OutgoingCall,
            Self::InCall(_) => UiScreen::InCall,
            Self::Power(_) => UiScreen::Power,
            Self::Loading(_) => UiScreen::Loading,
            Self::Error(_) => UiScreen::Error,
        }
    }

    pub fn chrome(&self) -> &ChromeModel {
        match self {
            Self::Hub(model) => &model.chrome,
            Self::Listen(model)
            | Self::Playlists(model)
            | Self::RecentTracks(model)
            | Self::Talk(model)
            | Self::Contacts(model)
            | Self::CallHistory(model) => &model.chrome,
            Self::NowPlaying(model) => &model.chrome,
            Self::Ask(model) => &model.chrome,
            Self::TalkContact(model) | Self::VoiceNote(model) => &model.chrome,
            Self::IncomingCall(model) | Self::OutgoingCall(model) | Self::InCall(model) => {
                &model.chrome
            }
            Self::Power(model) => &model.chrome,
            Self::Loading(model) | Self::Error(model) => &model.chrome,
        }
    }
}
