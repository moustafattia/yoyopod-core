"""
Application context for YoyoPod.

Maintains shared state across the application including
current playlist, playback status, volume, and user settings.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional, List, Dict, Any, TYPE_CHECKING
from pathlib import Path
from loguru import logger

from yoyopy.ui.input.hal import InteractionProfile

if TYPE_CHECKING:
    from yoyopy.audio.manager import AudioManager
    from yoyopy.power import PowerSnapshot


@dataclass
class Track:
    """Represents a single audio track."""
    title: str
    artist: str
    duration: float = 0.0  # Duration in seconds
    file_path: Optional[Path] = None
    stream_url: Optional[str] = None
    album: Optional[str] = None
    artwork_url: Optional[str] = None


@dataclass
class Playlist:
    """Represents a playlist of tracks."""
    name: str
    tracks: List[Track] = field(default_factory=list)
    current_index: int = 0

    def current_track(self) -> Optional[Track]:
        """Get the currently playing track."""
        if 0 <= self.current_index < len(self.tracks):
            return self.tracks[self.current_index]
        return None

    def next_track(self) -> Optional[Track]:
        """Move to next track and return it."""
        if self.current_index < len(self.tracks) - 1:
            self.current_index += 1
            return self.current_track()
        return None

    def previous_track(self) -> Optional[Track]:
        """Move to previous track and return it."""
        if self.current_index > 0:
            self.current_index -= 1
            return self.current_track()
        return None

    def has_next(self) -> bool:
        """Check if there's a next track."""
        return self.current_index < len(self.tracks) - 1

    def has_previous(self) -> bool:
        """Check if there's a previous track."""
        return self.current_index > 0


@dataclass
class PlaybackState:
    """Current playback state."""
    is_playing: bool = False
    is_paused: bool = False
    is_stopped: bool = True
    position: float = 0.0  # Current position in seconds
    volume: int = 50  # Volume 0-100
    is_muted: bool = False


@dataclass
class VoiceState:
    """Runtime voice settings and recent voice activity."""

    commands_enabled: bool = True
    ai_requests_enabled: bool = True
    screen_read_enabled: bool = False
    stt_enabled: bool = True
    tts_enabled: bool = True
    mic_muted: bool = False
    speaker_device_id: str | None = None
    capture_device_id: str | None = None
    stt_available: bool = False
    tts_available: bool = False
    last_transcript: str = ""
    last_spoken_text: str = ""
    last_mode: str = ""
    output_volume: int = 50


_VOICE_UNSET = object()


class AppContext:
    """
    Central application context.

    Maintains all shared state including playback, playlists,
    user settings, and system status.
    """

    def __init__(
        self,
        audio_manager: Optional['AudioManager'] = None,
        interaction_profile: InteractionProfile = InteractionProfile.STANDARD,
    ) -> None:
        """
        Initialize application context.

        Args:
            audio_manager: Optional AudioManager instance for actual playback
        """
        # Playback state
        self.playback = PlaybackState()

        # Audio manager (optional, for actual audio playback)
        self.audio_manager = audio_manager
        self.interaction_profile = interaction_profile

        # Current playlist
        self.current_playlist: Optional[Playlist] = None

        # Available playlists
        self.playlists: Dict[str, Playlist] = {}

        # User settings
        self.settings = {
            "brightness": 100,
            "auto_sleep_minutes": 30,
            "parental_controls_enabled": False,
            "max_volume": 80,  # Parental control default
        }
        self.voice = VoiceState(output_volume=self.playback.volume)

        # System status
        self.battery_percent: int = 100
        self.battery_charging: bool = False
        self.external_power: bool = False
        self.power_available: bool = False
        self.power_error: str = ""
        self.voip_configured: bool = False
        self.voip_ready: bool = False
        self.screen_awake: bool = True
        self.screen_idle_seconds: int = 0
        self.screen_on_seconds: int = 0
        self.app_uptime_seconds: int = 0
        self.signal_strength: int = 4  # 0-4 bars
        self.is_connected: bool = False
        self.connection_type: str = "none"  # wifi, 4g, none
        self.network_enabled: bool = False
        self.gps_has_fix: bool = False
        self.missed_calls: int = 0
        self.recent_calls: List[str] = []
        self.unread_voice_notes: int = 0
        self.latest_voice_note_by_contact: Dict[str, Dict[str, Any]] = {}
        self.talk_contact_name: str = ""
        self.talk_contact_address: str = ""
        self.voice_note_recipient_name: str = ""
        self.voice_note_recipient_address: str = ""
        self.voice_note_send_state: str = "idle"
        self.voice_note_status_text: str = ""
        self.voice_note_file_path: str = ""
        self.voice_note_duration_ms: int = 0

        # Navigation history for back button
        self.navigation_history: List[str] = []

        logger.info("AppContext initialized")

    def set_playlist(self, playlist: Playlist) -> None:
        """
        Set the current playlist.

        Args:
            playlist: Playlist to set as current
        """
        self.current_playlist = playlist
        logger.info(f"Playlist set: {playlist.name} ({len(playlist.tracks)} tracks)")

    def get_current_track(self) -> Optional[Track]:
        """Get the currently playing/selected track."""
        if self.current_playlist:
            return self.current_playlist.current_track()
        return None

    def play(self) -> bool:
        """
        Start playback.

        Returns:
            True if playback started, False otherwise
        """
        if not self.current_playlist or not self.current_playlist.current_track():
            logger.warning("Cannot play: no track selected")
            return False

        self.playback.is_playing = True
        self.playback.is_paused = False
        self.playback.is_stopped = False
        logger.info(f"Playing: {self.get_current_track().title}")
        return True

    def pause(self) -> None:
        """Pause playback."""
        if self.playback.is_playing:
            self.playback.is_playing = False
            self.playback.is_paused = True
            logger.info("Playback paused")

    def resume(self) -> None:
        """Resume playback."""
        if self.playback.is_paused:
            self.playback.is_playing = True
            self.playback.is_paused = False
            logger.info("Playback resumed")

    def stop(self) -> None:
        """Stop playback."""
        self.playback.is_playing = False
        self.playback.is_paused = False
        self.playback.is_stopped = True
        self.playback.position = 0.0
        logger.info("Playback stopped")

    def toggle_playback(self) -> bool:
        """
        Toggle between play and pause.

        Returns:
            True if now playing, False if paused
        """
        if self.playback.is_playing:
            self.pause()
            return False
        elif self.playback.is_paused or self.playback.is_stopped:
            if self.playback.is_stopped:
                self.play()
            else:
                self.resume()
            return True
        return False

    def set_volume(self, volume: int) -> None:
        """
        Set playback volume.

        Args:
            volume: Volume level (0-100)
        """
        max_volume = self.settings.get("max_volume", 100)
        volume = max(0, min(volume, max_volume))
        self.playback.volume = volume
        self.voice.output_volume = volume

        # Sync with audio manager if available
        if self.audio_manager:
            self.audio_manager.volume = volume

        logger.debug(f"Volume set to {volume}")

    def volume_up(self, step: int = 5) -> int:
        """
        Increase volume.

        Args:
            step: Amount to increase (default 5)

        Returns:
            New volume level
        """
        new_volume = self.playback.volume + step
        self.set_volume(new_volume)
        return self.playback.volume

    def volume_down(self, step: int = 5) -> int:
        """
        Decrease volume.

        Args:
            step: Amount to decrease (default 5)

        Returns:
            New volume level
        """
        new_volume = self.playback.volume - step
        self.set_volume(new_volume)
        return self.playback.volume

    def next_track(self) -> Optional[Track]:
        """Skip to next track."""
        if self.current_playlist:
            track = self.current_playlist.next_track()
            if track:
                logger.info(f"Next track: {track.title}")
                if self.playback.is_playing:
                    self.play()
            return track
        return None

    def previous_track(self) -> Optional[Track]:
        """Go to previous track."""
        if self.current_playlist:
            track = self.current_playlist.previous_track()
            if track:
                logger.info(f"Previous track: {track.title}")
                if self.playback.is_playing:
                    self.play()
            return track
        return None

    def create_demo_playlist(self) -> Playlist:
        """Create a demo playlist for testing."""
        demo_tracks = [
            Track(
                title="The Adventure Begins",
                artist="Story Time Stories",
                duration=180.0,
            ),
            Track(
                title="Journey to the Mountains",
                artist="Story Time Stories",
                duration=240.0,
            ),
            Track(
                title="The Magic Forest",
                artist="Bedtime Tales",
                duration=200.0,
            ),
            Track(
                title="Ocean Waves & Dreams",
                artist="Relaxing Sounds",
                duration=300.0,
            ),
        ]

        playlist = Playlist(name="Demo Playlist", tracks=demo_tracks)
        self.playlists["demo"] = playlist
        logger.info(f"Created demo playlist with {len(demo_tracks)} tracks")
        return playlist

    def get_playback_progress(self) -> float:
        """
        Get current playback progress as percentage.

        Returns:
            Progress from 0.0 to 1.0
        """
        track = self.get_current_track()
        if track and track.duration > 0:
            return min(1.0, self.playback.position / track.duration)
        return 0.0

    def update_system_status(
        self,
        battery: Optional[int] = None,
        signal: Optional[int] = None,
        connected: Optional[bool] = None
    ) -> None:
        """
        Update system status information.

        Args:
            battery: Battery percentage (0-100)
            signal: Signal strength (0-4)
            connected: Connection status
        """
        if battery is not None:
            self.battery_percent = max(0, min(100, battery))
        if signal is not None:
            self.signal_strength = max(0, min(4, signal))
        if connected is not None:
            self.is_connected = connected

    def update_power_status(self, snapshot: "PowerSnapshot") -> None:
        """Update cached power telemetry from the latest backend snapshot."""
        self.power_available = snapshot.available
        self.power_error = snapshot.error

        if snapshot.battery.level_percent is not None:
            level = round(snapshot.battery.level_percent)
            self.battery_percent = max(0, min(100, level))

        if snapshot.battery.charging is not None:
            self.battery_charging = snapshot.battery.charging

        if snapshot.battery.power_plugged is not None:
            self.external_power = snapshot.battery.power_plugged

    def update_voip_status(self, *, configured: bool, ready: bool) -> None:
        """Update cached VoIP availability used by the simplified chrome."""

        self.voip_configured = configured
        self.voip_ready = ready

    def update_network_status(
        self,
        *,
        network_enabled: bool | None = None,
        signal_bars: int | None = None,
        connection_type: str | None = None,
        connected: bool | None = None,
        gps_has_fix: bool | None = None,
    ) -> None:
        """Update cached network telemetry from the modem backend."""
        if network_enabled is not None:
            self.network_enabled = network_enabled
        if signal_bars is not None:
            self.signal_strength = max(0, min(4, signal_bars))
        if connection_type is not None:
            self.connection_type = connection_type
        if connected is not None:
            self.is_connected = connected
        if gps_has_fix is not None:
            self.gps_has_fix = gps_has_fix

    def update_screen_runtime(
        self,
        *,
        screen_awake: bool,
        app_uptime_seconds: float,
        screen_on_seconds: float,
        idle_seconds: float,
    ) -> None:
        """Update runtime metrics for app uptime and display activity."""
        self.screen_awake = screen_awake
        self.app_uptime_seconds = max(0, int(app_uptime_seconds))
        self.screen_on_seconds = max(0, int(screen_on_seconds))
        self.screen_idle_seconds = max(0, int(idle_seconds))

    def update_call_summary(self, *, missed_calls: int, recent_calls: list[str]) -> None:
        """Update Talk summary state used by the hub and call-related screens."""

        self.missed_calls = max(0, int(missed_calls))
        self.recent_calls = list(recent_calls)

    def set_talk_contact(self, *, name: str, sip_address: str) -> None:
        """Store the currently selected Talk contact."""

        self.talk_contact_name = name
        self.talk_contact_address = sip_address

    def set_voice_note_recipient(self, *, name: str, sip_address: str) -> None:
        """Store the currently selected voice-note recipient."""

        self.voice_note_recipient_name = name
        self.voice_note_recipient_address = sip_address
        self.voice_note_send_state = "idle"
        self.voice_note_status_text = ""
        self.voice_note_file_path = ""
        self.voice_note_duration_ms = 0

    def update_voice_note_summary(
        self,
        *,
        unread_voice_notes: int,
        latest_voice_note_by_contact: Dict[str, Dict[str, Any]],
    ) -> None:
        """Update unread counts and latest voice-note metadata exposed to Talk."""

        self.unread_voice_notes = max(0, int(unread_voice_notes))
        self.latest_voice_note_by_contact = dict(latest_voice_note_by_contact)

    def update_active_voice_note(
        self,
        *,
        send_state: str,
        status_text: str = "",
        file_path: str = "",
        duration_ms: int = 0,
    ) -> None:
        """Update the active voice-note UI state for the selected recipient."""

        self.voice_note_send_state = send_state
        self.voice_note_status_text = status_text
        self.voice_note_file_path = file_path
        self.voice_note_duration_ms = max(0, int(duration_ms))

    def configure_voice(
        self,
        *,
        commands_enabled: bool | None = None,
        ai_requests_enabled: bool | None = None,
        screen_read_enabled: bool | None = None,
        stt_enabled: bool | None = None,
        tts_enabled: bool | None = None,
        speaker_device_id: str | None | object = _VOICE_UNSET,
        capture_device_id: str | None | object = _VOICE_UNSET,
    ) -> None:
        """Update persistent voice feature toggles cached in context."""

        if commands_enabled is not None:
            self.voice.commands_enabled = commands_enabled
        if ai_requests_enabled is not None:
            self.voice.ai_requests_enabled = ai_requests_enabled
        if screen_read_enabled is not None:
            self.voice.screen_read_enabled = screen_read_enabled
        if stt_enabled is not None:
            self.voice.stt_enabled = stt_enabled
        if tts_enabled is not None:
            self.voice.tts_enabled = tts_enabled
        if speaker_device_id is not _VOICE_UNSET:
            self.voice.speaker_device_id = speaker_device_id  # type: ignore[assignment]
        if capture_device_id is not _VOICE_UNSET:
            self.voice.capture_device_id = capture_device_id  # type: ignore[assignment]

    def update_voice_backend_status(self, *, stt_available: bool, tts_available: bool) -> None:
        """Update backend availability flags used by voice UI flows."""

        self.voice.stt_available = stt_available
        self.voice.tts_available = tts_available

    def set_mic_muted(self, muted: bool) -> None:
        """Cache the app-facing microphone mute state."""

        self.voice.mic_muted = muted

    def toggle_mic_muted(self) -> bool:
        """Toggle and return the cached app-facing microphone mute state."""

        self.voice.mic_muted = not self.voice.mic_muted
        return self.voice.mic_muted

    def record_voice_transcript(self, transcript: str, *, mode: str) -> None:
        """Cache the latest transcript for command or AI request flows."""

        self.voice.last_transcript = transcript.strip()
        self.voice.last_mode = mode

    def record_voice_response(self, text: str) -> None:
        """Cache the latest spoken response text."""

        self.voice.last_spoken_text = text.strip()
