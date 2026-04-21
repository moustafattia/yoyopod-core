"""Tests for display and input factory fallback to Cubie adapters."""

from unittest.mock import MagicMock

import pytest


def test_display_factory_falls_back_to_cubie_pimoroni_simulation():
    """When displayhatmini and GPIO config are both unavailable, use simulation."""
    from yoyopod.ui.display.factory import get_display

    # Without displayhatmini and without board config, pimoroni should fall back
    display = get_display(hardware="pimoroni", simulate=False)
    try:
        assert display.DISPLAY_TYPE == "pimoroni"
        assert display.WIDTH == 320
        assert display.HEIGHT == 240
        # Should be in simulate mode since neither displayhatmini nor spidev are available
        assert display.simulate is True
    finally:
        display.cleanup()


def test_input_factory_falls_back_to_gpiod_buttons():
    """When displayhatmini is unavailable, use GpiodButtonAdapter for pimoroni display."""
    from yoyopod.ui.input.factory import get_input_manager

    mock_display = MagicMock()
    mock_display.DISPLAY_TYPE = "pimoroni"
    mock_display.__class__.__name__ = "CubiePimoroniAdapter"
    mock_display.device = None

    config = {
        "input": {
            "pimoroni_gpio": {
                "button_a": {"chip": "gpiochip0", "line": 34},
                "button_b": {"chip": "gpiochip0", "line": 35},
                "button_x": {"chip": "gpiochip0", "line": 36},
                "button_y": {"chip": "gpiochip0", "line": 313},
            },
        },
    }

    manager = get_input_manager(mock_display, config=config, simulate=True)
    assert manager is not None
    assert len(manager.adapters) > 0
