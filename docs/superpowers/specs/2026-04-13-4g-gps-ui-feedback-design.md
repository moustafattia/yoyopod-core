# 4G/GPS UI Feedback — Design Spec

**Date:** 2026-04-13
**Depends on:** 4G Cellular Connectivity (PR #81)

---

## Summary

Add visual feedback for the 4G modem and GPS module to the YoyoPod UI:

1. **Status bar indicators** — signal strength bars and GPS fix dot in the top chrome
2. **Setup screen pages** — Network and GPS detail pages in the existing Setup/Power screen

---

## Status Bar Indicators

**Layout (left to right):** Signal bars → GPS dot → VoIP dot → Time → Battery

### Signal Bars

Four vertical bars (3px wide each, increasing height), drawn in the status bar left area before the existing VoIP dot.

| State | Color | When |
|---|---|---|
| Online | Green (SUCCESS), filled bars matching `context.signal_strength` 0-4 | PPP is up |
| Registered | Grey (MUTED), filled bars matching signal | Modem registered, no PPP |
| No signal | Red (ERROR), no filled bars | CSQ 99 or modem unreachable |
| Hidden | Not rendered | `network.enabled = false` |

Unfilled bar positions render in dark grey (same as battery outline color).

### GPS Indicator

A small filled circle (3px radius) next to the signal bars, before the VoIP dot.

| State | Color | When |
|---|---|---|
| Fix acquired | Green (SUCCESS) | `context.gps_has_fix = True` |
| No fix | Grey (MUTED) | GPS enabled but no fix |
| Hidden | Not rendered | GPS disabled or network disabled |

---

## Setup Screen Pages

Two new pages added to the existing `PowerScreen`, inserted after the Power page.

**Page order:** Power → **Network** → **GPS** → Time → Care → Voice

Both pages use the existing `PowerPage(title, rows)` dataclass — no new screen class.

### Network Page

Title: "Network"

| Row Label | Value | Color Logic |
|---|---|---|
| Status | Online / Offline / Disabled | Green if online, red if offline, grey if disabled |
| Carrier | e.g. "Telekom.de" | Default (INK) |
| Type | 4G / 3G / 2G / "" | Default |
| Signal | "N/4" (e.g. "3/4") | Default |
| PPP | Up / Down | Green if up, red if down |

When network is disabled, show only: Status = "Disabled" (grey).

### GPS Page

Title: "GPS"

| Row Label | Value | Color Logic |
|---|---|---|
| Fix | Yes / No | Green if yes, red if no |
| Lat | Decimal degrees or "--" | Default or grey for "--" |
| Lng | Decimal degrees or "--" | Default or grey for "--" |
| Alt | Meters with "m" suffix or "--" | Default or grey |
| Speed | km/h with "km/h" suffix or "--" | Default or grey |

When GPS is disabled or no fix, show dashes for coordinate fields.

Data for both pages reads from `NetworkManager.modem_state` and `AppContext` at render time.

---

## AppContext Changes

Add one field:

```python
self.gps_has_fix: bool = False
```

Extend `update_network_status()` with one parameter:

```python
def update_network_status(
    self,
    *,
    signal_bars: int | None = None,
    connection_type: str | None = None,
    connected: bool | None = None,
    gps_has_fix: bool | None = None,
) -> None:
```

---

## Data Flow

1. `NetworkManager.start()` publishes `NetworkSignalUpdateEvent` and `NetworkPppUpEvent`
2. `app.py` subscribes to these events and calls `context.update_network_status()`
3. `render_status_bar()` reads `context.signal_strength`, `context.is_connected`, `context.connection_type`, `context.gps_has_fix`
4. `PowerScreen.build_pages()` reads `NetworkManager.modem_state` for the Network and GPS detail pages

---

## Files Modified

| File | Change |
|---|---|
| `yoyopy/ui/screens/theme.py` | Add signal bars and GPS dot rendering in `render_status_bar()` |
| `yoyopy/ui/screens/system/power.py` | Add Network and GPS pages to `build_pages()` |
| `yoyopy/app_context.py` | Add `gps_has_fix` field, extend `update_network_status()` |
| `yoyopy/app.py` | Subscribe to network events, update context on state changes |

## Files Not Changed

- `yoyopy/network/` — no changes to the network package itself
- No new screen classes — reuses existing `PowerPage` pattern
