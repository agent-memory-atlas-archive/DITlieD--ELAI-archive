Phase A-39: AS During-Generation Safety Defaults

What: ensure all during-generation features (steering vector, ETC, DoLa) ship with safe disabled defaults. Add [inference] config schema to elai_cc.toml. Create calibration script targeting Qwen2.5-Coder-7B.

Key decision: server.py already has correct defaults (verified by context-compiler). Primary work is additive: toml schema + calibration script. No server.py modifications expected.

Expected test outcome: existing 459 tests pass unchanged. Smoke check (--help) exits 0. No new test failures.

Unknowns: whether calibrate_etc_threshold.py already exists (context-compiler flagged HIGH). Whether tomlkit is installed (need pip check before using).
