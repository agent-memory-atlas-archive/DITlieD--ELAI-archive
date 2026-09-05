# Q-8: locked size/dependency ceiling on the M-plan standalone verifier — the thing that audits everything must stay smaller than what it audits

STATUS: PLANNED (WAVE 3; NEW-small; discipline class — a locked, lint-enforced ceiling, K-4 gate-tax style)

- Track: ENGINEERING-TASTE | lever: single-binary-dependency-light-doctrine-made-a-number | source: bellard-taste V4 (TextSynth/LibNC single-binary doctrine, verified 2026-07-10) | grounding: STRONG (bellard.org/ts_server fetched 2026-07-10; M-plan standalone-verifier design) | nature: NEW-small | effort: S | WAVE 3
- PROOF_SPINE: none (discipline). Owner DELEGATED the authoring call 2026-07-10 ("i dont really know, you can make this call") — recorded as the PSR-2 ack by explicit delegation. Does not feed a PS-1..6 number; it is an enforced maintainability property on the M-plan verifier.

- Premise (bellard-taste V4): Bellard's single-binary, dependency-light doctrine ("all included in a single binary; very few external dependencies; Python not needed" — bellard.org/ts_server, fetched 2026-07-10; LibNC is zero-dep + deterministic). ELAI's M-plan standalone OSS verifier + open evidence standard exist precisely so a THIRD PARTY can verify a run with a small, boring tool. But there is no MEASURED ceiling on that tool's size — every new letter adds verdict types, and the verifier is the ONE component that must stay smaller than what it audits. V4 converts "small, auditable verifier" from posture into an ENFORCED property: a locked number in the K-4 gate-tax style. It fits as an M-plan ACCEPTANCE CRITERION; Q-8 authors the ceiling + its lint here (in the Q batch) cross-linked to M, rather than opening M for a one-criterion edit.

## Steal/build
The M-plan standalone verifier builds as a single static binary, zero network, zero Python; its CRATE-COUNT and BINARY-SIZE are recorded and CEILING-LOCKED (a named constant, K-4-gate-tax idiom), lint-checked in CI so a later letter cannot silently BLOAT the thing that has to be smaller than what it audits. `VerifierSizeMeasured{crate_count, binary_bytes, ceiling_crate_count, ceiling_bytes}` is emitted; a build that crosses either ceiling is a fail-closed BLOCK. The ceilings stay PENDING (recorded, not enforced-as-BLOCK) until a real M-plan verifier build sets the baseline — exactly the K-4 pattern where a ceiling is pending a real journal.
EXCLUDED: a network or Python dependency in the verifier build (the whole point — the verifier must be boring + third-party-runnable; TextSynth itself is REJECTED as a dependency, binary-only + non-commercial, verified); a ceiling that is a soft warning (it is a fail-closed BLOCK once the baseline is set, like K-4's --enforce-ceiling); asserting a size NUMBER as a claim before a real build measures it (KD-1 — pending until measured).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- M-plan standalone OSS verifier + the versioned open evidence standard — the component Q-8 puts a ceiling on (the design; a real build sets the baseline).
- K-4 composed gate-tax instrument (coding_task_gate_tax + audit_gate_tax vs LOCKED ceilings, GateTaxMeasured schema_version 271, --enforce-ceiling fail-closed exit, commit d1eb94a) — the EXACT locked-ceiling-pending-a-real-journal idiom Q-8 mirrors.
- E-51 (MLX + llama.cpp single-binary local lane) + 4M RuntimeAdapter — the Bellard-adjacent single-binary posture already in the corpus.
- the claims-policy lint (K-1/KD-1) — the number is pending until a real build measures it.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the size/crate-count MEASUREMENT of the M verifier build + `VerifierSizeMeasured` + the ceiling CONSTANTS (pending, per K-4) + the CI lint that BLOCKS a build crossing a SET ceiling + the zero-network/zero-Python assertion on the build.
- DEFER (Tier-2): SETTING the enforced ceiling values (waits on a real M-plan verifier build to establish the baseline — until then the ceilings are recorded PENDING, never a fabricated number).

## Integration
- .claude/tools or the M-verifier build (crates/): the size/crate-count measurement + the CI lint check + the ceiling constants.
- crates/cli: `elai bench verifier-size` (or the M verifier build step) emits `VerifierSizeMeasured`.
- frontend_pair: N/A with justification — a build-time CI number + SSB row; no panel. Justified.

## MOVES (wargame)
M1 measure the M verifier build: crate-count (from the lockfile/dep tree) + static-binary byte size; assert zero network + zero Python in the build; emit `VerifierSizeMeasured`.
EXPECTED_OBSERVATION: the build reports a crate_count + binary_bytes; a build that pulls a network dep or a Python runtime is REFUSED (the boring-tool invariant is structural).
ON_FAIL: a network/Python dep slips in -> the verifier is no longer third-party-boring -> fail the build on any network/Python dep; assert a deliberately-added net dep is refused.
M2 add the ceiling constants (K-4 style) + the CI lint: a build crossing a SET ceiling is a fail-closed BLOCK; while the ceiling is PENDING (no baseline yet) the lint reports the measured number without blocking (K-4 RunnerPresent-on-dry-journal pattern).
EXPECTED_OBSERVATION: with a SET ceiling, a bloated build BLOCKS; with a PENDING ceiling, the build reports the number and passes (never a fabricated ceiling).
ON_FAIL: a pending ceiling is treated as 0 and blocks everything, OR a soft-warns-only on a set ceiling -> mis-enforcement -> pending = report-not-block, set = fail-closed BLOCK; assert both arms.
M3 cross-link the ceiling to M-plan as its verifier acceptance criterion (a one-line reference in the M index + this file), so the M verifier's done-gate inherits the ceiling.
EXPECTED_OBSERVATION: the M-plan verifier acceptance criteria reference the Q-8 ceiling; the ceiling is the single source (no second size number elsewhere).
ON_FAIL: M defines its own size number separately -> two masters -> M references the Q-8 constant; the ceiling lives once.

## FORK
FORK: if the M standalone verifier is not yet BUILT (design only) -> ship Q-8 as the MEASUREMENT + lint machinery with the ceiling recorded PENDING (no enforced value), and the enforced ceiling is SET when the first real verifier build lands (a follow-up one-line constant change, gated on the real build). Do NOT invent a ceiling number before a real build measures the baseline (KD-1).

## RECON_NEEDED
RECON_NEEDED-1: whether the M standalone verifier has a real build target yet, or is design-only -> settle via reading docs/thefinalglance/M-plan + `grep -rn "verifier\|standalone\|evidence.standard" .elai_cc/crates` (decides Tier-1 ship shape vs PENDING).
RECON_NEEDED-2: the crate-count + static-binary size measurement method (cargo tree / cargo bloat / lockfile parse) that is deterministic + CI-runnable -> settle via checking the repo's existing size/dep tooling before adding a dep (anti-slopsquat any new tool).
RECON_NEEDED-3: the K-4 --enforce-ceiling / pending-ceiling pattern to mirror -> settle via reading crates/cli/src/measure.rs (the K-4 gate-tax runner) so Q-8 reuses the exact pending/enforced semantics.

## ABORT
ABORT: if enforcing the ceiling would require ADDING a heavy dependency to MEASURE it (a size tool that itself bloats the tree) -> STOP; the measurement must be lockfile/cargo-native + deterministic. A verifier-size gate that bloats the build to check the build's size is self-defeating.

## WIRING_CONTRACT
produces_symbols:
  - the M-verifier size/crate-count measurement + the zero-network/zero-Python build assertion
  - `VerifierSizeMeasured` SSB row + the ceiling constants (pending per K-4) + the CI lint (baseline+1; re-read at edit W-9)
  - the M-plan cross-link (the ceiling is M's verifier acceptance criterion)
consumes_symbols:
  - M-plan standalone verifier + open evidence standard; K-4 gate-tax pending/enforced-ceiling idiom (measure.rs); the claims-policy lint (KD-1); E-51/4M single-binary posture
reachable_from:
  - the M-verifier build / `elai bench verifier-size` emits the number + runs the lint [PRODUCTION build/CI]; a crossed set-ceiling BLOCKS the build
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO network or Python dependency in the verifier build (zero-dep boring-tool invariant)
  - NO fabricated ceiling number before a real build measures the baseline (pending per K-4; KD-1)
  - NO soft-warn on a SET ceiling (fail-closed BLOCK, like K-4 --enforce-ceiling)
  - NO second verifier-size number elsewhere (the constant lives once; M references it)
observable_effect:
  trigger: "build the M standalone verifier (or `elai bench verifier-size`), then a build that adds a bloat dep past a SET ceiling"
  expect_ssb: "VerifierSizeMeasured{crate_count, binary_bytes, ceiling_*}; a build over a SET ceiling BLOCKS, a PENDING ceiling reports-not-blocks"
  expect_artifact: "a lint verdict tied to the recorded ceiling; NEGATIVE: a network/Python dep in the verifier build, or a fabricated ceiling before a real build, must FAIL"

PRODUCTION_HOOK: producer the size measurement + lint + `VerifierSizeMeasured` -> consumer the M-verifier build step / `elai bench verifier-size` CI, wired same-changeset; the measured row + the CI exit code are the off-agent artifacts.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-8_verifier_size_ceiling.md --run-trigger`
`python .claude/tools/claims_policy_lint.py --files <the M verifier-size copy> --json`   # no fabricated size number; pending tagged
pass = a `VerifierSizeMeasured` row landed off-agent from a real verifier build with zero network/Python deps, the CI lint enforced a SET ceiling as a BLOCK (or reported a PENDING ceiling without a fabricated value), and the M-plan cross-link points at the single Q-8 constant.

INTEGRATION_SMOKE: an independent agent builds the REAL M verifier (or the closest shipped standalone target), confirms the measured size + zero-dep assertion, then adds a deliberate bloat dependency and confirms the CI lint BLOCKS once a ceiling is set — a real build, not a mocked size (R10). If the verifier is design-only (FORK), the smoke drives the measurement machinery on the nearest real static binary and records the PENDING state.

## REDTEAM
REDTEAM: attack that FAILED — "sneak a network dependency into the verifier so it phones home during third-party verification": FAILS because M1's zero-network/zero-Python build assertion refuses the dep at build time — the boring-tool invariant is structural, not advisory. Attack that SUCCEEDED and its patch — "claim a tiny verifier size before anything is built": patched by the K-4 pending-ceiling pattern + the claims lint that BLOCKS a fabricated number; the ceiling is PENDING until a real build sets the baseline, and the number is always journal-measured, never asserted.

## Grounding
- bellard-taste-elai-2026-07-10.md V4 + A2 (the evidence standard has no anti-bloat number) + the TextSynth/LibNC single-binary facts (bellard.org/ts_server, fetched 2026-07-10; TextSynth REJECTED as a dep, license-verified).
- K-4 gate-tax --enforce-ceiling / pending-ceiling (measure.rs, GateTaxMeasured schema_version 271, d1eb94a), M-plan standalone verifier, E-51/4M single-binary posture, claims-policy lint — confirmed at authoring, executor re-greps (W-7).

## COMPOSES-WITH
M-plan standalone verifier + open evidence standard, K-4 gate-tax pending/enforced-ceiling idiom, claims-policy lint, E-51/4M single-binary posture. Re-implements NONE (GD-1). The ceiling is M's verifier acceptance criterion, authored here to avoid opening M for one line.

End of Q-8.
