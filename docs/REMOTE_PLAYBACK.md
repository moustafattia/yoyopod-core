# YoYoPod Remote Playback On Device

This document describes the device-side playback behavior for `yoyo-py`.

## Contract

Topics:

- `yoyopod/{deviceId}/cmd`
- `yoyopod/{deviceId}/ack`
- `yoyopod/{deviceId}/evt`

Rules:

- `ack` topic is only for command acceptance or rejection
- `evt` topic is only for playback lifecycle

Accepted playback commands:

- `play_track`
- `pause`
- `resume`
- `stop`

## ACK Payloads

```json
{
  "command_id": "cmd_123",
  "status": "ack",
  "payload": {
    "command": "play_track"
  }
}
```

```json
{
  "command_id": "cmd_123",
  "status": "nack",
  "reason": "invalid_command",
  "payload": {}
}
```

## Playback Event Payloads

Lifecycle events are published on `evt` with `type: "playback"`.

Supported `eventType` values:

- `buffering`
- `playing`
- `paused`
- `stopped`
- `completed`
- `failed`

## Caching

Remote playback now uses a bounded local cache:

- cache key includes `track_id`
- checksum is verified when provided
- least-recently-used pruning is based on file mtime
- cache size is bounded by config

Relevant config fields:

- `media.music.remote_cache_dir`
- `media.music.remote_cache_max_bytes`

Operational behavior:

- first remote play downloads and verifies the asset
- repeated plays use the cached local file when available
- playback runs against the local cached file, not the signed backend URL

## Playback Behavior

- only one active remote playback session is tracked
- a new valid `play_track` interrupts current playback
- `pause` and `resume` use the current mpv backend path
- duplicate `commandId` values are ACKed as duplicates and not replayed

## Current Operational Caveat

The real Pi used for validation currently lacks a working LVGL production shim build, so cloud/media validation was performed with the app running on the actual Pi in `--simulate` display mode. Audio, MQTT, remote fetch, and cache behavior were validated on-device; the display contract remains a separate operational restore task.

## Validation Notes

Observed real-device validation on `piz` confirmed:

- `ack`
- `buffering`
- `playing`
- `paused`
- `resume -> playing`
- `stop -> stopped`
- long-track playback beyond the short device token lifetime
- cache miss on first long-track play
- cache hit on repeated long-track play
