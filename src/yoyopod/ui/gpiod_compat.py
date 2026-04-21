"""Backward-compatible import target for the legacy gpiod compatibility path."""

from __future__ import annotations

import sys

from yoyopod.device import gpiod_compat as _device_gpiod_compat

sys.modules[__name__] = _device_gpiod_compat
