"""Helpers for pooled LVGL scenes shared by multiple Python view wrappers."""

from __future__ import annotations

from typing import Protocol

from yoyopod.ui.lvgl_binding import LvglDisplayBackend


class RetainedLvglView(Protocol):
    """Structural type for LVGL views that share one retained native scene."""

    backend: LvglDisplayBackend
    scene_key: str
    _built: bool


def _retained_scene_claims(backend: LvglDisplayBackend) -> dict[str, int]:
    """Return the per-backend registry of current pooled scene owners."""

    claims = getattr(backend, "_retained_scene_claims", None)
    if claims is None:
        claims = {}
        setattr(backend, "_retained_scene_claims", claims)
    return claims


def retained_scene_claimed_by(view: RetainedLvglView) -> bool:
    """Return True when the pooled native scene is still owned by this view."""

    return _retained_scene_claims(view.backend).get(view.scene_key) == id(view)


def mark_retained_view_built(view: RetainedLvglView) -> None:
    """Record this view as the current owner of its pooled native scene."""

    view._built = True
    _retained_scene_claims(view.backend)[view.scene_key] = id(view)


def mark_retained_view_destroyed(view: RetainedLvglView) -> None:
    """Drop this view's native-scene ownership without disturbing newer claimants."""

    view._built = False
    claims = _retained_scene_claims(view.backend)
    if claims.get(view.scene_key) == id(view):
        claims.pop(view.scene_key, None)
