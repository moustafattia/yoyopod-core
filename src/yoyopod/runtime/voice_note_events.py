"""Runtime-owned voice-note context and screen synchronization helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from yoyopod.app import YoyoPodApp


class VoiceNoteEventHandler:
    """Keep shared voice-note state aligned with the active VoIP session."""

    def __init__(self, app: "YoyoPodApp") -> None:
        self.app = app

    def handle_voice_note_summary_changed(
        self,
        unread_voice_notes: int,
        latest_voice_note_by_contact: dict[str, dict[str, object]],
    ) -> None:
        """Keep Talk voice-note summary state in sync with the VoIP manager."""

        if self.app.context is None:
            return
        self.app.context.update_voice_note_summary(
            unread_voice_notes=unread_voice_notes,
            latest_voice_note_by_contact=latest_voice_note_by_contact,
        )
        self.refresh_talk_related_screen()

    def handle_voice_note_activity_changed(self, *_args: Any) -> None:
        """Refresh active draft state after a message or delivery update."""

        self.sync_active_voice_note_context()
        self.sync_talk_summary_context()
        self.refresh_talk_related_screen()

    def handle_voice_note_failure(self, *_args: Any) -> None:
        """Refresh draft state after a failed message operation."""

        self.sync_active_voice_note_context()
        self.refresh_talk_related_screen()

    def sync_active_voice_note_context(self) -> None:
        """Mirror the active voice-note draft into the shared app context."""

        if self.app.context is None or self.app.voip_manager is None:
            return
        draft = self.app.voip_manager.get_active_voice_note()
        if draft is None:
            self.app.context.update_active_voice_note(send_state="idle")
            return
        self.app.context.update_active_voice_note(
            send_state=draft.send_state,
            status_text=draft.status_text,
            file_path=draft.file_path,
            duration_ms=draft.duration_ms,
        )

    def sync_talk_summary_context(self) -> None:
        """Refresh Talk summary counts and latest voice-note metadata in AppContext."""

        if self.app.context is None or self.app.call_history_store is None:
            return

        self.app.context.update_call_summary(
            missed_calls=self.app.call_history_store.missed_count(),
            recent_calls=self.app.call_history_store.recent_preview(),
        )
        if self.app.voip_manager is not None:
            self.app.context.update_voice_note_summary(
                unread_voice_notes=self.app.voip_manager.unread_voice_note_count(),
                latest_voice_note_by_contact=self.app.voip_manager.latest_voice_note_summary(),
            )

    def refresh_talk_related_screen(self) -> None:
        """Re-render Talk screens when their message state changes."""

        if self.app.screen_manager is None:
            return
        current_screen = self.app.screen_manager.get_current_screen()
        if current_screen is None:
            return
        if current_screen.route_name in {"call", "talk_contact", "voice_note"}:
            self.app.screen_manager.refresh_current_screen()
