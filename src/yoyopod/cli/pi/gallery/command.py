"""CLI command entrypoint for deterministic Whisplay gallery captures."""

from __future__ import annotations

from pathlib import Path
from typing import Annotated

import typer

from yoyopod.cli.common import configure_logging
from yoyopod.cli.pi.gallery.capture import _build_capture_specs, _capture_screen

gallery_app = typer.Typer(
    name="gallery",
    help="Capture a deterministic gallery of Whisplay LVGL screens.",
    invoke_without_command=True,
    no_args_is_help=False,
)


_ASK_RESPONSE_HEADLINE = "Volume"
_ASK_RESPONSE_BODY = "Volume is 75."


def _advance_ask_to_response(screen: object) -> None:
    """Set AskScreen into a stable reply state for deterministic captures."""

    set_response = getattr(screen, "_set_response", None)
    if not callable(set_response):
        raise TypeError("Ask response capture requires a screen with _set_response(headline, body)")
    set_response(_ASK_RESPONSE_HEADLINE, _ASK_RESPONSE_BODY)


@gallery_app.callback(invoke_without_command=True)
def gallery(
    output_dir: Annotated[
        str, typer.Option("--output-dir", help="Directory where PNG captures should be written.")
    ] = "temp/pi_gallery",
    simulate: Annotated[
        bool,
        typer.Option(
            "--simulate",
            help="Use the Whisplay adapter in simulation mode instead of driving hardware.",
        ),
    ] = False,
    settle_seconds: Annotated[
        float,
        typer.Option("--settle-seconds", help="How long to let LVGL settle before each capture."),
    ] = 0.18,
    verbose: Annotated[bool, typer.Option("--verbose", help="Enable DEBUG logging.")] = False,
) -> None:
    """Capture a deterministic gallery of Whisplay LVGL screens."""
    from loguru import logger

    from yoyopod.ui.display import Display

    configure_logging(verbose)

    output_path = Path(output_dir).expanduser().resolve()
    output_path.mkdir(parents=True, exist_ok=True)

    display = Display(
        hardware="whisplay",
        simulate=simulate,
        whisplay_renderer="lvgl",
    )
    backend = display.get_ui_backend()
    if backend is None or not getattr(backend, "available", False):
        logger.error("LVGL backend unavailable. Build it first with `uv run yoyoctl build lvgl`.")
        raise typer.Exit(code=1)
    if not backend.initialize():
        logger.error("Failed to initialize the Whisplay LVGL backend")
        raise typer.Exit(code=1)
    display.refresh_backend_kind()

    try:
        specs = _build_capture_specs(display, advance_ask_to_response=_advance_ask_to_response)
        for spec in specs:
            _capture_screen(
                display,
                spec,
                output_path,
                settle_seconds=settle_seconds,
            )
    finally:
        display.cleanup()

    saved_count = len(list(output_path.glob("*.png")))
    logger.info("Saved {} screenshots to {}", saved_count, output_path)
