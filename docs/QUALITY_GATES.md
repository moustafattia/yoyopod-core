# Quality Gates

This repo now owns one executable quality contract instead of relying on tribal knowledge:

```bash
uv run python scripts/quality.py gate
```

CI runs that exact command.

## Gated now

The staged gate currently covers the developer-workflow surface tracked in `[tool.yoyopod_quality]` inside `pyproject.toml`:

- `scripts/quality.py`
- `yoyopy/main.py`
- `yoyopy/cli/__init__.py`
- `yoyopy/cli/common.py`
- `yoyopy/cli/build.py`
- `yoyopy/cli/remote/`

The gate enforces:

- `black --check`
- `ruff check`
- `mypy`

That means pull requests can no longer regress the entrypoint and Pi workflow tooling without CI calling it out.

## Ungated now

Everything outside the staged target lists above is still outside the lint/type/format gate.

That is deliberate, not hidden:

- full-repo `black --check` still wants to rewrite a large chunk of the tree
- full-repo `ruff check .` still reports existing violations outside the gated workflow surface
- full-repo `mypy yoyopy` still reports substantial legacy type debt outside the gated workflow surface

You can measure the current full-repo debt with:

```bash
uv run python scripts/quality.py audit
```

## Path to full gating

The rollout path is explicit:

1. Clean one subsystem at a time with `uv run python scripts/quality.py audit`.
2. Expand the `[tool.yoyopod_quality]` gate target lists in `pyproject.toml`.
3. Once the repo is clean enough, replace the staged lists with whole-tree targets.

The point is to make progress visible and enforceable without pretending the repo is already cleaner than it is.
