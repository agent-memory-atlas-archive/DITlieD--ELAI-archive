Phase 2A Grounding Results
Date: 2026-04-06
Session: s-1003192

SPEC VERIFICATION
  docs/specs/crates/reasoning.txt sections 1-3: PRESENT, non-empty
  docs/specs/crates/reasoning.txt section 10 (KEY TYPES): PRESENT, non-empty (added to READ directives)
  docs/specs/crates/reasoning.txt section 11 (TESTABLE INVARIANTS): PRESENT, non-empty (added to READ directives)
  docs/specs/architecture-overview.txt section 4 (GrcEngine): PRESENT, trait with think/set_budget/verify_claim
  docs/specs/integration/prompt-architecture.txt section 1: PRESENT, non-empty

CLAIM GROUNDING
  "GRC state machine: observe->hypothesize->plan_action->act->verify" | GROUNDED | reasoning.txt section 2
  "Budget per-topic: max_calls 1-12, max_chars 200-1500" | GROUNDED | reasoning.txt section 3
  "Grounding required on every step" | GROUNDED | reasoning.txt section 2 + section 11 item 1
  "Budget overflow forces convergence" | GROUNDED | reasoning.txt section 3 + section 11 item 2
  "Status step is free echo" | VERIFIED | CLAUDE.md GRC tools section: "mcp_think(step_type='status', content) for free trailing echoes"
  "<5ms validation overhead" | UNGROUNDED | assumed reasonable for HashMap + string parsing, will verify empirically
  "Memory per topic <1KB base" | UNGROUNDED | assumed from struct sizes, will verify empirically
  "thiserror for GrcError" | VERIFIED | crates.io standard, 290M+ downloads

SUBSTANTIVE IMPROVEMENTS ADDED (with evidence)
  1. State transition validation via allowed_transitions map
     Evidence: standard FSM practice. guidance-ai (github.com/guidance-ai/guidance) uses FSM-from-schema for token masking. Our version validates at step level.
  2. GrcError typed enum with 4 variants
     Evidence: standard Rust error handling. thiserror derive macro is the ecosystem standard.
  3. Budget forced convergence at remaining_calls<=1
     Evidence: BudgetThinker (OpenReview 2025, openreview.net/forum?id=ahatk5qrmB) injects budget-remaining signals. Token-Budget-Aware LLM Reasoning (ACL 2025, aclanthology.org/2025.findings-acl.1274/) adjusts by complexity.
  4. Grounding claim decomposition (format validation)
     Evidence: RARR (arxiv:2210.08726) decomposes output into claims for per-claim verification. SAFE (arxiv:2403.18802) does similar. Phase 2A validates format; phase 2B verifies sources.
  5. GroundingLevel enum (VERIFIED/HIGH/UNCERTAIN)
     Evidence: reasoning.txt section 4 defines these three levels explicitly.
  6. Hard/soft grounding distinction
     Evidence: DSPy Assertions (arxiv:2312.13382) distinguish Assert (hard, backtrack+retry) vs Suggest (soft, log+continue). Improved quiz validity from 37.6% to 98.8%.
  7. Expanded READ directives to include sections 10 (types) and 11 (invariants)
     Evidence: reasoning.txt sections 10-11 define the exact types and testable invariants phase-2A must satisfy.

ADVERSARIAL AUDIT RESULTS (auditor session, 5 BLOCKs resolved)
  BLOCK 1: T2 budget test used max_calls=2, conflicts with must_converge at remaining<=1.
    FIX: changed to max_calls=3, added explicit must_converge assertion + Converge step.
  BLOCK 2: GrcEngine trait had 4 methods, but verify_claim/ground_plan needed for phase 2B.
    FIX: expanded to 6-method trait with unimplemented!() stubs for 2B methods.
  BLOCK 3: GrcState missing Clone derive, needed for InvalidTransition::allowed Vec.
    FIX: added #[derive(Debug, Clone, Copy, PartialEq, Eq)] to all type specs.
  BLOCK 4: ThinkResult not defined in step 2A.1, but referenced by GrcEngine::think().
    FIX: added ThinkResult to 2A.1 TASK type list and DETERMINISTIC_CHECK grep.
  BLOCK 5: search/tool grounding types had no GroundingLevel mapping.
    FIX: search->Verified (per reasoning.txt s4), tool->High.
  WARN fixes applied: check-then-increment pattern (I3), TopicState consistency (grounding_log
    in 2A.1), default budget (max_calls=5, max_chars=800), PRE_GATE exit code, scoped fn count,
    T8 (default budget test), T9 (first-step-must-be-Observe test), criterion dep clarification,
    entropy quarantine note for benchmarks.

SPEC GAPS: none found
SPEC CONFLICTS: none found (architecture-overview.txt updated to match 6-method trait)
