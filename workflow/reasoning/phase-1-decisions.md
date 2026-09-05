# Phase 1 decision doc — PART6 §3.4 LMFE detail constant extraction

Session: s-46448
Phase: §3.4 (carried-over from s-1976592 PART6, WARN-3)
Date: 2026-05-07
GRC topic: lmfe-constant-extract (3 calls / 600 chars)

What I will implement:
Add module-level constant `_ERR_DETAIL_INSTALL_LMFE` in `.elai_cc/python/elai_inference/server.py` directly under `_ERR_CONSTRAINED_DECODING_UNAVAILABLE` (currently at L115). Replace the byte-identical 4-line implicit-concatenated operator-install-hint detail string at the two 503 sites (L2841-2845 in `handle_messages`, L2924-2928 in `handle_completions`) with `"detail": _ERR_DETAIL_INSTALL_LMFE,`. Net: +5 / -10 LOC.

Key technical decision:
Preserve the existing string verbatim (operator-facing hint copy, deliberate per PART6 acceptance — "borderline Rule 15 but minor"). Single canonical constant, two call-site references, one substring concentration — applicationRules §1 / codeRules Rule 17.

Expected gate outcomes:
- G1 (PRE): grep "Run: pip install " server.py → 2 hits before edit (verified).
- G2 (POST): grep "Run: pip install " server.py → exactly 1 hit (constant declaration only).
- G3 (POST): grep "_ERR_DETAIL_INSTALL_LMFE" server.py → exactly 3 hits (1 decl + 2 usages).
- G4 (POST): python -m py_compile server.py → exit 0.
- G5 (POST, OFFLINE): cargo check -p elai_cc_contracts → clean (no Rust changes; smoke-only).

Unknowns to verify empirically:
- Whether ruff is available in the local venv (offline gate fallback if not).
- Whether `cargo check` picks up unrelated drift from the 270-uncommitted-file working tree (PART6's pre-existing HashMap/BTreeMap mismatch in fu_build_dispatch_context_phase_slot_contract.rs noted as not-introduced-by-us).

Constraints (user, this session):
NO COMMITS. 1 audit cycle max. Sidecar offline → audit limited to compile/grep/ruff (no live SSE/curl). §3.3 streaming gate test deferred next session.
