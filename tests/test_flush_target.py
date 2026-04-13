"""Tests for the generic LVGL flush target protocol."""

import pytest


def test_display_hal_has_draw_rgb565_region():
    """DisplayHAL base class should provide a default draw_rgb565_region."""
    from yoyopy.ui.display.hal import DisplayHAL

    assert hasattr(DisplayHAL, "draw_rgb565_region")


def test_display_hal_has_get_flush_target():
    """DisplayHAL base class should provide get_flush_target defaulting to None."""
    from yoyopy.ui.display.hal import DisplayHAL

    assert hasattr(DisplayHAL, "get_flush_target")


def test_cubie_pimoroni_is_flush_target():
    """CubiePimoroniAdapter should return self as flush target."""
    from yoyopy.ui.display.adapters.cubie_pimoroni import CubiePimoroniAdapter

    adapter = CubiePimoroniAdapter(simulate=True)
    target = adapter.get_flush_target()
    assert target is None  # simulate=True -> no hardware -> no flush target
    assert hasattr(adapter, "draw_rgb565_region")
    assert hasattr(adapter, "WIDTH")
    assert hasattr(adapter, "HEIGHT")
    adapter.cleanup()


def test_simulation_get_flush_target_returns_none():
    """Simulation adapter returns None (no LVGL in CI)."""
    from yoyopy.ui.display.adapters.simulation import SimulationDisplayAdapter

    adapter = SimulationDisplayAdapter()
    target = adapter.get_flush_target()
    assert target is None
    adapter.cleanup()


def test_whisplay_has_flush_target_method():
    """WhisplayDisplayAdapter should have get_flush_target and draw_rgb565_region."""
    from yoyopy.ui.display.adapters.whisplay import WhisplayDisplayAdapter

    assert hasattr(WhisplayDisplayAdapter, "get_flush_target")
    assert hasattr(WhisplayDisplayAdapter, "draw_rgb565_region")
