"""PIL fallback view for the root Hub screen."""

from __future__ import annotations

from datetime import datetime
from math import cos, pi, sin
from typing import TYPE_CHECKING

from yoyopod.ui.screens.theme import (
    BACKGROUND,
    FOOTER_BAR,
    INK,
    MUTED_DIM,
    draw_icon,
    mix,
    render_backdrop,
    render_status_bar,
    rounded_panel,
)

if TYPE_CHECKING:
    from yoyopod.ui.screens.navigation.hub import HubScreen


def render_hub_pil(screen: "HubScreen") -> None:
    """Render the selected Hub card through the PIL display path."""

    cards = screen.cards()
    screen.selected_index %= len(cards)
    selected_card = cards[screen.selected_index]
    if selected_card.title == "Watch":
        _render_watch_face(screen)
        return

    render_backdrop(screen.display, selected_card.mode)
    render_status_bar(screen.display, screen.context, show_time=True)

    tile_size = 96
    tile_left = (screen.display.WIDTH - tile_size) // 2
    tile_top = screen.display.STATUS_BAR_HEIGHT + 30
    glow_padding = 10

    rounded_panel(
        screen.display,
        tile_left - glow_padding,
        tile_top - glow_padding,
        tile_left + tile_size + glow_padding,
        tile_top + tile_size + glow_padding,
        fill=screen.tile_glow_color(selected_card.mode),
        outline=None,
        radius=24,
        shadow=False,
    )

    rounded_panel(
        screen.display,
        tile_left,
        tile_top,
        tile_left + tile_size,
        tile_top + tile_size,
        fill=screen.tile_fill_color(selected_card.mode),
        outline=None,
        radius=16,
        shadow=True,
    )

    draw_icon(
        screen.display,
        selected_card.icon,
        tile_left + 20,
        tile_top + 20,
        56,
        INK,
    )

    title_y = tile_top + tile_size + 24
    title_text = selected_card.title
    title_width, title_height = screen.display.get_text_size(title_text, 22)
    screen.display.text(
        title_text,
        (screen.display.WIDTH - title_width) // 2,
        title_y,
        color=INK,
        font_size=22,
    )

    dots_y = title_y + title_height + 30
    dot_gap = 10
    dots_width = ((len(cards) - 1) * dot_gap) + 4
    dots_x = (screen.display.WIDTH - dots_width) // 2
    inactive_dot = mix(INK, BACKGROUND, 0.8)
    for index in range(len(cards)):
        dot_color = INK if index == screen.selected_index else inactive_dot
        screen.display.circle(dots_x + (index * dot_gap), dots_y, 2, fill=dot_color)

    footer_top = screen.display.HEIGHT - 32
    screen.display.rectangle(
        0, footer_top, screen.display.WIDTH, screen.display.HEIGHT, fill=FOOTER_BAR
    )
    footer_text = "Tap = Next / 2x = Open / Hold = Ask"
    footer_width, footer_height = screen.display.get_text_size(footer_text, 10)
    screen.display.text(
        footer_text,
        (screen.display.WIDTH - footer_width) // 2,
        footer_top + ((32 - footer_height) // 2) - 1,
        color=MUTED_DIM,
        font_size=10,
    )
    screen.display.update()


def _render_watch_face(screen: "HubScreen") -> None:
    """Render the selected watch face on the Watch hub card."""

    now = screen.watch_timestamp()
    battery_percent = screen.watch_battery_percent()
    is_charging = screen.watch_is_charging()
    face = screen.picker_watch_face() if screen.watch_picker_active else screen.active_watch_face()

    render_backdrop(screen.display, "setup")
    if face.key == "analog":
        _render_analog_face(screen, now)
    elif face.key == "activity":
        _render_activity_face(screen, now)
    else:
        _render_minimal_digital_face(screen, now)

    _draw_watch_chrome(screen, now, battery_percent, is_charging, picker_active=screen.watch_picker_active)
    _draw_watch_footer(screen, picker_active=screen.watch_picker_active)
    screen.display.update()


def _draw_watch_chrome(
    screen: "HubScreen",
    now: datetime,
    battery_percent: int,
    is_charging: bool,
    *,
    picker_active: bool,
) -> None:
    """Draw shared watch-face chrome (day/date + battery + picker label)."""

    day_text = now.strftime("%a").upper()
    date_text = now.strftime("%b %d").upper()
    label = f"{day_text} {date_text}"
    label_width, _ = screen.display.get_text_size(label, 16)
    screen.display.text(
        label,
        (screen.display.WIDTH - label_width) // 2,
        screen.display.STATUS_BAR_HEIGHT + 8,
        color=(72, 225, 255),
        font_size=16,
    )

    _draw_battery_chip(screen, battery_percent, is_charging)
    if picker_active:
        face_label = screen.picker_watch_face().label
        pill = f"Picker · {face_label}"
        width, _ = screen.display.get_text_size(pill, 12)
        screen.display.text(
            pill,
            (screen.display.WIDTH - width) // 2,
            screen.display.STATUS_BAR_HEIGHT + 28,
            color=(164, 171, 186),
            font_size=12,
        )


def _draw_battery_chip(screen: "HubScreen", battery_percent: int, is_charging: bool) -> None:
    """Draw top-left battery percent and a compact charging icon."""

    x = 10
    y = screen.display.STATUS_BAR_HEIGHT + 8
    screen.display.rectangle(x, y, x + 17, y + 10, outline=(206, 235, 246), width=1)
    screen.display.rectangle(x + 17, y + 3, x + 19, y + 7, fill=(206, 235, 246))
    fill_width = max(1, int((max(0, min(100, battery_percent)) / 100.0) * 15))
    fill_color = (70, 214, 149) if is_charging else (72, 225, 255)
    screen.display.rectangle(x + 1, y + 1, x + fill_width, y + 9, fill=fill_color)
    pct_label = f"{battery_percent}%"
    screen.display.text(
        pct_label,
        x + 25,
        y - 1,
        color=(235, 239, 247),
        font_size=14,
    )
    if is_charging:
        bolt = "⚡"
        screen.display.text(
            bolt,
            x + 58,
            y - 2,
            color=(70, 214, 149),
            font_size=14,
        )


def _render_minimal_digital_face(screen: "HubScreen", now: datetime) -> None:
    """Draw a minimal, high-contrast digital watch face."""

    time_text = now.strftime("%I:%M")
    hour_text, minute_text = time_text.split(":")
    am_pm = now.strftime("%p")
    x = 14
    y = screen.display.STATUS_BAR_HEIGHT + 62
    screen.display.text(hour_text, x, y, color=(232, 236, 244), font_size=64)
    screen.display.text(":", x + 98, y, color=(232, 236, 244), font_size=64)
    screen.display.text(minute_text, x + 122, y, color=(72, 225, 255), font_size=64)
    screen.display.text(am_pm, x + 192, y + 14, color=(163, 170, 185), font_size=18)


def _render_analog_face(screen: "HubScreen", now: datetime) -> None:
    """Draw an analog-style face with cyan accents."""

    center_x = screen.display.WIDTH // 2
    center_y = (screen.display.HEIGHT // 2) + 10
    radius = min(screen.display.WIDTH, screen.display.HEIGHT) // 3
    screen.display.circle(center_x, center_y, radius + 6, outline=(38, 129, 150), width=2)
    screen.display.circle(center_x, center_y, radius, outline=(220, 228, 238), width=1)

    for tick in range(60):
        angle = (tick / 60.0) * 2 * pi - (pi / 2)
        outer_x = int(center_x + cos(angle) * radius)
        outer_y = int(center_y + sin(angle) * radius)
        inner_radius = radius - (10 if tick % 5 == 0 else 5)
        inner_x = int(center_x + cos(angle) * inner_radius)
        inner_y = int(center_y + sin(angle) * inner_radius)
        color = (72, 225, 255) if tick % 15 == 0 else (210, 218, 230)
        screen.display.line(inner_x, inner_y, outer_x, outer_y, color=color, width=1)

    hour = now.hour % 12
    minute = now.minute
    second = now.second
    _draw_hand(screen, center_x, center_y, radius * 0.48, ((hour + minute / 60) / 12) * 360, (238, 240, 244), 4)
    _draw_hand(screen, center_x, center_y, radius * 0.72, (minute / 60) * 360, (238, 240, 244), 3)
    _draw_hand(screen, center_x, center_y, radius * 0.82, (second / 60) * 360, (72, 225, 255), 2)
    screen.display.circle(center_x, center_y, 5, fill=(238, 240, 244))


def _draw_hand(
    screen: "HubScreen",
    center_x: int,
    center_y: int,
    length: float,
    degrees: float,
    color: tuple[int, int, int],
    width: int,
) -> None:
    """Draw one analog hand from center with a degree-based angle."""

    angle = (degrees / 180.0) * pi - (pi / 2)
    end_x = int(center_x + cos(angle) * length)
    end_y = int(center_y + sin(angle) * length)
    screen.display.line(center_x, center_y, end_x, end_y, color=color, width=width)


def _render_activity_face(screen: "HubScreen", now: datetime) -> None:
    """Draw a playful activity-style digital face with rings."""

    time_text = now.strftime("%I:%M")
    screen.display.text(time_text, 20, screen.display.STATUS_BAR_HEIGHT + 58, color=(232, 236, 244), font_size=58)
    ring_center_y = screen.display.HEIGHT - 58
    centers = [42, 94, 146]
    ring_colors = [(255, 67, 133), (255, 178, 36), (109, 224, 52)]
    for cx, color in zip(centers, ring_colors):
        screen.display.circle(cx, ring_center_y, 22, outline=(58, 61, 67), width=8)
        screen.display.circle(cx, ring_center_y, 22, outline=color, width=5)

    # Small pseudo-metrics to complete the face without adding fake sensor dependencies.
    screen.display.text("MOVE", 172, ring_center_y - 20, color=(164, 171, 186), font_size=11)
    screen.display.text("CAL", 172, ring_center_y, color=(164, 171, 186), font_size=11)
    screen.display.text("MIND", 172, ring_center_y + 20, color=(164, 171, 186), font_size=11)


def _draw_watch_footer(screen: "HubScreen", *, picker_active: bool) -> None:
    """Draw mode-specific footer hints for Watch hub interactions."""

    footer_top = screen.display.HEIGHT - 32
    screen.display.rectangle(0, footer_top, screen.display.WIDTH, screen.display.HEIGHT, fill=FOOTER_BAR)
    footer_text = (
        "Tap = Next face / 2x = Choose / Hold = Hub"
        if picker_active
        else "Tap = Next card / 2x = Face picker / Hold = Ask"
    )
    width, height = screen.display.get_text_size(footer_text, 10)
    screen.display.text(
        footer_text,
        (screen.display.WIDTH - width) // 2,
        footer_top + ((32 - height) // 2) - 1,
        color=MUTED_DIM,
        font_size=10,
    )
