# Q-5: per-turn output-token-hash journaling on the local lane — localize replay divergence to the first divergent turn, and make the RRA claim boundary honest

STATUS: PLANNED (WAVE 1; EXTENSION(G-2 RRA + J journal-completeness); CHEAP — one hash per turn on the local lane)

- Track: ENGINEERING-TASTE | lever: hardware-independent-determinism-is-an-engineering-choice | source: bellard-taste V2a (ts_zip verbatim determinism claim, verified 2026-07-10) | grounding: STRONG (bellard.org/ts_zip fetched 2026-07-10) | nature: EXTENSION | effort: S-M | WAVE 1
- PROOF_SPINE: hardens the K re-executable-RRA differentiator (tightens the replay-claim honesty + adds first-divergent-turn localization); no direct PS-1..6 number — trust-spine durability class. Not a product-surface/distribution `none`; it declares a spine relationship (the K differentiator), so no separate ack is required.

- Premise (bellard-taste V2a): ts_zip demonstrates HARDWARE-INDEPENDENT deterministic LLM evaluation is an engineering CHOICE, not a law ("the result does not depend on the exact GPU or CPU model nor on the number of configured threads" — bellard.org/ts_zip, fetched 2026-07-10). ELAI's replay is hardware-CLASS-scoped: G-2 RRA pins the SSB action stream + seed + HardwareClass + model pin + gate-verdict hashes, the cloud lane is a reproducible ACTION-STREAM not tokens (GD-5), but the LOCAL-lane token stream is only asserted deterministic within a hardware class. The cheap fix: journal a per-turn output-token-HASH (blake3 of the emitted token-id sequence) on the local lane. Payoff: `replay --verify` localizes divergence to the FIRST divergent turn instead of a diverged end-state; a same-class replay becomes token-verifiable; a cross-class replay reports precisely WHERE the exogenous model diverged (composes with G's model-drift canary). It also makes the RRA CLAIM honest: "byte-recomputable" is true of the harness stream, not of local token streams across hardware — the hash makes that boundary explicit (claims-policy KD-1).

## Steal/build
On the local lane, after each turn, compute `blake3(emitted_token_id_sequence)` and journal it as a field on the EXISTING J-28 per-turn usage row (reuse the row; add `output_token_hash: Option<[u8;32]>` rather than a new action_type where the schema allows — cheaper, W-9 baseline re-read at edit). Optionally a top-k logit hash where the backend exposes logits (Tier-2). `elai replay --verify` compares the recomputed per-turn hash against the journaled one and reports the FIRST turn where they diverge, with its hardware-class + model-pin context. The RRA docs/claims state exactly which layers replay byte-identically on which scope (harness stream: anywhere; local tokens: same-class, hash-verified; cloud: action-stream).
EXCLUDED: hashing on the CLOUD lane (cloud is a reproducible action-stream by design, GD-5 — a token hash there is meaningless and would imply a determinism ELAI does not claim); a NEW heavy determinism mode (that is V2b/Q — recon-only, NOT this unit; blake3 never blake2/sha2); asserting hardware-INDEPENDENT determinism (the hash localizes divergence, it does not eliminate cross-hardware nondeterminism — GD-5 honesty).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- G-2 RRA: SSB stream + seed + HardwareClass + model pin + gate-verdict hashes; `elai replay --verify` re-derives — the replay path Q-5 tightens.
- GD-5: cloud lane is a reproducible action-stream not tokens — the boundary Q-5 makes explicit rather than overturns.
- J-28 usage_turn_event (the uniform per-turn usage row, commit fe069d0) — the row Q-5 extends with the token hash (reuse, no new subsystem).
- the sidecar threads sampling_seed; local-lane token determinism is mentioned (systems-overview) — the same-class scope the hash verifies.
- G model-drift canary — the cross-class divergence Q-5's per-turn localization composes with.
- blake3 (CLAUDE.md hard rule) — the hash function; never blake2/sha2.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the per-turn `blake3(token_id_seq)` on the local lane journaled on the J-28 row; `replay --verify` first-divergent-turn localization; the RRA claim-wording tightening (which layer replays on which scope).
- DEFER (Tier-2): the top-k logit hash (backends that expose logits) + the V2b heavy deterministic-eval llama.cpp mode is OUT of scope entirely (recon-only per the steal doc; a separate future unit after its own recon).

## Integration
- crates/orchestrator or crates/providers (the local-lane turn boundary): compute + attach the token hash to the J-28 usage row; crates/cli replay: the per-turn compare + first-divergence report.
- frontend_pair: N/A with justification — the divergence report is a CLI/replay artifact + the SSB field; no new panel. Justified.

## MOVES (wargame)
M1 at the local-lane turn boundary, compute `blake3(emitted_token_id_sequence)` and attach it to the J-28 per-turn usage row (field, not a new row where the schema allows).
EXPECTED_OBSERVATION: a local-lane turn journals a usage row carrying a 32-byte blake3 output_token_hash; a cloud-lane turn carries NONE (the hash is local-only).
ON_FAIL: the hash is attached on the cloud lane -> implies a determinism ELAI does not claim (GD-5) -> gate the hash on lane==local; assert the cloud row has no token hash.
M2 in `replay --verify`, recompute the per-turn hash and compare to the journaled one, reporting the FIRST divergent turn with its hardware-class + model-pin.
EXPECTED_OBSERVATION: a same-class same-seed replay verifies every turn hash (no divergence); an induced divergence (change the seed on one turn) is reported at exactly that turn index, not as an end-state mismatch.
ON_FAIL: divergence is reported only as a final-state mismatch -> the first-divergent-turn payoff is lost -> compare per-turn and short-circuit at the first mismatch; assert the reported turn index.
M3 tighten the RRA claim wording (docs + the replay report): harness action-stream replays anywhere; local token stream replays same-class (hash-verified); cloud is action-stream. Run the claims lint.
EXPECTED_OBSERVATION: the RRA claim copy states the per-layer replay scope; `claims_policy_lint.py` passes (no unscoped "byte-recomputable" over local tokens across hardware).
ON_FAIL: the copy still claims whole-session byte-recomputability unscoped -> KD-1 overclaim -> scope the claim per layer; the lint blocks the unscoped form.

## FORK
FORK: if the local backend does not expose the emitted token-ID sequence at the turn boundary (only decoded text) -> hash the decoded-text bytes as a WEAKER same-class signal and label it text-hash (not token-hash) in the row + the claim; record the seam. Do NOT claim token-level determinism from a text hash. If neither token-ids nor stable decoded text are available, this is a material fork -> STOP + handoff.

## RECON_NEEDED
RECON_NEEDED-1: whether the local backend exposes emitted token-IDs at the turn boundary -> settle via `grep -rn "token_id\|logits\|generate\|emit" .elai_cc/crates/providers/src` + the sidecar turn API.
RECON_NEEDED-2: the J-28 usage_turn_event row shape + whether a field-add vs a new action_type is required at the LIVE schema -> settle via `grep -rn "usage_turn_event\|UsageTurn" .elai_cc/crates/orchestrator/src/ssb` (re-read schema_version per W-9).
RECON_NEEDED-3: the `replay --verify` per-turn iteration seam -> settle via `grep -rn "replay\|verify\|RRA\|re.derive" .elai_cc/crates/cli/src`.

## ABORT
ABORT: if attaching the token hash would require moving a wall-clock / nondeterministic read into the replay set, or would imply the cloud lane is token-deterministic -> STOP. The hash is a LOCAL-lane same-class verifier + an honesty boundary; it must not overstate the determinism ELAI claims (GD-5).

## WIRING_CONTRACT
produces_symbols:
  - the per-turn `blake3(token_id_seq)` computation on the local lane + its field on the J-28 usage row (or a new hash row if a field-add is not schema-legal; re-read at edit W-9)
  - the `replay --verify` first-divergent-turn localization
  - the scoped RRA claim wording (per-layer replay scope)
consumes_symbols:
  - G-2 RRA + `elai replay --verify`; J-28 usage_turn_event row; GD-5 cloud-action-stream boundary; G model-drift canary; blake3
reachable_from:
  - a local-lane turn on a live run journals the hash [PRODUCTION local turn]; `elai replay --verify` reads + compares it [PRODUCTION replay CLI]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO token hash on the cloud lane (GD-5)
  - NO blake2/sha2 (blake3 only)
  - NO unscoped "byte-recomputable" claim over local tokens across hardware (KD-1)
  - NO claim of hardware-INDEPENDENT determinism (the hash localizes, it does not eliminate cross-class nondeterminism)
observable_effect:
  trigger: "run a local-lane turn, then `elai replay --verify` on the run, then re-run with a one-turn seed change"
  expect_ssb: "the local usage row carries a 32-byte blake3 output_token_hash; the cloud row carries none"
  expect_artifact: "replay --verify reports no divergence on a faithful same-class replay and the EXACT first divergent turn on the seed-changed replay; NEGATIVE: divergence reported only as an end-state mismatch, or a token hash on a cloud turn, must FAIL"

PRODUCTION_HOOK: producer the per-turn hash -> consumer the local-lane turn-journaling seam that writes the J-28 row (RECON-2), wired same-changeset; producer the localization -> consumer `replay --verify` (RECON-3); the hashed usage row is the off-agent artifact.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-5_local_token_hash_journal.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
`python .claude/tools/claims_policy_lint.py --files <RRA claim copy> --json`   # per-layer replay scope, no unscoped byte-recomputable
pass = a local-lane run journaled per-turn blake3 hashes off-agent, `replay --verify` verified them on a faithful replay and localized an induced divergence to the exact turn, and no cloud row carried a token hash.

INTEGRATION_SMOKE: an independent agent runs a real multi-turn local-lane task (a HARD input, long enough for several turns), verifies the full run replays with all per-turn hashes matching, then induces a single-turn divergence and confirms `replay --verify` names that turn — the real turn-journaling + replay path, not a synthetic hash (R10).

## REDTEAM
REDTEAM: attack that FAILED — "claim a whole ELAI session is byte-recomputable on any hardware": FAILS because M3 + the claims lint scope the claim per layer (harness anywhere / local same-class / cloud action-stream), so the overclaim cannot ship. Attack that SUCCEEDED and its patch — "hide a local-lane nondeterminism by only checking the end-state": patched by M2's per-turn compare that surfaces the FIRST divergent turn, turning a silent end-state drift into a precisely-located, canary-composable signal.

## Grounding
- bellard-taste-elai-2026-07-10.md V2a + the ts_zip verbatim determinism claim (bellard.org/ts_zip, fetched 2026-07-10).
- G-2 RRA / GD-5 cloud-action-stream / J-28 usage_turn_event (fe069d0) / G model-drift canary / blake3 hard rule — feature-map + git-log-confirmed at authoring, executor re-greps (W-7).

## COMPOSES-WITH
G-2 RRA + `replay --verify`, J-28 usage_turn_event, GD-5 boundary, G model-drift canary, blake3. Re-implements NONE (GD-1). The V2b heavy deterministic-eval llama.cpp mode is explicitly OUT (recon-only). Pairs with Q-6 (both are trust-spine durability/honesty units) and the K re-executable-RRA differentiator it hardens.

End of Q-5.
