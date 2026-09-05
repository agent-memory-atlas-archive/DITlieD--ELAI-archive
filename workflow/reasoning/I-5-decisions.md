I-5 decisions -- pristine context injector

what: PristinePathFilter::build_branch_context in crates/orchestrator/src/pristine_path_filter.rs.
Enforces the ID-1 stateless-generator doctrine: hard-excludes failed-sibling tracebacks
from rebuilt branch contexts; compresses tracebacks into <=30-token contrastive constraint
phrases; emits ContextPruned SSB rows (3-arm Pattern-2: SiblingExcluded, TracebackContrasted,
VividnessProbe). Adds VividnessProbe as the 6th arm of ssb::ProbeKind (PAYLOAD-INTERNAL,
no SCHEMA/TV bump for that arm). ContextPruned action_type bumps SCHEMA_VERSION 246->247
and TOTAL_VARIANTS 555->556.

key decisions:

D1 - WIRE-DARK[I-5] on build_branch_context.
    I-4 SearchLadder::run_best_of_n is the intended production caller; I-4 is Wave-4 and
    does not ship in this changeset. WIRE-DARK marker applied directly above the fn def;
    backing row registered in workflow/sessions/s-107b022a/stubs.md. Tier-1 coverage via
    i5_pristine_context_contract.rs. R7 note: contract test caller != production caller.

D2 - VividnessProbe is ContextPrunedKind arm, not a new top-level action_type.
    The E-19 ProbeKind::VividnessProbe arm extends ssb::ProbeKind (orchestrator-internal
    5->6 arms) as a PAYLOAD-INTERNAL discriminator, not a new SsbActionType. This avoids
    an unnecessary SCHEMA/TV bump for the probing arm. The primary SSB row is ContextPruned
    with kind=VividnessProbe; future correlation with ReasoningQualityProbe rows happens via
    session_id join in the offline analytics path.

D3 - SeededClock in tests, not a custom FixedClock.
    Initial impl used a custom FixedClock struct; it did not satisfy the full ClockProvider
    trait (4 required methods: now, elapsed_since, system_now, instant_now). Replaced with
    elai_core::clock::SeededClock::new() which already implements all required methods and
    is the canonical deterministic test clock across the codebase.

D4 - MAX_CONSTRAINT_TOKENS = 30 via word-split proxy.
    True tokenizer not available at this call site; 30-word approximation matches the
    <=30-token budget stipulated in ID-1 at ~1 word/token for English error text. Real
    tokenizer caller (I-4) should pre-truncate before calling build_branch_context.

D5 - H-50 compose: regime_policy(mode) called for window selection.
    The return value is stored as _policy (unused in the current body) to establish the
    composition seam. I-4 will use the ContextPolicy to constrain the rebuilt slot budgets.

expected test outcome: cargo test -p elai_cc_contracts --test i5_pristine_context_contract -> 3 PASS.
    cargo test -p elai-orchestrator --test action_type_lock -> PASS (TOTAL_VARIANTS == 556).
    cargo test -p elai_cc_contracts --test e45_cross_language_drift_contract -> PASS (556).

unknowns: I-4 SearchLadder architecture (Wave 4); real per-slot byte accounting from SSB store
    for excluded_bytes calculation (currently uses action_type tag length as proxy).
