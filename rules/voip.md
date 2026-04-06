# VoIP (linphonec)

Applies to: `yoyopy/voip/**`

## Overview

Wraps `linphonec` CLI subprocess. Parses stdout for call state changes.

## Linphone 5.x Patterns

- Case-insensitive pattern matching for call state parsing
- Square brackets for SIP addresses: `[sip:user@domain]`
- Uses `"CallSession"` not `"Call"` in output
- VoIP monitor thread reads linphonec output continuously
- Callbacks fire on the coordinator thread for UI updates

## Configuration

SIP account config in `config/voip_config.yaml`:
- SIP account, transport, STUN server
- HA1 hash authentication (never plaintext passwords)
- Contact list and speed dial in `config/contacts.yaml`

## Audio

- Ring tone generated via `speaker-test` subprocess (800Hz on `plughw:1`)
- Audio device: WM8960 codec with dual MEMS microphones (on Whisplay HAT)
