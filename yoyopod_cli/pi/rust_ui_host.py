"""Whisplay-only Rust UI host validation command."""

from __future__ import annotations

from pathlib import Path
from typing import Annotated, Literal, cast

import typer

from yoyopod.ui.rust_host.hub import HubRenderer, RustHubSnapshot
from yoyopod.ui.rust_host.protocol import UiEnvelope
from yoyopod.ui.rust_host.supervisor import RustUiHostSupervisor


def _default_worker_path() -> Path:
    suffix = ".exe" if __import__("os").name == "nt" else ""
    return Path("src") / "ui-host" / "build" / f"yoyopod-ui-host{suffix}"


def rust_ui_host(
    worker: Annotated[
        Path,
        typer.Option("--worker", help="Path to the Rust UI host binary."),
    ] = _default_worker_path(),
    frames: Annotated[
        int,
        typer.Option("--frames", min=1, help="Number of test scene frames to send."),
    ] = 10,
    hardware: Annotated[
        str,
        typer.Option("--hardware", help="Worker hardware mode: mock or whisplay."),
    ] = "whisplay",
    screen: Annotated[
        str,
        typer.Option("--screen", help="Screen to render: test-scene or hub."),
    ] = "test-scene",
    hub_renderer: Annotated[
        str,
        typer.Option(
            "--hub-renderer",
            help="Hub renderer: auto, lvgl, or framebuffer.",
        ),
    ] = "auto",
) -> None:
    """Run the Rust UI host against Whisplay hardware."""

    selected_screen = _screen_name(screen)
    selected_hub_renderer = _hub_renderer(hub_renderer)
    argv = [str(worker), "--hardware", hardware]
    supervisor = RustUiHostSupervisor(argv=argv)
    ready = supervisor.start()
    typer.echo(f"Rust UI host ready: {ready.payload}")

    try:
        for counter in range(1, frames + 1):
            if selected_screen == "hub":
                supervisor.send(
                    UiEnvelope.command(
                        "ui.show_hub",
                        RustHubSnapshot.static().to_payload(renderer=selected_hub_renderer),
                        request_id=f"hub-frame-{counter}",
                    )
                )
            else:
                supervisor.send(
                    UiEnvelope.command(
                        "ui.show_test_scene",
                        {"counter": counter},
                        request_id=f"frame-{counter}",
                    )
                )
        supervisor.send(UiEnvelope.command("ui.health", request_id="health"))
        health = supervisor.read_event()
        typer.echo(
            "Rust UI host health: "
            f"frames={health.payload.get('frames')} "
            f"button_events={health.payload.get('button_events')} "
            f"last_hub_renderer={health.payload.get('last_hub_renderer', '')}"
        )
    finally:
        supervisor.stop()


ScreenName = Literal["test-scene", "hub"]


def _screen_name(value: str) -> ScreenName:
    if value in {"test-scene", "hub"}:
        return cast(ScreenName, value)
    raise typer.BadParameter("screen must be test-scene or hub")


def _hub_renderer(value: str) -> HubRenderer:
    if value in {"auto", "lvgl", "framebuffer"}:
        return cast(HubRenderer, value)
    raise typer.BadParameter("hub-renderer must be auto, lvgl, or framebuffer")
