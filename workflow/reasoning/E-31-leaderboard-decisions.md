E-31 coding-model-leaderboard scraper substrate decisions
Date: 2026-05-28
Session: s-25120 completion-drive batch (E-31 re-scope)
Scope: Pattern-2 single SSB variant + benches/external scraper + ENV gate.
Defers: sidecar boot assert + EscalationPolicy + RuntimeOrchestrator hook +
dashboard panel + CLAUDE.md pointer.

(a) Scope split rationale (E-31.A SHIPPED + E-31 leaderboard scope this
session + E-31.B sidecar + E-31.C tier-matrix)

E-31.A SHIPPED (prior session s-orch-e31):
  - Task 7: .elai_cc/crates/contracts/tests/head_dim_invariant_contract.rs
    (CI-resilient guard against the dense-128 invariant at the config.json
    layer; skipped if weights absent).
  - Task 8: workflow/reasoning/E-31-decisions.md (4 plan-drifts catalogued).

This session (E-31 leaderboard scraper scope, per session prompt):
  - SCHEMA_VERSION 173 -> 174 atomic bump.
  - Single SsbActionType variant CodingModelLeaderboardEvent + 3-arm
    LeaderboardEventKind discriminator (scrape_started / scrape_completed
    / scrape_failed). Pattern-2 26th application (sibling D-19..D-30 +
    E-2/E-3/E-7/E-9/E-10/E-11/E-19/E-22/E-23/E-25/E-26/E-29/E-42/E-43/E-54/E-55).
  - TOTAL_VARIANTS 446 -> 447; ALL_VARIANTS + exhaustive match + golden
    file updated lockstep in the same commit.
  - benches/external/coding_model_leaderboard.py scraper + _common.py
    helper class CodingModelLeaderboardEventRow + score_str_to_bp parser.
  - crates/contracts/tests/e31_coding_model_leaderboard_ssb_contract.rs
    (round-trip per kind + ENV-gate const + score_pct_bp range).
  - ENV gate ELAI_CODING_MODEL_LEADERBOARD_ENABLE; wire-dark default.
  - Live HTTP scrape RUNPOD-PENDING.

E-31.B DEFERRED (future session):
  - python/elai_inference/server.py boot-time assert_head_dim_invariant
    (DRIFT-3: server.py:1993 / :1997 / :2093 silently default head_dim to
    256 via getattr; Qwen2.5-family has no head_dim field and computes
    hidden_size/num_attention_heads = 3584/28 = 128).
  - EscalationPolicy + RuntimeChoice (DRIFT-1: RuntimeOrchestrator::
    pick_runtime does NOT exist; real entry is try_complete + Wave-1B
    PENDING per cli/src/runtime_cmd.rs:14 + 271-277. Cloud routing lives
    in cloud_mode.rs::CloudModeRouter::route).
  - File location collision (DRIFT-2: providers/src/runtime/escalation_
    policy.rs name-collides with orchestrator/src/escalation_policy.rs
    SHIPPED from E-32 with HandoffContext/HandoffDecision/EscalationPolicyError).
    Recommendation: rename the cloud-route module to cloud_route_policy.rs
    or runtime_route_policy.rs.

E-31.C DEFERRED (rides with baseline measurement):
  - workflow/improvement-sweep-2026-05-19/E-plan/E-31-tier-matrix.md.
  - CLAUDE.md pointer + Qwen3-family REJECTED line.
  - .elai_cc/benches/coding_lane/ (renamed to benches/external/ in this
    session per existing scraper-scaffolding convention; the plan-original
    benches/coding_lane/ dir does not exist).

(b) Pattern-2 single-variant decision (vs 3 variants)

Plan-original called for 2 variants (CloudRouteDecision +
HeadDimInvariantVerified). Both producers live in AVOIDED crates this
session (sidecar python + providers/runtime/) per DRIFT-3 + DRIFT-1 +
DRIFT-2. Per E-31-decisions.md (d) the >2-variant deferral threshold
applies; a 2-variant atomic bump with no producers is dead schema.

The leaderboard scraper is the only producer that lands this session, so
it gets exactly one variant + one Pattern-2 discriminator. Future
producers (cloud-route policy + head-dim assert) get their own SsbActionType
variants when their crates ship.

This collapses the originally planned 2 variants into 1 by re-scoping
under the session prompt "coding-model leaderboard scraper module". The
plan body's CloudRouteDecision + HeadDimInvariantVerified are unchanged
in intent -- they re-emerge in E-31.B alongside their producers.

(c) Wire-dark gate design

ENV gate: ELAI_CODING_MODEL_LEADERBOARD_ENABLE
  - Match-string is "1" or "true" (case-insensitive); any other value
    leaves the emitter as a noop per existing skill_telemetry.rs:89-97 +
    skills_cmd.rs:400 precedent.
  - Default-off so existing tests (action_type_lock_count_is_locked +
    test_typed_payloads payload count 384 -> 385) keep their expected SSB
    row counts.
  - Scraper at benches/external/coding_model_leaderboard.py refuses live
    scrape AND emits a scrape_failed row with reason_label='env_gate_off'
    when the env var is unset. This way the dispatcher always observes
    the refusal -- no silent skip.

(d) score_pct_bp wire shape (basis points NOT float)

Scores arrive in heterogeneous formats: "46%" / "0.46" / "46.3" / "46.3%"
/ integer 46 / integer 4600. The wire field is u32 basis points (0-10000):
  - 0.0 to 1.5: treated as fraction, multiply by 10000 (0.46 -> 4600)
  - else: treated as %, multiply by 100 (46.3 -> 4630, 46 -> 4600)

Rationale: float drift on msgpack round-trip + sqlite REAL is a known
hazard (prior B-40 round-trip failure with float scores). Basis points
keeps the round-trip deterministic and the contract test exercises the
exact bit pattern.

(e) blake3 entry_hash for dedup (NOT blake2 / NOT sha2)

Per CLAUDE.md Anti-Stale-Training HARD RULE: blake3, never blake2/sha2.
The 8-byte hex prefix is for dedup only -- not cryptographic. Scraper
falls back to sha256 prefix only if the blake3 dependency is missing
(logged in details_json so the consumer can spot the degradation).

The canonical row tuple includes scrape_ts_ms rounded to the nearest
hour so two scrapes of the same row in quick succession dedup to the
same hash. This keeps the dispatcher dedup table small without losing
the "this was actually scraped twice" signal in details_json.

(f) Citation-verification (per memory:feedback_paper_citation_verify)

Prior to encoding any quoted figure into a comment or payload, WebSearched
every cite the plan body lists. All 8 confirmed live as of 2026-05-28:
  - arxiv.org/abs/2409.12186 (Qwen2.5-Coder tech report) -- VERIFIED.
  - arxiv.org/abs/2509.16941 (SWE-Bench Pro paper) -- VERIFIED.
  - morphllm.com/swe-bench-pro -- VERIFIED. Confirms Opus 4.5: 80.9
    Verified / 45.9 Pro; contamination gap is the headline number.
  - labs.scale.com/leaderboard/swe_bench_pro_public -- VERIFIED. Public
    dataset has 731 tasks; held-out + commercial behind login.
  - bigcode-bench.github.io -- VERIFIED. ICLR'25 paper; 1140 task full,
    ~150 task hard subset.
  - evalplus.github.io/leaderboard.html -- VERIFIED. Phi 4 Reasoning
    leads HumanEval+ at 92.9 as of search date.
  - livecodebench.github.io/leaderboard.html -- VERIFIED. Time-windowed
    contamination-free design.
  - platform.claude.com/docs/en/about-claude/pricing -- VERIFIED.
    Sonnet 4.6 $3/$15, Opus 4.7 $5/$25, Haiku 4.5 $1/$5 per MTok.

Plan body claims that DID NOT verify cleanly and are therefore NOT
encoded as authoritative payload fields:
  - benchlm.ai/blog/posts/claude-api-pricing (cite was second-hand;
    primary platform.claude.com is now the wire-grounding).
  - marc0.dev May 2026 leaderboard (cite as "marc0.dev" gave no direct
    URL; codeant.ai/blogs/swe-bench-scores + aicodereview.cc serve the
    same Verified-leaderboard role and are referenced as alt sources).
  - insiderllm.com/guides + nathan.sapwell.net + metr.org/blog +
    thamizhelango.medium.com + openhands.dev/blog -- not used as
    grounding for any payload field this session; deferred to E-31.B/C
    where they may surface in the tier-matrix doc rationale.

(g) DRIFT catalogue (4 HIGH-severity drifts confirmed; 3 deferred, 1
fixed in this commit)

DRIFT-1 (HIGH, DEFERRED to E-31.B): plan Task 4 names RuntimeOrchestrator
::pick_runtime as the EscalationPolicy hook surface; that fn does not
exist. Real surface is try_complete + health_snapshot (Wave-1B PENDING).
Cloud routing lives in cloud_mode.rs::CloudModeRouter::route. The
correct hook is on CloudModeRouter not RuntimeOrchestrator. E-31.B
authors EscalationPolicy + wires it into CloudModeRouter::route.

DRIFT-2 (HIGH, DEFERRED to E-31.B): plan Task 3 directs creating
providers/src/runtime/escalation_policy.rs as NEW; that basename collides
with orchestrator/src/escalation_policy.rs SHIPPED from E-32. The
collision is a footgun for cargo / IDE / grep. E-31.B renames the new
file to cloud_route_policy.rs OR runtime_route_policy.rs at a NEW
location (not providers/src/runtime/).

DRIFT-3 (HIGH, DEFERRED to E-31.B): server.py:1993 + :1997 + :2093 read
head_dim with `getattr(..., "head_dim", 256)` DEFAULT 256. Qwen2-family
has NO head_dim field; the correct value computes from
hidden_size/num_attention_heads (3584/28 = 128 on Qwen2.5-Coder-7B).
The current silent default 256 masks the head_dim invariant the
companion contract test (head_dim_invariant_contract.rs, SHIPPED in
E-31.A) is designed to catch. E-31.B authors assert_head_dim_invariant
that computes head_dim the same way: hidden_size/num_attention_heads.

DRIFT-4 (HIGH, FIXED this session): 12+ paper/URL cites in plan
UNVERIFIED at plan-author time. WebSearched all 12, 8 confirmed (see
section (f)), 4 deferred to where they would actually be used. No
quoted figure is encoded into a payload field without a verified cite.

(h) Lockstep sites touched this commit (8 sites)

  1. ssb.rs SCHEMA_VERSION 173 -> 174
  2. ssb.rs SQL CHECK 'coding_model_leaderboard_event' arm
  3. ssb.rs SsbActionType::CodingModelLeaderboardEvent variant
  4. ssb.rs as_sql_str() arm
  5. ssb.rs ssb_action_type_from_str() arm
  6. ssb.rs SsbPayload::CodingModelLeaderboardEvent arm + action_type()
  7. ssb.rs test_payload() fixture builder + all_test_payloads() round-trip
  8. action_type_lock.rs TOTAL_VARIANTS 446 -> 447 + ALL_VARIANTS append
     + exhaustive_match_guard arm + action_type_lock.golden regenerate

(i) Open questions (E-31.B work items)

  - Live HTTP scrape strategy: SearXNG-mediated fetch vs direct urllib
    vs offline mirror? E-31.B picks one.
  - Anti-rate-limit: morphllm + Scale labs + EvalPlus serve from
    cloudflare; the scraper needs an exponential-backoff + retry-after
    honor. RUNPOD-PENDING.
  - Cross-suite correlation: when 5 suites all report Opus 4.7's number,
    the dispatcher should emit a synthetic `cross_suite_correlation_
    computed` row carrying the (mean, std, n) across suites. Deferred
    to E-31.B alongside the live scrape.
  - Dashboard wiring: no dashboard panel ships this session (session
    prompt: NO dashboard). E-31.B adds a CodingModelLeaderboardPanel
    once the scraper has produced enough live rows to render usefully.
