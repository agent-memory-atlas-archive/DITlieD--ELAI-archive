# P-19: config/ops lifecycle + `elai doctor` + verified backups

STATUS: PLANNED (WAVE OC-2; NEW-area (composition-heavy) — ELAI's thinnest area; EXTENSION of J config-ingress + M + 3H; PROOF_SPINE: none (ops), owner-acked 2026-07-11; execute after the proof spine per A-7)

- Track OPENCLAW-STEAL | lever recoverable-config-and-state | grounding STRONG (openclaw's most underrated subsystem, source-verified; ELAI's thinnest area) | nature NEW-area | effort M-L | WAVE OC-2 | PROOF_SPINE none (ops)
- Premise: a trust product that corrupts its own config or cannot restore state undermines the trust story cheaply. openclaw's config/ops lifecycle is its most underrated subsystem and ELAI's thinnest. Add recoverable config, an `elai doctor`, verified journal backups, and a restart continuation queue.

## Context — why this matters
Grounded in workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md 2.6 + 2.17 + steals S5/S13/S14 + C2. openclaw: strict-schema fail-closed boot (unknown key = REFUSE), atomic writes, last-known-good with doctor-only restore, clobber guards (>50% shrink rejected, rejected write preserved as .rejected.<ts>), hot-reload with settle + invalid-edit rejection; a doctor detect/repair/lint triple with STABLE exit codes for CI; VACUUM-INTO WAL-safe SQLite snapshots + manifest verify; a restart continuation queue resuming one turn through restart; a compaction split-rule (keeps tool-call/result pairs together) + successor transcripts (never rewrite in place). CORRECTION C2: openclaw copies full process.env into the workflow context with NO allowlist — do the OPPOSITE (allowlist env); adopt their cwd path-jail (case-fold-before-compare on win32) verbatim, it is correct.

## Existing ground truth (B..Q coverage; PLAN-asserted; executor re-greps W-7)
- J config-ingress admission gate (J-SUMMARY, read-access+time): today it ADMITS; it should also make bad writes recoverable + journaled. EXTEND.
- M signed release channel + auto-updater + support bundle (M-SUMMARY): the doctor + backup live here.
- 3H session handoff (spec-overview Phase 3H): the compaction split-rule + successor-transcript — make explicit as stated invariants.
- Q ops hardening (Q-8 verifier size/dep ceiling; Q-9 distribution hygiene): the ops-discipline sibling.
- ELAI already worships deterministic exit codes (K gate-tax, plan_gate_check) — extend the same contract to `elai doctor`.

## Steal/build (REVISED scope, W-10 tiered)
- SHIP NOW (Tier-1): (a) a fail-closed-default-object config posture (policy objects fail closed even when ABSENT, cf openclaw ssrfPolicy) + last-known-good + rejected-write preservation (.rejected.<ts>) + shrink/clobber guards (>50% shrink rejected) + atomic writes, on the J config-ingress gate; (b) `elai doctor` detect/repair/lint with STABLE exit codes + structured findings; ENV IS ALLOWLISTED (invert C2), cwd path-jail adopted verbatim.
- DEFER (Tier-2): VACUUM-INTO WAL-safe verified SSB/session backups + manifest verify; the restart continuation queue (one in-flight turn survives a restart); post-upgrade compat probes; the compaction split-rule + successor-transcript stated invariants (likely already implied in 3H — make explicit).
- EXCLUDED as vaporware: openclaw's full-env-copy (C2 bug — env allowlist instead); any config write that overwrites on >50% shrink.

## Reads (mandatory before coding)
- workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md (2.6, 2.17; steals S5/S13/S14; C2)
- docs/thefinalglance/J-plan (config-ingress gate), M-plan (release/support), 3H (session handoff)
- crates/orchestrator/src/ (the config ingress + session store), crates/cli/src/ (the doctor entrypoint)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/config/lifecycle.rs (ConfigLifecycle: last-known-good + rejected-write + shrink/clobber guard + atomic write + env allowlist)
  - crates/cli/src/doctor_cmd.rs (elai doctor detect/repair/lint with stable exit codes + structured findings)
consumes_symbols:
  - crates/orchestrator/src/config/ingress.rs::ConfigIngressGate (J; EXTEND to recoverable — do NOT re-implement admission)
  - crates/orchestrator/src/session/store.rs::SessionStore (3H; the compaction split-rule + successor-transcript target)
reachable_from:
  - crates/cli/src/main.rs (`elai doctor` subcommand) [PRODUCTION CLI]
  - crates/orchestrator/src/config/ingress.rs (the admission path invoking the lifecycle guards) [PRODUCTION config ingress]
  - a #[cfg(test)] or contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO config write that overwrites on >50% shrink (rejected + preserved as .rejected.<ts>)
  - NO full process.env copy into the workflow context (env is ALLOWLISTED — invert the openclaw C2 bug)
  - NO doctor exit code that varies run-to-run for the same state (stable exit codes for CI)
  - NO compaction that splits a tool-call from its result; NO in-place transcript rewrite (successor-transcript); NO .unwrap on prod; NO blake2/sha2
frontend_pair:
  N/A with justification: `elai doctor` is a CLI + ops surface; doctor output is structured text with stable exit codes (CI-facing), not a dashboard tile (W-6 N/A). Justified.
observable_effect:
  trigger: "elai doctor lint --config tests/fixtures/config/shrink_50pct.toml"
  expect_ssb: "config_write_rejected"
  expect_artifact: "the >50%-shrink write is REJECTED and preserved as .rejected.<ts> with a config_write_rejected row + `elai doctor lint` returns a STABLE nonzero exit + structured doctor_finding rows; NEGATIVE: a >50% shrink that overwrites, a doctor exit code that varies run-to-run, or a full-env copy, FAILS"

## Wargame (plan-standard unit shape)
EXPECTED_OBSERVATION: writing a config that shrinks the file >50% is REJECTED and preserved as .rejected.<ts> with a config_write_rejected row; `elai doctor lint` returns a stable nonzero exit + structured doctor_finding rows for the same bad state every run.
ON_FAIL: the shrink write overwrites, or doctor's exit code varies for the same input (CAUSE: no clobber guard, or non-deterministic lint ordering). COUNTER-MOVE: add the >50%-shrink guard + .rejected.<ts> preservation; sort doctor findings deterministically and map to fixed exit codes; the negative tests must FAIL.
FORK: if a config is corrupt beyond a last-known-good restore, `elai doctor repair` refuses to auto-repair and surfaces a structured finding for human action rather than guessing a value; do NOT silently write a default over user config.
RECON_NEEDED: confirm the exact J ConfigIngressGate + 3H SessionStore symbols to extend, and openclaw's VACUUM-INTO + restart-continuation shape. Settling check: `grep -n "ConfigIngressGate\|SessionStore" crates/orchestrator/src/` + source-read openclaw doctor/backup.
ABORT: if a config-ingress path could ever admit an unknown key silently (drop fail-closed boot) or copy full env, STOP and flag — both are the openclaw failure modes (strict-schema boot; C2) this unit exists to beat.
VERIFY: the load-bearing proof is the off-agent config_write_rejected + doctor_finding rows + the stable doctor exit code, driven by the real `elai doctor` entrypoint. Run:
```bash
python .claude/tools/plan_gate_check.py --file docs/thefinalglance/P-plan/P-19_config_ops_lifecycle_doctor.md   # authoring gate (exit 0)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-19_config_ops_lifecycle_doctor.md   # execution done-gate
```
REDTEAM: surviving-attack seed — a truncating write that would zero a config is defeated by the >50%-shrink clobber guard (rejected + preserved). Attack that would succeed if unpatched: a config edit that smuggles a secret via a full-env copy (C2) — PATCH: env is allowlisted; secrets live in the P-10 vault, never in config or the journal. Deep red-team DEFERRED to execution pre-flight.

## INTEGRATION_SMOKE
An independent agent drives `elai doctor` + a config round-trip on a HARD input: a malformed config with an unknown key + a >50% shrink + a kill mid-turn, asserting fail-closed boot rejects the unknown key, the shrink is preserved not overwritten, the doctor exit code is stable across 3 runs, and the in-flight turn resumes once after restart. Toy single-lint does NOT satisfy R10.

## Grounding / citations
- Source: workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md 2.6/2.17, steals S5/S13/S14, C2 (openclaw config/ops source-verified there).
- ELAI ground truth (PLAN-asserted, executor re-greps W-7): J config-ingress, M, 3H, Q per the indexes cited.
- CLAUDE.md hard rules: deterministic exit codes; env allowlist (invert C2); blake3; no .unwrap on prod; no new dep without confirmation.

## PRODUCTION_HOOK
crates/orchestrator/src/config/ingress.rs — the admission path invokes ConfigLifecycle's guards on every write; `elai doctor` in crates/cli/src/main.rs drives detect/repair/lint. Both are wired into their production seams in this changeset.

## Next
Closes O3 (config/ops lifecycle — ELAI's thinnest area vs the trust story it sells). Pairs with M (packaging/support) and Q (ops hardening); the journal-backup leg protects the product's own evidence store.

End of P-19.
