H-42 decisions — serial-depth-scaled reasoning budget + depth-not-width guard

WHAT
Scale the per-role GRC budget by estimated serial decomposition depth, bounded by
a locked ceiling. Add a FailureClass diagnostic to distinguish depth-class failures
(fix = more steps) from access-class failures (fix = more context).

KEY DECISIONS

D-1: CoT-step lower bound grounding (why depth matters)
Grounded in Merrill & Sabharwal arXiv:2310.07923 (single forward pass is bounded to
~TC0; polynomial CoT reaches exactly P), Li et al. arXiv:2402.12875 (T serial CoT
steps simulate a size-T circuit — serial steps are the expressivity resource), and
Yehudai et al. arXiv:2503.01544 (~O(n) CoT tokens needed for compositional reasoning
with tight step lower bounds). A flat per-role budget therefore UNDER-budgets a deep
serial sub-task. H-42 scales the budget proportionally to depth.

D-2: Single-pass ~TC0 bound (why context != depth) — basis for depth-not-width guard
A bigger context window buys information ACCESS (more tokens available per ~TC0 pass),
not compute DEPTH (circuit depth class). The formal result (Merrill & Sabharwal
2310.07923 §4) means a context lift is structurally insufficient to fix a depth-class
failure. This is the RD-4 grounding: a DepthBound row's remedy is MORE serial CoT
steps or finer decomposition, NEVER a larger context window. Applying a context lift
to a depth_bound failure is a category error.

D-3: Locked ceiling rationale (loop-inflation proof)
LOCKED_BUDGET_CEILING = 3 is a compile-time constant. Nothing at runtime can raise it.
A deep task can never inflate its own budget beyond 3x the base — the loop that uses
the budget cannot use the budget to increase the budget. The starter value (3x) is
conservative; calibrate on >=3 measured budget-exhaustion runs before raising
(GD-4, RISK-3 from context-package).

D-4: Scale factor selection (starter configuration)
factor = 1 + 0.25 * d, where d is 0-based decomposition level depth (BabyStep=3,
Step=2, Phase=1, Concept=0). At BabyStep d=3, factor=1.75x. This is a conservative
starter — empirical motivation only, not derived from the ~O(n) bound directly. The
~O(n) bound (2503.01544) gives direction (deeper = more steps needed) not a specific
multiplier. The exact factor should be calibrated on measured ELAI runs once the
instrument is live (see RISK-3 / GD-4).

D-5: FailureClass as struct, not strict single-arm enum (RISK-2 mitigation)
A sub-task that is simultaneously deep (high d) AND retrieval-starved is BOTH
depth_bound AND access_bound. Forcing a single discriminant would over-claim a clean
dichotomy. FailureClass uses two boolean flags { depth_bound, access_bound } instead
of an enum, so both can be true simultaneously. The dominant SSB Kind (depth_bound
wins over access_bound when both are true) surfaces the depth-not-width signal for
downstream consumers without suppressing the access signal.

D-6: depth signal choice (cheapest shipped one — GD-1)
Option A (DecompositionLevel from pipeline_tree_navigator): free, deterministic, 0
lines of new infrastructure. BabyStep maps to d=3, Concept to d=0. This is the floor
signal. Option B (DKG reachability-chain length from rules/predicate.rs::DkgSubgraphSnapshot)
gives a richer per-task signal for code tasks but requires a DkgQuerier instance.
H-42 ships Option A as the default and is designed to compose with Option B in a
future patch (dispatch sites can pass a richer level estimate when available).

D-7: Default-ON, kill-switch only (no wire-dark-allowlist row needed)
Scaling is default-ON because it directly improves budget allocation correctness with
no user-visible side effects. A kill-switch (ELAI_REASONING_BUDGET_SCALE_DISABLE)
is provided for emergency rollback. Default-ON flags do not require a
wire-dark-allowlist.toml row (R9 only applies to default-OFF flags). Kill-switch
type = kill_switch (ops, may be long-lived); no expiry_date required for kill_switch
type per wiring-enforcement.md R9.

HONEST RESIDUE (GD-5)
Depth-scaling helps a SERIAL-DECOMPOSABLE task that genuinely needs more serial
reasoning steps. It does NOT help a task that is above the model's reachable
complexity class even with CoT (e.g., a genuinely NP-complete or undecidable
sub-task). Those must route to a solver/tool (H-41 checker-coverage, E-53
FormalVerifierCompleted) or refuse (D-16 RefusalDecision). "More steps" is not a
cure for out-of-class tasks. The DepthBound classification MUST propagate "try
finer decomposition before throwing more compute at it" as the first remedy.

EXPECTED TEST OUTCOME
cargo check --workspace --all-targets: 0 errors (warnings only)
cargo test -p elai-orchestrator --test action_type_lock: 2 passed (220/531 consistent, golden regenerated)
cargo test --test h42_serial_depth_budget_contract: 24 passed, 0 failed
cargo test -p elai-orchestrator --lib -- reasoning_budget: 17 passed, 0 failed

UNKNOWNS / OPEN ITEMS
- Scale factor calibration: 1 + 0.25*d is a starter; measure budget-exhaustion rates
  over >=3 real runs to determine if the factor needs adjustment (GD-4).
- DKG reachability-chain signal (Option B): not integrated in this phase. Future patch.
- Dashboard wiring (ReasoningDrawer columns): W-4 frontend_pair deferred; not blocked
  on core logic. Dashboard panel can be added in a follow-on H-plan patch.
- Live GPU/RunPod benchmark runs: deferred per POLICY (H-42 plan explicitly allows
  deferring live GPU benchmark runs).
