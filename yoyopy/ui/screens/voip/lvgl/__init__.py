"""LVGL-backed VoIP screen views."""

from yoyopy.ui.screens.voip.lvgl.call_view import LvglCallView
from yoyopy.ui.screens.voip.lvgl.call_history_view import LvglCallHistoryView
from yoyopy.ui.screens.voip.lvgl.contact_list_view import LvglContactListView
from yoyopy.ui.screens.voip.lvgl.in_call_view import LvglInCallView
from yoyopy.ui.screens.voip.lvgl.incoming_call_view import LvglIncomingCallView
from yoyopy.ui.screens.voip.lvgl.outgoing_call_view import LvglOutgoingCallView
from yoyopy.ui.screens.voip.lvgl.voice_note_view import LvglVoiceNoteView

__all__ = [
    "LvglCallView",
    "LvglCallHistoryView",
    "LvglContactListView",
    "LvglInCallView",
    "LvglIncomingCallView",
    "LvglOutgoingCallView",
    "LvglVoiceNoteView",
]
