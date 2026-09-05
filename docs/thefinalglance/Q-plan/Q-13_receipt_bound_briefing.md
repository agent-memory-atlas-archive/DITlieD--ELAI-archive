# Q-13: receipt-bound Cosmos briefing lines — every compiled briefing line resolves a fetched-byte-hash or journal-row-id receipt at compile time, or is dropped and counted

STATUS: PLANNED (WAVE AIDE2-2 sibling; single small unit; EXTENSION of the shipped receipt/citation spine onto the briefing surface. Trust infrastructure — improves grounding, not search efficiency.)

Authoring provenance: paper-to-plan PHASE 5 over the AIDE² deep-research steal (conjecture engine run 2026-07-15; pick C11, user-sequenced 2026-07-15 — "excellent trust infrastructure ... improves grounding rather than search efficiency"). Auditors do NOT read the engine run artifact — anchors below are re-greppable in-tree (W-7).

- Track: AIDE2-RESEARCH-INSTRUMENT-ECONOMICS | lever: fabricated-briefing-line-made-unrepresentable | source: ELAI-precedent composition (H-52 anchor resolver + E-48 verified citation), extended to the Cosmos briefing surface; dossier §31 output-guard is adjacent design context | grounding: STRONG (gap grep-confirmed in-tree; pure ELAI-pattern composition, no external effect claim) | nature: EXTENSION (binds an existing trust pattern to an unbound surface) | effort: S | WAVE AIDE2-2 sibling
- PROOF_SPINE: none (trust-infrastructure class; it improves briefing grounding, not a spine number). User-acked 2026-07-15 (in-session sequencing ruling authorizing this build IS the ack).

- Premise: ELAI's chat answer path is receipt-bound (E-48 `VerifiedCitation`, H-52 anchor resolver), but the Cosmos briefing surface is NOT — `grep -icE "receipt|citation|row_id"` in `cosmos_worker.rs` = 0. So a compiled briefing line can assert a claim with no resolvable provenance, an asymmetry with the chat path. C11 closes it: every compiled briefing line carries a receipt (a fetched-byte-hash of its source or a journal-row-id) resolved AT COMPILE TIME; a line whose receipt does not resolve is DROPPED and COUNTED, never silently emitted. A fabricated briefing line becomes structurally unrepresentable. Pure ELAI-pattern composition — no external benefit claim.

## Steal/build

Bind each compiled Cosmos briefing line to a receipt at `compile_cosmos_briefing` time: the line's source must resolve to either a fetched-byte-hash (blake3 of the fetched bytes it summarizes) or a journal-row-id (the SSB/ledger row it derives from), reusing the H-52 anchor-resolver + E-48 verified-citation machinery (GD-1, do NOT re-implement a citation type). An unresolvable line is dropped and the drop is counted into a `dropped_receipt_lines` tally emitted with the briefing. Ships default-ON behind an `ELAI_COSMOS_RECEIPT_BIND_DISABLE` kill-switch (W-2 kill-switch, never a default-OFF enable; R9 N/A, no allowlist row).

DUP-RISK DECLARED: expected verdict is EXTENSION; if RECON-1 finds the briefing surface ALREADY consumes E-48/H-52 (a functional-equivalent receipt binding already present), C11 is a premise-false kill and collapses to wiring the drop-count only. The engine premise grep (0 hits) says the NAMED binding is absent, not that no equivalent exists — RECON settles it before building.

EXCLUDED: a second/parallel citation type family (reuse E-48, RECON-2); an LLM judge deciding receipt validity (resolution is deterministic hash/row lookup); silent emission of an unresolvable line under any flag; a receipt that binds to the briefing OUTPUT rather than the SOURCE (that would be self-certifying, the vetoed shape).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)

- `CosmosWorker`, `CosmosStage` in `.elai_cc/crates/knowledge/src/cosmos_worker.rs`; `compile_cosmos_briefing` in `.elai_cc/crates/orchestrator/src/briefing.rs` (per Q-10 ground truth); `grep -icE "receipt|citation|row_id"` in `cosmos_worker.rs` = 0 (briefing lines receipt-unbound today).
- E-48 `VerifiedCitation` / `VerificationOutcome` in `.elai_cc/crates/orchestrator/src/research/web_orchestrator/citation_verifier.rs` — the citation spine to REUSE.
- H-52 deterministic finding-anchoring / anchor resolver — the resolve-or-drop precedent to REUSE.
- The chat answer path is receipt-bound while the briefing path is not (asymmetry this unit removes).

## REVISED scope (W-10 tiered)

- SHIP NOW (Tier-1): the compile-time receipt binding on `compile_cosmos_briefing` + the resolve-or-drop rule + the `dropped_receipt_lines` count + the `ELAI_COSMOS_RECEIPT_BIND_DISABLE` kill-switch. In-process; no network beyond what the briefing already fetches.
- DEFER (Tier-2): any UI surfacing of the drop count (no dashboard surface in scope).

## Integration

- `crates/orchestrator/src/briefing.rs` (or `crates/knowledge/src/cosmos_worker.rs` per RECON-1): the receipt binding at `compile_cosmos_briefing` + the drop-and-count.
- Reuse `citation_verifier.rs::VerifiedCitation` (E-48) + the H-52 anchor resolver for resolution; no new citation type.
- `crates/orchestrator/src/ssb.rs`: the `dropped_receipt_lines` count as a FIELD on the existing briefing/Cosmos row, or at most +1 discriminated action_type if no compatible row exists (RECON-3, W-9).
- frontend_pair: N/A with justification — briefing compile-path trust binding; no dashboard surface in scope (Tier-2 deferred). Justified.

## MOVES (wargame)

M1 bind each compiled briefing line to a receipt at `compile_cosmos_briefing` and drive a real Cosmos briefing compile.
EXPECTED_OBSERVATION: every emitted briefing line carries a resolvable receipt (fetched-byte-hash or journal-row-id); a line whose source resolves is kept; the briefing compiles with a `dropped_receipt_lines` tally present (0 on a clean input).
ON_FAIL: some emitted line carries no receipt (an assembly path bypassed the binding) -> the binding missed a line source; route ALL line construction through the receipt binder before emit.
M2 inject a briefing input line whose source cannot resolve (dangling hash / missing row) and compile.
EXPECTED_OBSERVATION: the unresolvable line is DROPPED (absent from the briefing) and `dropped_receipt_lines` increments; nothing unresolvable is emitted.
ON_FAIL: the unresolvable line still emits (with or without a warning) -> the drop is advisory not enforced; make it fail-closed (no emit when the receipt does not resolve).
M3 flip the `ELAI_COSMOS_RECEIPT_BIND_DISABLE` kill-switch and re-compile.
EXPECTED_OBSERVATION: with the kill-switch set, the briefing reverts to the prior unbound behavior (rollback path exists); default (unset) keeps binding ON.
ON_FAIL: the kill-switch does nothing or the feature is instead a default-OFF enable -> convert to a W-2 default-ON kill-switch; a default-OFF enable here would need an allowlist row and is the wrong shape.

## FORK

FORK: if a briefing line legitimately has NO single source (a synthesized cross-source summary line), take route B — bind it to the SET of contributing journal-row-ids (a multi-receipt line), all of which must resolve, rather than dropping a valid synthesis; record the multi-receipt rule in the decision doc. Do NOT let a synthesis line escape binding entirely.

## RECON_NEEDED

RECON_NEEDED-1: re-grep `cosmos_worker.rs` + `briefing.rs::compile_cosmos_briefing` for a FUNCTIONAL-equivalent receipt/provenance binding already present (the engine premise grep proved the NAMED tokens absent, not that no equivalent exists — inspect via `mcp codegraph-elai callers compile_cosmos_briefing` + `.claude/tools/elai.map`); a live equivalent = premise-false kill, collapse C11 to the drop-count wiring only.
RECON_NEEDED-2: re-read `citation_verifier.rs` (E-48) + the H-52 anchor resolver to pick the THINNEST adapter from an existing receipt into a briefing-line binding; no second citation type family (GD-1).
RECON_NEEDED-3: re-read the live `SCHEMA_VERSION` + briefing/Cosmos row sites in `ssb.rs` + `action_type_lock.rs` at edit to place `dropped_receipt_lines` as a field vs +1 (W-9).

## ABORT

ABORT: any proposal that (a) emits an unresolvable line under any flag, (b) binds the receipt to the briefing OUTPUT rather than the SOURCE (self-certifying), (c) adds a second citation type family instead of reusing E-48/H-52, or (d) uses an LLM judge to decide receipt validity -> STOP and flag. Each breaks the trust property the unit exists to add.

## WIRING_CONTRACT
produces_symbols:
  - briefing.rs / cosmos_worker.rs — compile-time receipt binding on compile_cosmos_briefing + resolve-or-drop + the dropped_receipt_lines count
  - the ELAI_COSMOS_RECEIPT_BIND_DISABLE kill-switch (default-ON)
consumes_symbols:
  - citation_verifier.rs::VerifiedCitation / VerificationOutcome (E-48) — reuse, do NOT re-implement (GD-1)
  - the H-52 anchor resolver — reuse for line-source resolution
  - briefing.rs::compile_cosmos_briefing / cosmos_worker.rs::CosmosWorker/CosmosStage (extend the compile path)
reachable_from:
  - the Cosmos briefing compile entrypoint (`compile_cosmos_briefing` on the O-1/Cosmos daemon path) [PRODUCTION briefing compile]
  - a #[cfg(test)] caller NEVER satisfies reachability (R7)
stub_ban:
  - NO emission of an unresolvable line under any flag (fail-closed drop)
  - NO receipt bound to the briefing OUTPUT (must bind the SOURCE)
  - NO second citation type family — reuse E-48/H-52
  - NO LLM judge for receipt validity — deterministic hash/row resolution
  - NO default-OFF `ELAI_*_ENABLE` — default-ON `ELAI_COSMOS_RECEIPT_BIND_DISABLE` kill-switch (W-2); R9 N/A
  - NO blake2/sha2 — blake3 for the fetched-byte-hash receipt
frontend_pair:
  N/A with justification — briefing compile-path trust binding; no dashboard surface (Tier-2 deferred). Justified.
observable_effect:
  trigger: "compile a real Cosmos briefing on the O-1/Cosmos path with a mix of resolvable and one deliberately-dangling source line"
  expect_ssb: "the briefing row carries `dropped_receipt_lines`; the count increments by exactly the number of unresolvable input lines and recomputes from the source resolution log (GD-2)"
  expect_artifact: "a compiled briefing where every emitted line has a resolvable receipt + a dropped_receipt_lines tally. NEGATIVE: an emitted line with no receipt, an unresolvable line that survives, a receipt bound to the output, or a second citation type must FAIL"

PRODUCTION_HOOK: producer the receipt binder -> consumer the `compile_cosmos_briefing` line-emit path (fires on the Cosmos daemon briefing compile), wired same changeset; producer the drop-count -> consumer the briefing SSB row, wired same changeset. No producer-only fn and no test caller satisfies the hook.

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-13_receipt_bound_briefing.md --run-trigger
rg -c "receipt|row_id" .elai_cc/crates/orchestrator/src/briefing.rs   # MUST be >0 (or cosmos_worker.rs per RECON-1)
rg -c "dropped_receipt_lines" .elai_cc/crates/orchestrator/src        # MUST be >0
test -f workflow/reasoning/Q-13-decisions.md
```

## VERIFY

VERIFY: before declaring Q-13 done, the auditor runs off-agent —
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-13_receipt_bound_briefing.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`
plus the M2 dangling-source compile showing the unresolvable line DROPPED and `dropped_receipt_lines` incremented. Pass = wire-checker clean (or a WIRE-DARK[Q-13] row), the drop count journal-recomputable (GD-2), and the fail-closed drop proven on a real briefing. Cargo-green alone is necessary-not-sufficient.

INTEGRATION_SMOKE: an INDEPENDENT agent compiles a HARD real Cosmos briefing (many source lines, at least one dangling and one synthesized cross-source line) end-to-end on the daemon path: every emitted line resolves a receipt, the dangling line is dropped and counted, the synthesized line binds a resolving multi-receipt set (FORK route B), and flipping the kill-switch reproduces the prior unbound behavior. A single-line toy briefing does not satisfy R10.

## REDTEAM

REDTEAM: attack that FAILED — "emit a confident briefing line with no source and let it read as grounded": FAILS because the line has no resolvable receipt, so it is dropped and counted, never emitted. Attack that FAILED — "bind the receipt to the briefing's own output so any line self-certifies": FAILS because the binding is to the SOURCE (fetched bytes / journal row), and an output-bound receipt is an explicit stub_ban + ABORT. Attack that FAILED — "smuggle an unresolvable line through by disabling the feature": FAILS because it is a default-ON kill-switch whose OFF path reverts to prior behavior but still cannot emit a line the resolver rejects when ON, and the default is ON; a default-OFF enable would need an allowlist row and is refused. Attack that FAILED — "add a shiny second citation type that quietly accepts anything": FAILS because RECON-2 + stub_ban force reuse of E-48/H-52, no parallel type family.

## Grounding

- ELAI-precedent composition: E-48 `VerifiedCitation`/`VerificationOutcome` (`citation_verifier.rs`), H-52 anchor resolver, `compile_cosmos_briefing` (`briefing.rs`), `cosmos_worker.rs` (CosmosWorker/CosmosStage). Re-greppable in-tree (W-7). No external effect claim — this is a trust binding, not a measured benefit; the dossier §31 output-guard is adjacent design context only, not a cited number.
- CLAUDE.md hard rules: blake3 for the fetched-byte-hash receipt (never blake2/sha2); no .unwrap/.expect on prod; scoped tests; GD-2 journal-recomputable drop count; W-2 kill-switch not enable-flag.

## COMPOSES-WITH

E-48 verified citation, H-52 anchor resolver, B-42/O-1 Cosmos artifacts + `compile_cosmos_briefing`, the chat-path receipt spine (the symmetry this unit restores). Re-implements NONE (GD-1). The only new object is the briefing-line receipt binding + its drop count.

## Open questions / risks

- DUP-RISK: if RECON-1 finds an equivalent binding already present, C11 collapses to the drop-count only (premise-false kill) — decided before building, not after.
- Synthesized cross-source lines need the multi-receipt rule (FORK route B) or they would be over-dropped; the decision doc records the rule.

## Next

Removes the chat-vs-briefing receipt asymmetry: a fabricated Cosmos briefing line becomes unrepresentable. Trust sibling to Q-11 (instrument integrity) and Q-12 (research-lane economics). No spine number; grounding-quality only.

End of Q-13.
