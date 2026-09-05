phase-I41-decisions.md
session: s-107b022a  task: A113-I41-EXECUTE  date: 2026-06-13

WHAT
  Build the I-41 spec-time critique lenses module (L1-L8), wire into idea_framing.rs run_critic_loop
  (advisory only), add elai critique CLI command, add CritiqueLensFinding SSB variant, and
  contract-test the advisory-only isolation invariant.

LIVE BASELINE (re-read at edit time per W-9)
  ssb.rs SCHEMA_VERSION = 242 (I-35 ToolSequenceVerdict is last)
  action_type_lock.rs TOTAL_VARIANTS = 551
  I-41 target: SCHEMA_VERSION 243, TOTAL_VARIANTS 552 (+1 action_type, per controller correction)

CONTROLLER CORRECTION APPLIED
  Context-package/plan said 3 arms = +3 TV (549+3=552, 551+3=554 target).
  Controller says: Pattern-2 ONE action_type +1 TV. That means CritiqueLensFinding is ONE
  SsbActionType variant (carrying a CritiqueLensFindingPayload that itself has a Kind enum with
  3 arms). TV counts ACTION TYPE VARIANTS not payload arms. Verified by inspecting:
    - GovernancePrimitiveFired: 1 entry in ALL_VARIANTS (CeflExternalized representative) -> +1 TV
    - RoleContractApplied:      1 entry in ALL_VARIANTS (PersonaStripped representative)  -> +1 TV
    - ToolSequenceVerdict:      1 entry in ALL_VARIANTS (Allowed representative)           -> +1 TV
  Each of these is ONE SsbActionType variant wrapping a 3-arm Kind enum.
  THEREFORE: CritiqueLensFinding = +1 TV. Target = 551 + 1 = 552, SCHEMA 242 + 1 = 243.

KEY DECISIONS

  D-1 PATTERN-2 SINGLE-ACTION-TYPE:
    CritiqueLensFinding is ONE SsbActionType variant carrying CritiqueLensFindingPayload.
    The payload has a 3-arm inner Kind enum (AdvisoryFinding / LensRun / DroppedNoAnchor).
    ALL_VARIANTS gets ONE representative entry (AdvisoryFinding representative).
    TV bumps 551 -> 552. SCHEMA bumps 242 -> 243.
    Rationale: consistent with every other I-plan variant pattern observed in code.

  D-2 ADVISORY ISOLATION VIA DISTINCT TYPES:
    LensSeverity { Note, Concern, Flag } is distinct from idea_framing::Severity.
    No Into<Severity>, no Into<IdeaFramingError>, no imports of CriticFinding or Severity.
    critique/ module only imports ssb types + providers::runtime types.
    Compiler-enforced: impossible to accidentally route a LensFinding onto the gate path.

  D-3 RUNTIME PARAMETER THREADING (GAP-1 RESOLUTION):
    add runtime: Option<&dyn RuntimeAdapter> to run_critic_loop signature.
    Existing callers (frame_idea) pass None -> LensSet::run is skipped when runtime is None.
    This is additive and backwards-compatible (None = advisory-absent, never a Block).
    The ELAI_CRITIQUE_LENSES_ENABLE env flag is checked inside LensSet::run; kill-switch row
    in wire-dark-allowlist.toml registers it.

  D-4 SPEC TEXT SOURCE (GAP-2 RESOLUTION):
    framed.raw_prompt is the field (confirmed at idea_framing.rs:963 usage).
    Pass framed.raw_prompt.clone() to LensSet::run. No new field on FramedIdea.

  D-5 LENS PROMPTS AS include_str! FILES:
    crates/orchestrator/src/critique/prompts/ subdirectory, 8 .md files (L1.md .. L8.md).
    Loaded via include_str!("prompts/L1.md") inside lenses.rs.
    No gate-verdict logic in prompt files (ban-contract compliant).

  D-6 LENSSET::RUN ERROR HANDLING:
    LensSet::run returns Result<Vec<LensFinding>, LensCritiqueError>.
    In run_critic_loop, the error is logged (warn!) and swallowed: advisory absence != block.
    The function return type of run_critic_loop stays Result<u32, IdeaFramingError>.

  D-7 CONTRACT TEST MOCK:
    InlineMockRuntime implements RuntimeAdapter locally in the test file.
    Returns canned CompletionResponse from a VecDeque queue.
    Zero live network calls. This satisfies GAP-3.

EXPECTED TEST OUTCOME
  cargo check --workspace --tests: exit 0 (no compile errors)
  cargo test -p elai-contracts -- i41: all arms PASS
  cargo test -p elai-orchestrator -- action_type_lock: 2/2 PASS (243/552 constants)

A117 AUDIT FIX (task A117-I41-FIX, 2026-06-13)

  BLOCK B-1 RESOLUTION (F-1): R9 path taken.
    frame_idea has no RuntimeAdapter parameter; threading one would require a
    signature change across the framing stack (frame_idea, frame_idea_with_fsm,
    and every IPC/CLI caller). The recommended bounded fix (3 toml lines) was
    taken instead.
    Added to wire-dark-allowlist.toml: [[flag]] name="WIRE-DARK[i41-framing-surface]"
    type=experiment, owner=maintainer, expiry_date=2026-12-01, promotion_plan_id=I-42.
    Added doc note in critique/mod.rs naming the dead surface and promotion target.
    The CLI path (elai critique) and i41_a/c/d tests remain the live surface.

  F-2 PIN DOCTRINE (13 exact pins converted to >= floors across 9 files):
    i8 (2 pins): body.contains("...=550") -> parse-then->= 549
    i10 (1 pin): body.contains("...=550") -> parse-then->= 546
    i38 (1 pin): body.contains("...=550") -> parse-then->= 544
    i43 (1 pin): body.contains("...=550") -> parse-then->= 545
    h34 (1 pin): body.contains("...=524") -> parse-then->= 524
    h36 (1 pin): body.contains("...=526") -> parse-then->= 526
    h37 (1 pin): body.contains("...=527") -> parse-then->= 527
    h38 (1 pin): body.contains("...=528") -> parse-then->= 528
    h49 (1 pin): assert_eq!(SCHEMA_VERSION_PUBLIC, 208) -> assert!(>= 208)
    e45: left as exact lock per controller instruction.

  VERIFIED TEST RESULTS:
    i41: 5/5 PASS
    i8:  9/9 PASS
    i10: 23/23 PASS (includes total_variants_is_546)
    i38: 6/6 PASS
    i43: 11/11 PASS
    e45: 14/14 PASS (exact lock preserved at 552)
    action_type_lock: 2/2 PASS
    i1: 7/7 PASS
    i32: 7/7 PASS
    i33: 5/5 PASS
    i35: 5/5 PASS
    h39: 11/11 PASS
    h34: 7/7 PASS
    h36: 4/4 PASS
    h37: 9/9 PASS
    h38: 4/4 PASS
    h49: 23/23 PASS (schema_version_is_208 PASS)
    cargo check --workspace --tests: exit 0 (Finished, warnings only)
  cargo test -p elai-contracts -- e45: PASS (unaffected)
  cargo test -p elai-contracts -- i48: PASS (prompts ban contract unaffected)

UNKNOWNS / PENDING
  Live LLM lens runs: PENDING (fixture/mock mode only this session per plan DEFER list).
  I-48 prompts ban contract: need to verify exact rules before writing prompt .md files.
  wire-dark-allowlist.toml: ELAI_CRITIQUE_LENSES_ENABLE row to be added.
