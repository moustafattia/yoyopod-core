# Code Style

- Python 3.12+, type hints required on all function definitions
- Black formatting, 100 char line length
- Logging via `loguru` (not stdlib logging) -- see `rules/logging.md`
- Build system: hatchling
- Linting: `ruff check .`
- Type checking: `mypy yoyopy/`
