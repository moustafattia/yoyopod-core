# Code Style

- Python 3.12+, type hints required on all function definitions
- Black formatting, 100 char line length
- Logging via `loguru` (not stdlib logging) -- see `rules/logging.md`
- Build system: hatchling
- Repo-owned staged quality gate: `uv run python scripts/quality.py gate`
- Full quality debt audit: `uv run python scripts/quality.py audit`
