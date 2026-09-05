# J-55 decision record (model-strength modality router)

Status: UNIT W1 substrate landed across executor slices X1 (W1.4 lockstep + profile field), X2
(W1.1/W1.2 core+wiring), fix rounds 1-2 (runtime_client compile repairs); X3 (this unit) landed the
W1.5 contract suite `.elai_cc/crates/contracts/tests/j55_modality_router_contract.rs`. D1 remains
DEFERRED-gated. This file is outside git scoring (`workflow/reasoning`, gitignored) like docs/.

## FORKS closures

1. **J-51 sequencing** — CLOSED plan-side (J-55 FORKS item 1): `tool_call_adherence` verified absent
   from code, so UNIT W1 resolves CodeAct on params+tier alone via directional NAMED CONSTANTS;
   adherence folds in additively when J-51 ships.
2. **X1 [D] missing-tool-bit fork** — RESOLVED by controller ruling OPTION B: directional
   `ModelFeatures.tool_calls_native` bit added, seeded at detect time (single-source
   `seed_tool_calls_native()` policy: cloud-classified families sonnet/opus/haiku + qwen-coder
   family = true; unknown names stay false => honest TextFallback). Full question/options text
   preserved verbatim in `conductor/opus_walk/evidence/J-55/w1-x1-executor.md` section [D].
3. **Carrier shape deviation (W1.4)** — shipped fieldless `ModalityRoutedTag` on `SsbActionType`
   plus rich `ModalityRoutedKind` inside `ModalityRoutedPayload`, mirroring the I-41 two-enum split.
   Reason: AuthBrokerEvent/PromptCompiledKind precedents are FIELDLESS Copy kinds; no shipped
   precedent puts a data-carrying enum directly on `SsbActionType`, which would force lossy
   fabricated representatives into `from_str` + registry. Every planned wire field
   (model_fp/role/schema_tokens/diff) exists payload-side; deviation is shape-mechanical only.
4. **Golden derived-not-regenerated** — regen was unavailable because the plain-`cargo` depot shim
   FAIL-CLOSED (`status='setup-required'`, x1 evidence [E]); build law forbids workarounds. The
   committed golden row (`modality_code_act_chosen`) was derived deterministically from the registry
   append + LC_ALL=C sort position instead. Regen command recorded for when the depot route opens:
   `cargo test -p elai-orchestrator --test action_type_lock regen_golden_file -- --ignored --nocapture`.
   BYTE-DIFF-IF-DIFFERS RULE: once regenerated, THAT output wins over the committed derivation.

## RT-3 residual

A mis-provisioned endpoint whose TIER reads cloud while serving a 7B model would resolve CodeAct,
because `resolve`'s contractual FIRST rule is tier=Cloud => CodeAct before any parameter check (E-31:
the hosted front door serves tool calls natively; tier is trusted TD-3 input, profiles never derive
from LLM output). This stays pinned as RT-3's D1 LIVE replay fixture, deliberately excluded from the
W1 contract negative arm (which therefore asserts sub-14B => never CodeAct ON THE LOCAL TIER, all
roles, all crossover variants). Contract test `j55_modality_router_contract.rs` module header carries
the same exclusion note.

## Honest residues / wire-dark ledger

- **Site-B row emission** — resolved BY X2, correcting the earlier template assumption that it would
  defer to seam A: RuntimeClient::stream emits through its own `streaming_ssb` channel with identity
  ("cli:stream", superstep 0) (x2 evidence [B].7); site A emits through the drive_task SSB writer.
- **TextFallback emits NO ModalityRouted row** at either site: the substrate ships exactly three
  kinds; fabricating a fourth violates stub-ban honesty. Tracing-visible absence only (x2 [B].4).
  Decision carried forward: either add a TextFallback kind arm at D1 or accept silent text lanes.
- **EditFormatChosen** is emitted by the established edit-surface channel (tools/src/dispatcher.rs
  edit tiers own the universal SEARCH/REPLACE path), NOT by turn-prep emitters; turn-prep performs no
  edit operation (x2 [B].5). The contract suite pins only the payload/Tag substrate + round-trip.
- **schema_tokens** measured honestly but differently per site until the I-36 leg reaches the seam:
  site A = serialized narrowed name list / BUDGET_CHARS_PER_TOKEN surrogate; site B = real presented
  tool-definition bytes (x2 [B].6).
- **Direct-ctor seeding caveat**: `ModelCapabilityProfile::micro_7b_12gb()` as shipped leaves
  `tool_calls_native=false`; the directional TRUE exists only along the production detect path
  (single-source seeding). Direct-ctor consumers degrade honestly to TextFallback. The contract
  obtains arm-(c)'s keystone profile through `detect_model_capability_profile` (production-shaped).
- **D1 legs parked** on probed-dead endpoints http://127.0.0.1:8181 and :4000 (connection refused,
  dated 2026-08-27); live-turn proofs + crossover bench stay DEFERRED-gated behind named gates.
- **Compile authority**: no cargo feedback ran locally (depot route down); opus_land gate owns the
  GREEN verdict. Contract compile risk consciously reviewed symbol-by-symbol against source.

## Acceptance mapping (clause-by-clause)

| Clause | Where landed | Evidence |
| --- | --- | --- |
| W-1 entrypoint=two dispatch sites -> resolve | task_runner.rs drive_task turn-prep ~:6742 block; cli/runtime_client.rs stream closure | x2 STEP 4/5; auditor wire-check |
| W-2 kill-switch default-ON | ELAI_MODALITY_ROUTER_DISABLE, 1/true case-insensitive only; unset => enabled | contract_e_kill_switch_* |
| W-3 no opt-in flag | none exists (contract pins unset=>enabled) | same |
| W-4 objective measures | schema_tokens rows (real/surrogate per site); bench legs D1 | x2 [B].6 |
| W-5 production callers, test roots rejected | RT-2 census: exactly two prod call sites | this record + x1/x2 |
| W-6 badge + schema-token readout | dashboard code may land W1; visual proof gated to D1.3 | x2/D1 hooks note |
| W-7 anchors re-grepped | B-10/J-51/E-31/I-36/H-4/E-35 greps in gather-core/gather-ssb + refit | w1-gather-* |
| W-8 frozen gates | no checker/gate edits by any worker slice | git status censuses |
| W-9 baseline re-read + SCHEMA bump | 619->620 entries, SCHEMA_VERSION_PUBLIC==311 | x1 [C]; contract_f assert |
| W-10 in-process resolve now; bench behind D1 gates | modality_router.rs pure fn; bench deferred | plan UNIT D1 |
| TD-3 deterministic route, LLM no input | total/pure resolve(profile, role, tier); contract_a determinism + env-purity arm | modality_router.rs |
| Keystone guard (sub-14B never CodeAct local) | contract_c positive + contract_g NEGATIVE (all roles, crossover noise) | j55_modality_router_contract.rs |
| Thresholds: NAMED CONSTANTS, D1 re-base | CODEACT_VIABILITY_FLOOR_B directional; crossover override preferred when Some | modality_router.rs; g forged-floor arms |
| Universal edit path: diff not whole-file | E-35 matcher already wired at tools/src/dispatcher.rs try_edit_tiers; orchestrator must not dep elai-tools (dev-dep law) -> contract covers EditFormatChosen{diff} channel; matcher application itself is E-35-owned coverage | w1-gather-core §5; header notes |
| cargo check --workspace GREEN via depot shim | opus_land gate verdict only; worker ran no cargo | A5 build law |
| No new dependency | contract uses existing contracts dev-deps only (elai-orchestrator, elai-ipc-types, rmp-serde, serde_json untouched here) | Cargo.toml unmodified |

Residual honestly accepted: paper thresholds are 2024-2025-vintage directional values; the D1 bench
is the load-bearing proof. No residues from the refit beyond those logged above.
