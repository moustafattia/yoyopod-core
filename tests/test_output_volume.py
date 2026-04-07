from __future__ import annotations

import subprocess

from yoyopy.audio.music.backend import MockMusicBackend
from yoyopy.audio.volume import OutputVolumeController


def test_output_volume_controller_parses_system_volume(monkeypatch) -> None:
    def fake_run(args: list[str], **_kwargs) -> subprocess.CompletedProcess[str]:
        return subprocess.CompletedProcess(
            args=args,
            returncode=0,
            stdout="""
Simple mixer control 'Master',0
  Front Left: Playback 50462 [77%] [on]
  Front Right: Playback 50462 [77%] [on]
""",
            stderr="",
        )

    monkeypatch.setattr("yoyopy.audio.volume.subprocess.run", fake_run)

    controller = OutputVolumeController()

    assert controller.get_system_volume() == 77
    assert controller.get_volume() == 77


def test_output_volume_controller_sets_system_and_music_volume(monkeypatch) -> None:
    calls: list[list[str]] = []

    def fake_run(args: list[str], **_kwargs) -> subprocess.CompletedProcess[str]:
        calls.append(args)
        return subprocess.CompletedProcess(args=args, returncode=0, stdout="", stderr="")

    monkeypatch.setattr("yoyopy.audio.volume.subprocess.run", fake_run)

    backend = MockMusicBackend()
    backend.start()
    controller = OutputVolumeController(music_backend=backend)

    assert controller.set_volume(55) is True
    assert calls[0] == ["amixer", "sset", "Master", "55%"]
    assert backend.get_volume() == 55


def test_output_volume_controller_falls_back_to_music_backend_when_amixer_missing(
    monkeypatch,
) -> None:
    def fake_run(_args: list[str], **_kwargs) -> subprocess.CompletedProcess[str]:
        raise FileNotFoundError("amixer")

    monkeypatch.setattr("yoyopy.audio.volume.subprocess.run", fake_run)

    backend = MockMusicBackend()
    backend.start()
    controller = OutputVolumeController(music_backend=backend)

    assert controller.set_volume(68) is True
    assert controller.get_volume() == 68
    assert backend.get_volume() == 68


def test_output_volume_controller_falls_back_to_card_one_when_default_card_fails(
    monkeypatch,
) -> None:
    calls: list[list[str]] = []

    def fake_run(args: list[str], **_kwargs) -> subprocess.CompletedProcess[str]:
        calls.append(args)
        if args in (
            ["amixer", "sget", "Master"],
            ["amixer", "-c", "1", "sget", "Master"],
            ["amixer", "-c", "0", "sget", "Master"],
        ):
            return subprocess.CompletedProcess(
                args=args,
                returncode=1,
                stdout="",
                stderr="amixer: Unable to find simple control 'Master',0",
            )
        return subprocess.CompletedProcess(
            args=args,
            returncode=0,
            stdout="""
Simple mixer control 'Master',0
  Front Left: Playback 65536 [100%] [on]
""",
            stderr="",
        )

    monkeypatch.setattr("yoyopy.audio.volume.subprocess.run", fake_run)

    controller = OutputVolumeController()

    assert controller.get_system_volume() == 100
    assert calls[:4] == [
        ["amixer", "sget", "Master"],
        ["amixer", "-c", "1", "sget", "Master"],
        ["amixer", "-c", "0", "sget", "Master"],
        ["amixer", "-c", "1", "sget", "Speaker"],
    ]
