"""Local voice-command and spoken-response interfaces."""

from yoyopy.voice.capture import (
    AudioCaptureBackend,
    NullAudioCaptureBackend,
    SubprocessAudioCaptureBackend,
)
from yoyopy.voice.commands import (
    VOICE_COMMAND_GRAMMAR,
    VoiceCommandIntent,
    VoiceCommandMatch,
    VoiceCommandTemplate,
    match_voice_command,
)
from yoyopy.voice.devices import VoiceDeviceCatalog
from yoyopy.voice.models import (
    VoiceCaptureRequest,
    VoiceCaptureResult,
    VoiceSettings,
    VoiceTranscript,
)
from yoyopy.voice.output import AlsaOutputPlayer
from yoyopy.voice.service import VoiceService
from yoyopy.voice.stt import NullSpeechToTextBackend, SpeechToTextBackend, VoskSpeechToTextBackend
from yoyopy.voice.tts import (
    EspeakNgTextToSpeechBackend,
    NullTextToSpeechBackend,
    TextToSpeechBackend,
)

__all__ = [
    "AudioCaptureBackend",
    "AlsaOutputPlayer",
    "EspeakNgTextToSpeechBackend",
    "NullAudioCaptureBackend",
    "NullSpeechToTextBackend",
    "NullTextToSpeechBackend",
    "SpeechToTextBackend",
    "SubprocessAudioCaptureBackend",
    "TextToSpeechBackend",
    "VoiceDeviceCatalog",
    "VoiceCaptureRequest",
    "VoiceCaptureResult",
    "VoiceCommandIntent",
    "VoiceCommandMatch",
    "VoiceCommandTemplate",
    "VoiceService",
    "VoiceSettings",
    "VoiceTranscript",
    "VoskSpeechToTextBackend",
    "VOICE_COMMAND_GRAMMAR",
    "match_voice_command",
]
