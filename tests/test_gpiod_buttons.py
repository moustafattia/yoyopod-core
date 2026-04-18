"""Unit tests for the gpiod-based 4-button input adapter."""

from __future__ import annotations

import pytest

from yoyopod.ui.input.hal import InputAction


def test_adapter_capabilities():
    from yoyopod.ui.input.adapters.gpiod_buttons import GpiodButtonAdapter

    adapter = GpiodButtonAdapter(pin_config={}, simulate=True)
    caps = adapter.get_capabilities()
    assert InputAction.SELECT in caps
    assert InputAction.BACK in caps
    assert InputAction.UP in caps
    assert InputAction.DOWN in caps
    assert InputAction.HOME in caps


def test_adapter_fires_callback_on_simulate():
    from yoyopod.ui.input.adapters.gpiod_buttons import GpiodButtonAdapter

    adapter = GpiodButtonAdapter(pin_config={}, simulate=True)
    received = []
    adapter.on_action(InputAction.SELECT, lambda data: received.append(("select", data)))
    adapter._fire_action(InputAction.SELECT, {"button": "A"})
    assert len(received) == 1
    assert received[0] == ("select", {"button": "A"})


def test_clear_callbacks():
    from yoyopod.ui.input.adapters.gpiod_buttons import GpiodButtonAdapter

    adapter = GpiodButtonAdapter(pin_config={}, simulate=True)
    adapter.on_action(InputAction.SELECT, lambda data: None)
    assert len(adapter.callbacks) > 0
    adapter.clear_callbacks()
    assert len(adapter.callbacks) == 0


def test_adapter_start_stop_lifecycle():
    from yoyopod.ui.input.adapters.gpiod_buttons import GpiodButtonAdapter

    adapter = GpiodButtonAdapter(pin_config={}, simulate=True)
    adapter.start()
    assert adapter.running is True
    adapter.stop()
    assert adapter.running is False


def test_debounced_release_emits_action_without_busy_polling() -> None:
    from yoyopod.ui.input.adapters.gpiod_buttons import Button, GpiodButtonAdapter

    adapter = GpiodButtonAdapter(pin_config={}, simulate=True)
    received: list[tuple[str, dict[str, str]]] = []
    adapter.on_action(
        InputAction.SELECT,
        lambda data: received.append(("select", data)),
    )

    adapter._observe_raw_state(Button.A, True, 1.0)
    adapter._advance_button_state(Button.A, 1.06)
    adapter._observe_raw_state(Button.A, False, 1.20)
    adapter._advance_button_state(Button.A, 1.26)

    assert received == [("select", {"button": "A"})]


def test_edge_wait_loop_blocks_between_events(monkeypatch: pytest.MonkeyPatch) -> None:
    from yoyopod.ui.input.adapters import gpiod_buttons

    class FakeLine:
        def __init__(self, fd: int) -> None:
            self.fd = fd

        def get_value(self) -> int:
            return 1

        def release(self) -> None:
            return None

    class FakeChip:
        def close(self) -> None:
            return None

    line_by_offset = {
        10: FakeLine(101),
        11: FakeLine(102),
        12: FakeLine(103),
        13: FakeLine(104),
    }
    timeouts: list[float] = []

    def fake_select(reads, _writes, _errors, timeout):
        timeouts.append(timeout)
        adapter._stop_event.set()
        return ([], [], [])

    monkeypatch.setattr(gpiod_buttons, "HAS_GPIOD", True)
    monkeypatch.setattr(gpiod_buttons, "open_chip", lambda _name: FakeChip())
    monkeypatch.setattr(
        gpiod_buttons,
        "request_input_events",
        lambda _chip, line_offset, _consumer: line_by_offset[line_offset],
    )
    monkeypatch.setattr(gpiod_buttons, "get_event_fd", lambda line: line.fd)
    monkeypatch.setattr(gpiod_buttons, "read_edge_events", lambda _line: [])
    monkeypatch.setattr(gpiod_buttons.select, "select", fake_select)

    adapter = gpiod_buttons.GpiodButtonAdapter(
        pin_config={
            "button_a": {"chip": "gpiochip0", "line": 10},
            "button_b": {"chip": "gpiochip0", "line": 11},
            "button_x": {"chip": "gpiochip0", "line": 12},
            "button_y": {"chip": "gpiochip0", "line": 13},
        },
        simulate=False,
    )

    adapter._poll_loop()

    assert timeouts == [gpiod_buttons._EDGE_IDLE_WAIT_TIMEOUT]
