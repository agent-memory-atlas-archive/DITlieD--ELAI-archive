D-5 BAI Thompson Promotion — closure decisions

Scope: F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-5_bai_thompson_promotion.md
Session: s-25120 (closure pass)
Prior re-check report (PRIMARY):
  workflow/sessions/s-25120/cross-check-recheck/D-5_bai_thompson_promotion-report.md

NOTE: an unrelated 2026-05-05 file lives at workflow/reasoning/D5-decisions.md
(no hyphen) covering a live-sidecar grounding contract test. The two files
have nothing in common — D-5 here is the hyphenated form per the project
convention used by D-1-decisions.md, D-2-decisions.md, D-3-decisions.md,
and D-4-decisions.md.

D-5 substrate was ALREADY shipped before this closure pass: bai.rs at
~550 LOC with 13 tests, ArmPosterior(alpha, beta) f64, sample_beta,
Thompson::sample, promote_candidate, emit_bai_round. The closure was
NOT a re-implementation — it added the 6 gaps the prior re-check flagged.

==============================================================================
DECISION-1: SCHEMA bump 109 -> 110 (not the plan's 50 -> 51)
==============================================================================

WHAT
SCHEMA_VERSION in crates/orchestrator/src/ssb.rs went from 109 to 110.

WHY
The plan markdown (drafted pre-D-2 / pre-D-3 / pre-D-4) cites SCHEMA
50 -> 51. The actual current value at audit time was 109. The plan
file is stale on the version number, NOT on the intent. Bumping to 51
would clobber every SCHEMA bump between D-2 and the GitHub-PR landing
(C-40 was the most recent, 108 -> 109). The closure honours the real
current value and bumps atomically to 110.

GROUNDING
- Read: crates/orchestrator/src/ssb.rs:1129 const SCHEMA_VERSION: i64 = 109
- Bumped to: const SCHEMA_VERSION: i64 = 110
- Cascade: documented in the SCHEMA_VERSION header comment at lines
  1115-1144 of ssb.rs (post-edit).

==============================================================================
DECISION-2: BaiArmUpdated was already shipped at SCHEMA 72; ThompsonSampled
            is the ONLY new variant added by this closure
==============================================================================

WHAT
The closure added exactly one new SsbActionType variant: ThompsonSampled
(payload struct ThompsonSampledPayload with `arm_id`, `sampled_value`,
`alpha`, `beta`). BaiArmUpdated already existed at SCHEMA 72; no rename,
no re-add.

WHY
The plan markdown's variant table lists `ArmPosteriorUpdated` and
`bai_round_emitted` as if they were new. Both are RENAMES of the
already-shipped BaiArmUpdated. Renaming a wire tag breaks every
historical SSB row + every dashboard query that filters by
action_type. The closure leaves the shipped variant alone and adds
exactly the one variant the wire actually NEEDS — the Thompson sample
value the policy uses to pick an arm (not previously captured).

GROUNDING
- Read: ssb.rs:2874 (Self::BaiArmUpdated existing variant, SCHEMA 72)
- Read: ssb.rs:10341 (BaiArmUpdatedPayload existing struct)
- Read: docs/thefinalglance/D-plan/D-5_bai_thompson_promotion.md
  (plan markdown — flagged ArmPosteriorUpdated as drift)
- Audibert COLT 2010 paper interface
  (workflow/papers/audibert-colt-2010-bai-interface.md) makes the
  empirical-leader (`X_hat_{i, n_k}`) and the per-pull sample
  (`B_{i, s}` for UCB-E) separately observable — capturing both is
  consistent with the paper's surface.

==============================================================================
DECISION-3: pass_count / fail_count u32 fields in the plan are DRIFT.
            Kept the shipped alpha / beta f64 storage. Added accessor methods
            pass_count_estimate() + fail_count_estimate() returning u32 for
            plan-readability.
==============================================================================

WHAT
The plan markdown describes ArmPosterior with `pass_count: u32` and
`fail_count: u32`. The shipped struct has `alpha: f64` and `beta: f64`
(Beta-distribution conjugate notation). The closure left the f64 fields
alone and added two compatibility accessors:
  - ArmPosterior::pass_count_estimate() -> u32 (= floor(alpha - 1).clamp(0))
  - ArmPosterior::fail_count_estimate() -> u32 (= floor(beta - 1).clamp(0))

WHY
Changing the field types from f64 -> u32 would lose the ability to feed
soft (non-integer) rewards into the posterior — a documented future
extension (bai.rs:74-78 module header). Beta(alpha, beta) is the
canonical conjugate parameterisation in every reference (Audibert,
Variance-BAI, statrs, scipy.stats.beta); flipping to "pass/fail" naming
would be a step BACK from the canonical form. The accessors give the
plan its u32 surface without touching the storage.

GROUNDING
- Read: bai.rs:79-129 (existing ArmPosterior + module header rationale)
- Added: bai.rs ArmPosterior::pass_count_estimate / fail_count_estimate
  (alongside existing mean() + is_stopped()).
- Contract test: crates/contracts/tests/bai_contract.rs
  `pass_fail_count_accessors_track_alpha_beta_minus_one`.

==============================================================================
DECISION-4: credible_interval implementation is HAND-ROLLED via
            Lanczos ln_gamma + continued-fraction regularized incomplete
            beta + Newton inverse. NO new dependency (statrs NOT pulled).
==============================================================================

WHAT
crate::bai::credible_interval(posterior, level) returns (lo, hi) via:
  1. ln_gamma (Lanczos approximation, Numerical Recipes §6.1).
  2. regularized_incomplete_beta (continued-fraction, NR §6.4).
  3. inv_regularized_incomplete_beta (Newton iteration on the CDF).

The three private helpers add ~120 LOC. No statrs / no `special` / no
runtime dep changes.

WHY
CLAUDE.md HARD RULE: "No new dependencies without confirmation." statrs
is not in Cargo.toml — adding it for one function would (a) need
confirmation, (b) add a transitive surface that hand-rolled f64 math
does not, and (c) duplicate the well-trodden numerical-recipes path
the BAI module already uses (Marsaglia + Tsang gamma sampling lives in
the same file). The hand-rolled version is documented inline as
"implementation choice: not directly grounded in arxiv:2106.10417 paper
formula" per the prompt's explicit tag requirement.

GROUNDING
- Numerical Recipes 2007, §6.1 (Lanczos ln_gamma) + §6.4 (betacf CF).
- Existing precedent: bai.rs `sample_gamma` / `sample_beta` are also
  hand-rolled numerical-recipes implementations with no external dep.
- workflow/papers/variance-bai-2106.10417-interface.md drives the
  H_var formula expansion but does NOT prescribe a credible-interval
  formula — its concentration uses Bernstein + KL, not Beta-credible
  intervals. The credible interval is the standard Bayesian
  diagnostic, applied here for the dashboard / per-template summary
  surface (NOT for the Thompson decision itself).

==============================================================================
DECISION-5: log_bar(K) = 0.5 + sum_{i=2..=K} 1/i, NOT the natural logarithm
==============================================================================

WHAT
crate::bai::log_bar_k(K) returns 0.5 + sum_{i=2..=K} 1/i. For K = 2
this is exactly 1.0 (0.5 + 1/2). For K = 3 it is 1.333... (0.5 + 1/2 +
1/3). For K = 5 ~ 1.7167.

WHY
Audibert / Bubeck / Munos (COLT 2010) Section 4 defines log_bar(K) as
a harmonic-number variant. Implementation pitfall #6 in the paper
interface explicitly flags "Do NOT use log(K) (natural log)" because
ln(2) = 0.6931 < 1, which would push more pulls into Successive
Rejects's last phase than the budget proof allows. The contract test
`log_bar_k_is_not_natural_log` (bai_contract.rs) regression-guards this.

GROUNDING
- workflow/papers/audibert-colt-2010-bai-interface.md lines 43, 102, 286
- Used by: crate::bai::successive_rejects (the SR phase-length formula
  n_k = floor((n - K) / (log_bar(K) * (K + 1 - k)))).

==============================================================================
DECISION-6: emit_promotion_round phantom function references fixed
==============================================================================

WHAT
The bai.rs module header (line 28) cited a function
`crate::agent_pool::emit_promotion_round` which DOES NOT exist. The
archive_chain.rs comment (line 275) had the same phantom reference.

Both were edited to point at the real function: `bai::emit_bai_round`
(defined in bai.rs at line 567, with the SSB-emitting wire surface).

WHY
The phantom citation was a leftover from an earlier sketch where the
wire layer lived in agent_pool. The actual implementation lives in
bai.rs itself; rustdoc was generating broken `[crate::agent_pool::
emit_promotion_round]` links. Fixing the comments restores the doc
build + makes the wiring trail discoverable from the module surface.

GROUNDING
- Pre-edit: bai.rs:28 (broken intra-doc link)
- Pre-edit: archive_chain.rs:275 (broken comment reference)
- Post-edit: both point at `crate::bai::emit_bai_round`
- Verification: `grep emit_promotion_round .elai_cc/crates/orchestrator/src/`
  returns ZERO matches after the fix (closure-end self-check item).

==============================================================================
DECISION-7: GAP-G ts-rs binding REPLACED with hand-authored TS mirror in
            types.ts (per the explicit workspace policy already documented
            at types.ts:2339)
==============================================================================

WHAT
The plan asks to "Add ts-rs binding to the Rust struct" for BaiArmRow.
The actual closure added hand-authored TS interfaces (BaiArmRow,
ThompsonSampledRow, BaiUnionRow) to
`crates/dashboard-v2/src/lib/api/types.ts`. The Rust DTO at
`crates/dashboard-v2/src-tauri/src/commands_d5.rs:33` is left unchanged.

WHY
The codebase comment at types.ts:2339 reads literally:
  "Hand-authored TS mirror (workspace policy — types.ts is the shared
  surface, ts-rs is not used in this crate)."

Adding ts-rs would (a) violate the workspace policy + (b) introduce a
NEW dependency (CLAUDE.md HARD RULE: "No new dependencies without
confirmation"). The hand-authored mirror covers the same surface
(BaiArmRow + ThompsonSampledRow as a tagged-union BaiUnionRow) without
either violation. The dashboard panel
(crates/dashboard-v2/src/lib/drawers/BaiArmsPanel.svelte) currently
inlines its own BaiArmRow type — a follow-on patch can switch it to
import from types.ts for consistency.

GROUNDING
- Read: types.ts:2334-2339 (explicit hand-authored policy comment)
- Read: dashboard-v2/Cargo.toml (no ts_rs / ts-rs dep present)
- Pattern precedent: B-7 PackageRefVerified / FailureRecord TS mirrors
  at types.ts:549-566 (same hand-authored pattern).

==============================================================================
Closing self-check (executor-emitted)
==============================================================================

  - SCHEMA_VERSION_PUBLIC: 110 (verified by grep)
  - ThompsonSampled cascade points: 9 (CHECK / enum / as_sql_str /
    from_str / SsbPayload variant / action_type arm / test_payload arm /
    all_test_payloads arm / action_type_lock ALL_VARIANTS +
    exhaustive_match_guard + golden ordinal 290)
  - emit_promotion_round phantom references: 0 (verified by grep)
  - emit_bai_round call in task_runner.rs: present (via dispatch_bai_round)
  - dispatch_bai_round invoked post-dispatch_verifiers: yes
  - Contract test file: crates/contracts/tests/bai_contract.rs (10 tests)
  - ELAI_BAI_ENABLE env gate: BaiRuntimeConfig::is_enabled
  - [bai] toml section: present in elai_cc.toml (commented opt-in)
  - No new external dependencies introduced
  - No unwrap() / expect() in production paths
  - No async-trait / log crate / Box<dyn Error>
