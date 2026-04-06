"""VoIP screens for YoyoPod."""

from yoyopy.ui.screens.voip.quick_call import CallScreen
from yoyopy.ui.screens.voip.call_history import CallHistoryScreen
from yoyopy.ui.screens.voip.incoming_call import IncomingCallScreen
from yoyopy.ui.screens.voip.outgoing_call import OutgoingCallScreen
from yoyopy.ui.screens.voip.in_call import InCallScreen
from yoyopy.ui.screens.voip.contact_list import ContactListScreen
from yoyopy.ui.screens.voip.voice_note import VoiceNoteScreen

__all__ = [
    'CallScreen',
    'CallHistoryScreen',
    'IncomingCallScreen',
    'OutgoingCallScreen',
    'InCallScreen',
    'ContactListScreen',
    'VoiceNoteScreen',
]
