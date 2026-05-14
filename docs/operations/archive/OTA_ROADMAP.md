# OTA Roadmap

The slot-deploy foundation was originally designed so a future OTA
daemon could be added without changing the core deploy pieces. As of
2026-05-13, slot-deploy itself is paused while the CLI is rebuilt in
Rust; see
[`../../ROADMAP.md`](../../ROADMAP.md). Round 3 restores the
slot pipeline. The OTA work below sits on top of Round 3.

## Recommended build order for OTA (once Round 3 lands)

1. **Manifest schema** — re-introduce a release-manifest type in Rust
   (the Python equivalent shipped under `yoyopod_cli/release_manifest.py`
   and was deleted in Round 0). The schema should keep `signature`,
   `channel`, `requires`, and diff-artifact fields ready.
2. **Atomic slot flip + rollback** — re-port the atomic symlink primitive
   into the new Rust slot tooling. `yoyopod-prod.service`'s
   `OnFailure=yoyopod-prod-rollback.service` wiring is preserved and
   already triggers rollback when a flipped slot crash-loops.
3. **Health probes** — `yoyopod target health preflight` and
   `... health live` (also in Round 3) are the entry points the OTA
   daemon will share with the CLI.
4. **Manifest signing** — minisign/ed25519, CI-only signing key. Verify
   on device before applying.
5. **Static OTA bucket** — Cloudflare R2 / S3. Upload `manifest.json`
   + release tarballs + diff patches on every CI release tag.
6. **`yoyopod ota check` command** — fetches the manifest, compares
   versions, prints the update. No side effects yet.
7. **`yoyopod ota apply`** — downloads, verifies signature, runs
   preflight, flips symlinks.
8. **systemd timer** — runs `yoyopod ota check && yoyopod ota apply`
   every 6 h on the stable channel.
9. **Diff patches** — cut tarball size by 10–50× for small changes.
10. **Channel rollout** — server-side manifest variants + a device-ID
    hash check on the client.

See [`SLOT_DEPLOY.md`](SLOT_DEPLOY.md) for the ops flow as it stood
before the rebuild; that doc is being updated alongside Round 3.
