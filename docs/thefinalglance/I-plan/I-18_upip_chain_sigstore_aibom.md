# I-18: upip chain + sigstore + ai-bom

- Track 15 — DETERMINISTIC GATES + ANTI-GAMING VERIFICATION | lever AUDITABLE | grounding STRONG | nature EXTENSION | effort M | WAVE 3 | IC-11
- Premise: G-2 RRA and G-8 attestation.rs are SHIPPED (IS-10: rra.rs build_rra:384 + verify_rra; attestation.rs
  with in-toto/sigstore mentions). what is absent is tamper-evidence across the row sequence: a compromised row can be
  edited without detection because the SSB has no per-row chain. i-18 extends the shipped surface with a prev-hash
  chain on the single-writer, a 5-layer stack hash sealed into each RRA, opt-in external transparency (Rekor), and an
  AI-BOM/CycloneDX-ML entry so every artifact speaks OWASP AISVS AC.8. IC-11 is the binding correction: this is
  EXTENSION, restating nothing already in G-2/G-8.

## Steal/build
PER-ROW CHAIN (IN-5 confirmed absent): every SSB insert adds a chain_prev_hash field = blake3(prior_row_bytes); the
single-writer holds the rolling hash so no out-of-order insert can spoof the chain. Coordinated with the golden lock
(ssb.rs append-only discipline IS-1 + action_type_lock.rs) — the chain field is an extend, not a schema change.
5-LAYER STACK HASH sealed into the RRA alongside the existing build_rra:384 fields: STATE (SSB snapshot hash) /
DEPS (Cargo.lock blake3) / PROCESS (process execution environment) / RESULT (output artifact hash) / VERIFY
(verifier-stack outcome hash). all five layers use blake3. a Cargo.lock mutation mid-run invalidates the DEPS layer
immediately (halt).
OPT-IN REKOR PUSH: the cosign keyless signing flow pushes the RRA + stack hash to a Rekor transparency log. this is
a NEW DEP (cosign binary or the cosign-rs crate — anti-slopsquat: verify package name + repo + last-publish + license
before any import; user approval required before the dep lands). the offline blake3-Merkle default ships without it.
AI-BOM/CYCLONEDX-ML: emit a CycloneDX-ML component entry alongside every sealed attestation; validates against the
published CycloneDX schema (WebSearch-confirm schema URL at execution). cites OWASP AISVS AC.8.
EXCLUDED: any design that edits a sealed row ("editing a sealed row" is the primary stub_ban); sha256 or blake2
for the chain hash (blake3 only, anti-stale-training hard rule); a batch-path or async write that loses the prev_hash
ordering guarantee.

## Existing ground truth
- IS-10: rra.rs build_rra:384 (+ verify_rra, H-35 cite); attestation.rs in-toto/sigstore surface — SHIPPED; i-18 EXTENDS both, restates neither (IC-11, GD-1)
- IS-1: ssb.rs golden-locked, append-only discipline (:5) — the chain field appends to the existing row schema without breaking the lock
- IS-19: wiring_contract_check.py + gate_freeze_check.py — the frozen gate tooling i-18 must not modify (R8)
- IN-5: NO per-row chain_prev_hash in SSB rows — CONFIRMED absent this session; i-18 is the BUILD of that field
- F-4 stdout_hash (a chained field): mentioned in i-18 COMPOSES-WITH per the summary; executor re-greps the stdout_hash surface at EXECUTING (W-7, PLAN-asserted file:line)
- "PLAN-asserted; executor re-greps per W-7": cosign dep exact crate name + version; CycloneDX schema URL; Rekor endpoint default

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- chain_prev_hash field on every SSB row (single-writer rolling blake3 hash, deterministic in-process)
- 5-layer stack hash (STATE/DEPS/PROCESS/RESULT/VERIFY, all blake3) sealed into the RRA by build_rra
- Cargo.lock mutation mid-run halt (DEPS layer invalidation triggers a hard abort with RowChainSealed(ChainBroken))
- AI-BOM/CycloneDX-ML emission alongside the attestation
- `elai verify-attestation --chain` flag that walks the chain and reports the first broken link with its seq number
- RowChainSealed SSB action_type (Kind: ChainAdvanced / RekorPushed / AibomEmitted / ChainBroken)
DEFER (Tier-2 / gated):
- opt-in Rekor transparency push: cosign dep is a new external dependency (anti-slopsquat verification + user
  approval required before landing); the `--rekor` flag scaffolds the interface now, the push is a separately-gated
  checkbox (W-10). the offline default is complete without it.

## Integration
modify: crates/orchestrator/src/ssb.rs (chain_prev_hash field on insert; rolling hash in the single-writer; golden
lock coordination — executor checks ALL 6 lockstep bump sites per wire-remediation discipline).
modify: crates/orchestrator/src/rra.rs (extend build_rra with 5-layer stack hash, new verify leg, --chain flag).
modify: crates/orchestrator/src/attestation.rs (AI-BOM emit alongside existing attestation).
new: `elai verify-attestation` chain subcommand in crates/cli/src/.
SSB action_type: +RowChainSealed (Kind: ChainAdvanced / RekorPushed / AibomEmitted / ChainBroken) — baseline+1.
frontend_pair: chain badge on the existing RRA viewer (composing the G-2/G-8 surface — not a new drawer). the badge
shows CHAIN OK / BROKEN + seq of break. cross_lang_diff PASS required (W-4).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/ssb.rs::chain_prev_hash field on every SsbRow insert
  - crates/orchestrator/src/rra.rs::StackHash (5-layer struct + seal/verify methods)
  - AI-BOM CycloneDX-ML file emitted alongside every sealed RRA
  - RowChainSealed SSB action_type with ChainAdvanced / RekorPushed / AibomEmitted / ChainBroken discriminators
consumes_symbols:
  - crates/orchestrator/src/rra.rs::build_rra (G-2 — why: extends the sealing function; GD-1 do NOT re-implement)
  - crates/orchestrator/src/attestation.rs (G-8 — why: AI-BOM appended alongside; GD-1)
  - crates/orchestrator/src/ssb.rs single-writer append path (IS-1 — why: chain_prev_hash is written here; GD-1)
  - F-4 stdout_hash surface (why: a chained field; compose, do not redefine; PLAN-asserted, re-grep at EXECUTING)
reachable_from:
  - `elai run --emit-rra` (CLI entrypoint) which calls build_rra + the chain writer on every SSB flush  [PRODUCTION CLI run]
  - `elai verify-attestation --chain` (CLI entrypoint) which reads the chain and walks it  [PRODUCTION verify command]
  (contracts/ callers do not satisfy R7 — the production CLI paths are the reachable_from)
stub_ban:
  - NO editing a sealed row: once RowChainSealed(ChainAdvanced) is emitted for a seq, that row's bytes are immutable
  - NO sha256 or blake2 anywhere in the chain, stack hash, or AI-BOM integrity fields (blake3 only — anti-stale-training)
  - NO async or batch SSB path that can disorder the rolling prev_hash (the chain is only valid in single-writer order)
  - NO silent Rekor failure: if --rekor is passed and the push fails, the run halts with a clear error, not a SKIP
  - NO .unwrap()/.expect() on production paths; NO fabricated chain verification results
frontend_pair:
  chain badge on the existing RRA viewer drawer. Tauri IPC: RowChainSealed rows are read by the RRA viewer's existing
  SSB subscription. badge shows CHAIN OK (all ChainAdvanced since session start) or BROKEN (ChainBroken seq). new
  field on the existing row struct, not a new drawer. cross_lang_diff PASS required (W-4). Justified.
observable_effect:
  trigger: "elai run --emit-rra --task-fixture fixtures/basic_task.toml && elai verify-attestation --chain"
  expect_ssb: "RowChainSealed(ChainAdvanced) rows for every SSB insert during the run. RowChainSealed(AibomEmitted) when the RRA seals. on Cargo.lock mutation mid-run fixture: RowChainSealed(ChainBroken) at the DEPS layer with the exact seq named."
  expect_artifact: "the sealed RRA bundle passes verify-attestation --chain with exit 0. the AI-BOM file validates against CycloneDX schema (schema URL WebSearch-verified at execution). editing any mid-run SSB row in the db (sqlite UPDATE on seq N) causes verify-attestation to fail at seq N naming the break — FAILS (R2, the negative arm). the Rekor arm (--rekor, opt-in checkbox) produces an inclusion proof verifiable without re-running ELAI."

## Grounding
- IS-10 (rra.rs:384 + attestation.rs): the SHIPPED surface i-18 extends; IC-11 is the binding correction (EXTENSION not BUILD)
- IN-5: per-row chain_prev_hash absent — CONFIRMED; i-18 is the BUILD
- IS-1: ssb.rs append-only discipline — the golden lock coordination site; ALL 6 lockstep sites per wire-remediation
- OWASP AISVS AC.8 (trust-review B7 reference): the AI-BOM compliance target — WebSearch-confirm the current schema revision at execution
- Sigstore/Rekor transparency log: WebSearch-confirm cosign-rs crate name + version + repo before dep addition (anti-slopsquat mandatory)
- CycloneDX-ML schema: WebSearch-confirm URL + version at execution
- Consolidated Part IV + review 1.1 "per-row chaining + Rekor opt-in + AI-BOM"
- SSB: +RowChainSealed (Kind: ChainAdvanced/RekorPushed/AibomEmitted/ChainBroken) — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
G-2 rra.rs build_rra (EXTENDS seal — do NOT re-implement the RRA format or build logic); G-8 attestation.rs (AI-BOM
appended alongside — do NOT re-implement attestation write path); IS-1 ssb.rs single-writer (chain field is an
extend — do NOT fork the writer); F-4 stdout_hash (a chained field — do NOT redefine). the chain, stack hash, and
AI-BOM are the NET-NEW outputs; everything else is composition.

FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule) — effort M.

End of I-18.
