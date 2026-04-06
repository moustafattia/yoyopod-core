"""Focused tests for the Whisplay adapter hot paths."""

from __future__ import annotations

from yoyopy.ui.display.adapters.whisplay import WhisplayDisplayAdapter


class FakeDevice:
    """Small device double that records RGB565 writes."""

    def __init__(self) -> None:
        self.draw_calls: list[tuple[int, int, int, int, bytes]] = []

    def draw_image(
        self,
        x: int,
        y: int,
        width: int,
        height: int,
        pixel_data: bytes,
    ) -> None:
        self.draw_calls.append((x, y, width, height, pixel_data))


def test_hardware_lvgl_flush_skips_shadow_buffer_sync(monkeypatch) -> None:
    """Hardware LVGL flushes should not mirror every region into PIL."""

    adapter = WhisplayDisplayAdapter(simulate=True, renderer="lvgl")
    adapter.simulate = False
    adapter.ui_backend = type("Backend", (), {"available": True})()
    adapter.device = FakeDevice()

    mirrored: list[tuple[int, int, int, int, bytes]] = []
    monkeypatch.setattr(
        adapter,
        "_paste_rgb565_region",
        lambda x, y, width, height, pixel_data: mirrored.append((x, y, width, height, pixel_data)),
    )

    payload = b"\x12\x34" * 8
    adapter.draw_rgb565_region(1, 2, 4, 2, payload)

    assert adapter.device.draw_calls == [(1, 2, 4, 2, payload)]
    assert mirrored == []


def test_simulated_lvgl_flush_keeps_shadow_buffer_for_debug(monkeypatch) -> None:
    """Simulation still needs the PIL shadow buffer for local inspection."""

    adapter = WhisplayDisplayAdapter(simulate=True, renderer="lvgl")

    mirrored: list[tuple[int, int, int, int, bytes]] = []
    monkeypatch.setattr(
        adapter,
        "_paste_rgb565_region",
        lambda x, y, width, height, pixel_data: mirrored.append((x, y, width, height, pixel_data)),
    )

    payload = b"\xAA\x55" * 8
    adapter.draw_rgb565_region(0, 0, 4, 2, payload)

    assert mirrored == [(0, 0, 4, 2, payload)]


def test_shadow_screenshot_falls_back_to_lvgl_readback_when_shadow_sync_is_disabled(monkeypatch) -> None:
    """Hardware LVGL screenshots should use readback instead of per-flush shadow sync."""

    adapter = WhisplayDisplayAdapter(simulate=True, renderer="lvgl")
    adapter.simulate = False
    adapter.ui_backend = type("Backend", (), {"available": True})()

    readback_calls: list[str] = []
    monkeypatch.setattr(
        adapter,
        "save_screenshot_readback",
        lambda path: readback_calls.append(path) or True,
    )

    assert adapter.save_screenshot("/tmp/test.png") is True
    assert readback_calls == ["/tmp/test.png"]


def test_shadow_sync_mode_tracks_runtime_fallback_state_changes() -> None:
    """The sync mode should follow later renderer/simulation fallback changes."""

    adapter = WhisplayDisplayAdapter(simulate=True, renderer="lvgl")

    assert adapter.shadow_buffer_sync_enabled is True

    adapter.simulate = False
    adapter.renderer = "lvgl"
    adapter.ui_backend = type("Backend", (), {"available": True})()
    assert adapter.shadow_buffer_sync_enabled is False

    adapter.renderer = "pil"
    assert adapter.shadow_buffer_sync_enabled is True

    adapter.renderer = "lvgl"
    adapter.ui_backend = type("Backend", (), {"available": False})()
    assert adapter.shadow_buffer_sync_enabled is True
