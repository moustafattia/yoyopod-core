# Release Process

YoYoPod now uses one explicit release contract:

- package version source: `yoyopod/_version.py`
- release tag format: `vMAJOR.MINOR.PATCH`
- release artifacts: Python package distributions plus a full repo release bundle
- release automation: `.github/workflows/release.yml`

## Versioning

YoYoPod uses semantic versioning:

- `major` for breaking or externally visible contract changes
- `minor` for backward-compatible feature releases
- `patch` for backward-compatible fixes and polish

Check the current version:

```bash
uv run yoyopod release current
```

Bump the version source:

```bash
uv run yoyopod release bump patch
uv run yoyopod release bump minor
uv run yoyopod release bump major
```

Or set it explicitly:

```bash
uv run yoyopod release set-version 0.2.0
```

## Local Release Build

Build the release artifacts locally:

```bash
uv run yoyopod release build
```

That command builds:

- Python wheel
- Python source distribution
- full repo bundle as `.tar.gz`
- full repo bundle as `.zip`
- `release-metadata.json`
- `SHA256SUMS.txt`

By default it refuses to build from a dirty tracked worktree. For local-only experiments, you can override that:

```bash
uv run yoyopod release build --allow-dirty
```

## GitHub Release Pipeline

The release workflow runs on:

- push of a tag like `v0.2.0`
- manual dispatch with an existing release tag

The workflow:

1. checks out the tagged revision
2. installs the dev environment with `uv`
3. runs `uv run python scripts/quality.py gate`
4. runs `uv run pytest -q`
5. runs `uv run yoyopod release build --check-tag <tag>`
6. uploads the built artifacts
7. creates or updates the matching GitHub Release

## Recommended Release Flow

1. Bump the version:

   ```bash
   uv run yoyopod release bump patch
   ```

2. Run the repo gates:

   ```bash
   uv run python scripts/quality.py gate
   uv run pytest -q
   ```

3. Commit and merge the version bump.
4. Tag the release from the merged commit:

   ```bash
   git tag v0.1.1
   git push origin v0.1.1
   ```

5. Let the release workflow build and publish the artifacts.

## Notes

- The Python package artifacts are useful for packaging and inspection.
- The repo bundles are the more complete release asset for hardware-oriented YoYoPod workflows because they include the broader repo surface the device and deploy tooling rely on.
