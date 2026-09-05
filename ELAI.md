# ELAI.md

This file provides guidance to ELAI when working with code in this repository.

## Detected stack
- Languages: Python.
- Frameworks: none detected from the supported starter markers.

## Verification
- Run the Python project checks declared in `pyproject.toml` (for example: `pytest`, `ruff check`, and `mypy` when configured).

## Working agreement
- Prefer small, reviewable changes and keep generated bootstrap files aligned with actual repo workflows.
- Keep shared defaults in `.elai.json`; reserve `.elai/settings.local.json` for machine-local overrides.
- Do not overwrite existing `ELAI.md` content automatically; update it intentionally when repo workflows change.
