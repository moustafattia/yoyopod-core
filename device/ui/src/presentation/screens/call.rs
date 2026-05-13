use crate::presentation::screens::{
    chrome, CallViewModel, ListScreenModel, TalkActionButtonModel, TalkActionsViewModel,
};
use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot};

pub fn contacts_model(snapshot: &RuntimeSnapshot, focus_index: usize) -> ListScreenModel {
    ListScreenModel {
        chrome: chrome::chrome(snapshot, "Tap = Next | 2x Tap = Open | Hold = Back"),
        title: "More People".to_string(),
        subtitle: "Add contacts to call them here.".to_string(),
        rows: chrome::list_rows(&snapshot.call.contacts, focus_index),
    }
}

pub fn call_history_model(snapshot: &RuntimeSnapshot, focus_index: usize) -> ListScreenModel {
    ListScreenModel {
        chrome: chrome::chrome(snapshot, "Tap = Next | 2x Tap = Call | Hold = Back"),
        title: "Recents".to_string(),
        subtitle: "Calls will appear here.".to_string(),
        rows: chrome::list_rows(&snapshot.call.history, focus_index),
    }
}

pub fn talk_contact_model(
    snapshot: &RuntimeSnapshot,
    focus_index: usize,
    selected_contact: Option<&ListItemSnapshot>,
) -> TalkActionsViewModel {
    let contact = selected_contact
        .or_else(|| snapshot.call.contacts.first())
        .cloned()
        .unwrap_or_else(|| ListItemSnapshot::new("", "Friend", "", "mono:FR"));
    let actions = talk_contact_actions(snapshot, Some(&contact));
    let selected_index = focus_index.min(actions.len().saturating_sub(1));
    TalkActionsViewModel {
        chrome: chrome::chrome(snapshot, "Tap Next | 2x Select | Hold Back"),
        contact_name: contact.title,
        title: actions
            .get(selected_index)
            .map(|action| action.title.to_string())
            .unwrap_or_default(),
        status: String::new(),
        status_kind: 0,
        buttons: actions
            .iter()
            .map(|action| TalkActionButtonModel {
                title: action.title.to_string(),
                icon_key: action.icon_key.to_string(),
                color_kind: 0,
            })
            .collect(),
        selected_index,
        layout_kind: 0,
        button_size_kind: if actions.len() >= 3 { 0 } else { 1 },
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TalkContactAction {
    pub kind: &'static str,
    pub title: &'static str,
    pub icon_key: &'static str,
}

pub fn talk_contact_actions(
    snapshot: &RuntimeSnapshot,
    selected_contact: Option<&ListItemSnapshot>,
) -> Vec<TalkContactAction> {
    let contact = selected_contact.or_else(|| snapshot.call.contacts.first());
    let mut actions = vec![
        TalkContactAction {
            kind: "call",
            title: "Call",
            icon_key: "call",
        },
        TalkContactAction {
            kind: "voice_note",
            title: "Voice Note",
            icon_key: "voice_note",
        },
    ];
    if contact
        .and_then(|contact| snapshot.call.latest_voice_note_by_contact.get(&contact.id))
        .is_some_and(|note| !note.local_file_path.trim().is_empty())
    {
        actions.push(TalkContactAction {
            kind: "play_note",
            title: "Play Note",
            icon_key: "play",
        });
    }
    actions
}

pub fn incoming_model(snapshot: &RuntimeSnapshot) -> CallViewModel {
    CallViewModel {
        chrome: chrome::chrome(snapshot, "Tap = Answer | Hold = Decline"),
        title: call_peer_name(snapshot),
        subtitle: snapshot.call.peer_address.clone(),
        detail: "Incoming Call".to_string(),
        muted: snapshot.call.muted,
    }
}

pub fn outgoing_model(snapshot: &RuntimeSnapshot) -> CallViewModel {
    CallViewModel {
        chrome: chrome::chrome(snapshot, "Hold = Cancel"),
        title: call_peer_name(snapshot),
        subtitle: snapshot.call.peer_address.clone(),
        detail: "Dialing".to_string(),
        muted: snapshot.call.muted,
    }
}

pub fn in_call_model(snapshot: &RuntimeSnapshot) -> CallViewModel {
    let footer = if snapshot.call.muted {
        "Tap = Unmute | Hold = End"
    } else {
        "Tap = Mute | Hold = End"
    };
    CallViewModel {
        chrome: chrome::chrome(snapshot, footer),
        title: call_peer_name(snapshot),
        subtitle: snapshot.call.duration_text.clone(),
        detail: snapshot.call.peer_address.clone(),
        muted: snapshot.call.muted,
    }
}

fn call_peer_name(snapshot: &RuntimeSnapshot) -> String {
    if snapshot.call.peer_name.trim().is_empty() {
        "Unknown".to_string()
    } else {
        snapshot.call.peer_name.clone()
    }
}
