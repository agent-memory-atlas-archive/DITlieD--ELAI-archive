Phase: B-16B Chunk C (LazyLock<Regex> hoist)
Session: s-1327164
Date: 2026-04-29

What
  Hoist 1 inline literal-regex .expect() to module-level LazyLock<Regex>.
  HANDOFF estimate of "12 production hits" was wrong — context-compiler verified ALL
  HANDOFF-named sites in deterministic_auditor.rs (3) already use fn-scoped LazyLock
  (CLAUDE.md static-init exception applies). dkg/mod.rs and output_postprocessor.rs had
  zero matching sites.

Sites (1)
  .elai_cc/crates/reasoning/src/engine.rs:501-502
  Pattern: `let claim_re = Regex::new(r"...").expect("claim regex is valid")`
  Fn: `fn ground_plan(&self, plan: &str) -> PlanGroundingResult` — non-Result fn.
  Classification: X — module-level LazyLock hoist.

Key decision
  Use `static CLAIM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"...").expect("..."))`
  at the top of engine.rs (after imports). Reference as `CLAIM_RE.clone()` or `&*CLAIM_RE`
  inside ground_plan. The static-init .expect() is allowed by CLAUDE.md (panic at first
  use = process startup-equivalent failure).

Expected test outcome
  cargo check --workspace: PASS, 0 new warnings.
  Behavior change: regex compiled once on first call, reused thereafter. Net perf-positive.

Unknowns
  - Whether `Regex` is `Clone` cheaply (yes — `regex::Regex` is `Clone` and internally Arc).
    Use `&CLAIM_RE` if borrow is sufficient; clone only if owned needed.

Out of scope
  - deterministic_auditor.rs:946, 992, 1037 (already LazyLock)
  - dkg/mod.rs (no matching sites)
  - output_postprocessor.rs (no matching sites)
  - Any sig-change-requiring sites (none found)
