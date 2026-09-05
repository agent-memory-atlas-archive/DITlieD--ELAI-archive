DASHBOARD-V2 HANDLER JSON-SHAPE SHEET
task_id: handler-shapes-v1
scope: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands*.rs + DTO structs they reference (orchestrator/knowledge/etc).

WHAT THIS IS
The CONTRACT-SHEET.md maps each panel -> command -> handler file:line, but most handlers return CmdResult<Value>
(untyped serde_json::Value), so the contract sheet only carries the FRONT-END's asserted TS type. This sheet records
the ACTUAL JSON FIELD SHAPE each handler sends to the UI, transcribed from the real handler body (the serde_json::json!{}
literal or the typed DTO struct it serializes). Every block cites the real file:line read this session. A field NOT seen
in source is listed UNRESOLVED, never invented.

KEY PATTERN: most per-plan commands_<plan>.rs handlers do `viewer.<query>(cap)` returning Vec<SomeRow> and
serde_json::to_value(rows). The wire shape is therefore the *Row struct, which lives in
crates/orchestrator/src/ssb_viewer.rs (serde serializes Rust field names verbatim — confirmed by the module docs).
Shapes below are transcribed from those struct defs (file:line cited).

LEGEND per block:
  command_name  (handler file:line)  [returns: Value-json! | DTO:<name>@<file:line> | Vec<DTO> | array]
    <field>: <rust type>            (one line per top-level field; sub-objects nested <=2 levels)
    consumed_by: <panel(s) from CONTRACT-SHEET Section 1>


================================================================================
commands_g1.rs  (G-1 taint flow)
================================================================================
get_taint_flows  (commands_g1.rs:48)  [returns: Vec<TaintFlowRow> -> json array; empty [] if no viewer]
  element = TaintFlowRow @ ssb_viewer.rs:5415
    seq: i64                  (SSB sequence number, newest-first)
    ts_ms: i64                (row creation wall-clock ms)
    kind: String              (ingest kind: web | tool_output | file | sandbox_output)
    origin_blake3: String     (blake3 hex-prefix of ingest origin)
    trust_level: String       (joined trust label, e.g. "untrusted")
    sink_tool: String         (destructive sink tool blocked, e.g. git_commit | edit)
    blocked: bool             (true for an emitted taint_flow row)
    task_id: String           (task/turn id for correlation)
  consumed_by: TaintFlowDrawer

================================================================================
commands_g2.rs  (G-2 RRA / reproducible-run-artifact)
================================================================================
get_run_artifact  (commands_g2.rs:50)  [returns: Vec<RunArtifactRow> -> json array; empty [] if no viewer]
  element = RunArtifactRow @ ssb_viewer.rs:5439
    seq: i64
    ts_ms: i64
    root_blake3: String       (blake3 hex-prefix of canonical .rra bundle bytes — the root)
    guarantee_level: String   (honest per-lane label: reproducible_action_stream | verify_divergence)
    gate_count: u32           (F-gate verdict-hash entries in evidence manifest)
    seed: u64                 (pinned ELAI_SEED the action stream re-derives under)
    session_id: String
    objective_hash: Option<String>   (#[serde(default)]; 16-hex blake3, None on pre-I-45 receipts)
  consumed_by: RraViewerPanel

================================================================================
commands_g3.rs  (G-3 blast radius)
================================================================================
get_blast_radius_blocks  (commands_g3.rs:53)  [returns: Vec<BlastRadiusBlockRow> -> json array; empty [] if no viewer]
  element = BlastRadiusBlockRow @ ssb_viewer.rs:5617
    seq: i64
    ts_ms: i64
    kind: String                    (fs_escape | egress_denied)
    pre_expansion_command: String   (original un-expanded argv the agent issued)
    post_expansion_command: String  (resolved/canonicalised target or rejected host+ip)
    target_or_host: String          (resolved FS target path, or rejected egress host)
    worktree_root_blake3: String    (blake3 hex-prefix of worktree root; empty for egress arm)
    trust_label: String             (G-1 provenance trust join; "" for pure-containment block)
  consumed_by: BlastRadiusDrawer

================================================================================
commands_g4.rs  (G-4 MCP rug pull)
================================================================================
get_mcp_rug_pulls  (commands_g4.rs:49)  [returns: Vec<McpRugPullRow> -> json array; empty [] if no viewer]
  element = McpRugPullRow @ ssb_viewer.rs:5390
    seq: i64
    ts_ms: i64
    server_url: String
    schema_blake3_before: String    (blake3 schema hash at pin time)
    schema_blake3_after: String     (blake3 schema hash of rug-pulled re-fetch)
    changed_tools: Vec<String>      (tool names whose desc/presence changed)
    description_diff_count: u32      (per-tool description swap count)
    ingest_origin: String           (production entrypoint that re-fetched)
  consumed_by: McpRugPullDrawer

================================================================================
commands_g5.rs  (G-5 review surface)
================================================================================
get_review_surface  (commands_g5.rs:52)  [returns: Vec<ReviewSurfaceRow> -> json array; empty [] if no viewer]
  element = ReviewSurfaceRow @ ssb_viewer.rs:5579
    seq: i64
    ts_ms: i64
    session_id: String
    dsr: f64                        (Diff-Surface-to-Review ratio in [0,1])
    total_changed_lines: u32        (DSR denominator: total added+removed changed lines)
    attention_lines: u32            (DSR numerator: changed lines in un-vouched hunks)
    vouched_hunks: u32
    unvouched_hunks: u32
    base_commit: String             (resolved diff-base commit SHA)
    diff_available: bool            (false ONLY on git error; renders "unavailable" banner not clean DSR=0)
    diff_unavailable_reason: String (git error reason when diff_available==false; "" otherwise)
  consumed_by: CockpitWindow (get_review_surface)

================================================================================
commands_g6.rs  (G-6 SAFEDELTA)
================================================================================
get_safety_delta_latest  (commands_g6.rs:74)  [returns: Option<SafetyDeltaLatest>; null if no viewer/no row]
  DTO = SafetyDeltaLatest (local struct, commands_g6.rs; projected from SsbPayload::SafetyDeltaMeasured)
    suite: String                (e.g. secodeplt_static_gate)
    model_id: String
    safety_delta: f64
    ci_low: f64
    ci_high: f64
    n_tasks: <int>               (count of tasks)
    seed: <int>
    corpus_blake3: String
    gate_recall: f64
    runpod_pending: bool
    ts: i64                      (row ts from SSB entry)
  consumed_by: SafeDeltaPanel   (BENCH-PENDING — null until a bench runs)

================================================================================
commands_g7.rs  (G-7 claim confidence + trifecta quarantine)
================================================================================
get_claim_confidence  (commands_g7.rs:55)  [returns: Vec<ClaimConfidenceRow> -> json array; empty [] if no viewer]
  element = ClaimConfidenceRow @ ssb_viewer.rs:5519
    seq: i64
    ts_ms: i64
    claim_id: String
    claim_score: f64                     (calibrated fused confidence [0,1])
    logprob_component: Option<f64>       (None on cloud lane)
    self_consistency_component: f64      ([0,1])
    exec_agreement_component: f64        ([0,1])
    lane: String                         (local | cloud)
    tau: f64                             (conformal threshold)
    conformal_verdict: String            (abstain | answer)
    abstained: bool
  consumed_by: ConfidenceDrawer

get_trifecta_quarantines  (commands_g7.rs:73)  [returns: Vec<TrifectaQuarantineRow> -> json array; empty [] if no viewer]
  element = TrifectaQuarantineRow @ ssb_viewer.rs:5549
    seq: i64
    ts_ms: i64
    task_id: String
    untrusted_origin_blake3: String   (blake3 prefix of untrusted origin; never raw span)
    sink_tool: String                 (egress/destructive sink blocked)
    secret_present: bool              (a: secret pattern matched in live context)
    untrusted_span: bool              (b: pending span provenance was Untrusted)
    egress_available: bool            (c: pending tool was egress channel)
    quarantined: bool
    sub_llm_verdict: String           (no-tool sub-LLM verdict label)
  consumed_by: ConfidenceDrawer

================================================================================
commands_g8.rs  (G-8 attestation)
================================================================================
get_attestations  (commands_g8.rs:54)  [returns: Vec<AttestationRow> -> json array; empty [] if no viewer]
  element = AttestationRow @ ssb_viewer.rs:5467
    seq: i64
    ts_ms: i64
    session_id: String
    subject_blake3: String        (in-toto subject digest: blake3-Merkle root over predicate)
    run_artifact_root: String     (G-2 bundle root the attestation chains to)
    verified_levels: u32          (count of F-gate verdicts in SLSA-VSA rollup)
    all_passed: bool
    signing_model: String         (default local_blake3_merkle)
  consumed_by: AttestationDrawer

================================================================================
commands_g9.rs  (G-9 redaction stats)
================================================================================
get_redaction_stats  (commands_g9.rs:51)  [returns: Vec<RedactionStatRow> -> json array; empty [] if no viewer]
  element = RedactionStatRow @ ssb_viewer.rs:5495
    seq: i64
    ts_ms: i64
    session_id: String
    field: String                 (redacted SsbPayload field identity, e.g. idea_framing_started.prompt)
    tokens: Vec<String>           (stable blake3-keyed redaction tokens that replaced matches)
    match_count: u32
    kind_counts: Vec<(String, u32)>  (sorted (kind_label, count); serializes as JSON array of [string,number] tuples)
  consumed_by: RedactionPanel

================================================================================
commands_g10.rs  (G-10 model drift)
================================================================================
get_model_drift_status  (commands_g10.rs:72)  [returns: Option<ModelDriftStatus>; null if no viewer/no row]
  DTO = ModelDriftStatus (local struct, commands_g10.rs; projected from SsbPayload::ModelDriftSuspected)
    identity_key: String
    kind: String                          (drift kind label, e.g. canary_regressed)
    signals: Vec<String>
    per_family_delta: Vec<(String, f64)>  (serializes as JSON array of [family,delta] tuples)
    affected_template_ids: Vec<String>
    confirmed_by_fingerprint: bool
    baseline_captured_at: i64
    ts: i64
  consumed_by: ModelDriftPanel

================================================================================
commands_d1.rs  (D-1 mutation/reasoning-template archive)
================================================================================
get_archive_recent  (commands_d1.rs:52)  [returns: Vec<ArchiveRecentRow>; empty if no file / verify_chain runs first]
  element = ArchiveRecentRow (local struct @ commands_d1.rs:30)
    template: ReasoningTemplateDto   (one hop — see NOTE below)
    self_hash: String                (hex)
    parent_hash: Option<String>      (hex)
    promoted_at_ms: Option<i64>
    retired_at_ms: Option<i64>
    retirement_reason: Option<String>
  consumed_by: MutationArchivePanel
  NOTE: ReasoningTemplateDto fields not transcribed (1-hop type in orchestrator); carries at least id (hex). UNRESOLVED-SUBOBJECT.

get_reasoning_template_by_id  (commands_d1.rs:80)  [returns: Option<ReasoningTemplateDto>; not in dashboard Section-1 list]
  consumed_by: (not in Section 1 work list)

================================================================================
commands_d2.rs  (D-2 PPM scores)
================================================================================
get_ppm_recent_scores  (commands_d2.rs:49)  [returns: Vec<PpmScoreRow>; merges ppm_scored + ppm_gating_failed, empty if no viewer]
  element = PpmScoreRow (local struct @ commands_d2.rs:30)
    seq: i64
    ts: i64
    kind: &'static str        ("scored" gate passed | "gating_failed" gate tripped)
    candidate_id: String
    verifier_id: String
    mode: String
    score_mean: f64
    score_var: Option<f64>
    num_steps: u32
    threshold: Option<f64>    (None on scored rows, Some on gating_failed)
    latency_ms: Option<u64>   (scored rows only)
    cache_hit: Option<bool>   (scored rows only)
  consumed_by: PpmScoresPanel

================================================================================
commands_d3.rs  (D-3 verification consensus)
================================================================================
get_recent_verifications  (commands_d3.rs:52)  [returns: Vec<VerificationConsensusRow>; merges 5 action types, empty if no viewer]
  element = VerificationConsensusRow (local struct @ commands_d3.rs:30)
    seq: i64
    ts: i64
    kind: &'static str        (pass | fail | ppm_unavailable | verifier_disagreement | cross_context_dispatched)
    candidate_id: String
    template_id: String
    verdict: String
    rule_id: String
    verifier_count: Option<u32>       (pass rows only)
    pass_count: Option<u32>           (pass rows only)
    failing_verifier_ids: Vec<String> (fail rows)
    reason: Option<String>            (fail rows)
  consumed_by: VerificationConsensusPanel

================================================================================
commands_d4.rs  (D-4 tripwires)
================================================================================
get_recent_tripwires  (commands_d4.rs:64)  [returns: Vec<TripwireRow>; empty if no viewer]
  element = TripwireRow (local struct @ commands_d4.rs:30)
    seq: i64
    ts: i64
    kind: &'static str               (fired | auto_reverted | fired_concurrent)
    event_id: Option<String>
    tripwire_kind: Option<String>
    source: Option<String>           (#[serde(default)] reward|action|perplexity|classifier; None on legacy/other)
    candidate_id: Option<String>
    observed_value: Option<f64>
    threshold: Option<f64>
    fired_at_ms: Option<i64>
    template_id: Option<String>
    parent_template_id: Option<String>
    retirement_reason: Option<String>
    fired_tripwire_id: Option<String>
    tripwire_a: Option<String>       (#[serde(default)]; concurrent rows only)
    tripwire_b: Option<String>       (#[serde(default)]; concurrent rows only)
    fired_within_ms: Option<u64>     (#[serde(default)]; concurrent rows only)
  consumed_by: TripwireStatusStrip + TripwirePanel

================================================================================
commands_d5.rs  (D-5 BAI arms)
================================================================================
get_bai_arms  (commands_d5.rs:51)  [returns: Vec<BaiArmRow>; sorted by mean_estimate desc, empty if no viewer]
  element = BaiArmRow (local struct @ commands_d5.rs:33)
    template_id: String
    alpha: f64
    beta: f64
    n_pulls: u32
    was_promoted: bool
    mean_estimate: f64       (alpha/(alpha+beta), computed Tauri-side)
  consumed_by: BaiArmsPanel

================================================================================
commands_d6.rs  (D-6 held-out tasks)
================================================================================
get_held_out_recent  (commands_d6.rs:61)  [returns: Vec<HeldOutTaskRow>; sorted by ingested_ts_ms desc, empty if no viewer]
  element = HeldOutTaskRow (local struct @ commands_d6.rs:34)
    task_id: String
    source: String
    content_blake3: String
    ngram_count: u32
    ingested_ts_ms: i64
    eligibility_window_ms: i64
  consumed_by: HeldOutPanel

get_held_out_stats  (commands_d6.rs:98)  [returns: HeldOutStatsRow; zero-filled if no viewer]
  DTO = HeldOutStatsRow (local struct @ commands_d6.rs:45)
    total_added: u32
    total_used: u32
    by_source: BTreeMap<String,u32>   (serializes as JSON object {source: count})
    utilization_pct: f32              (used/added capped 1.0)
  consumed_by: HeldOutPanel

================================================================================
commands_d7.rs  (D-7 canary)
================================================================================
get_canary_recent  (commands_d7.rs:63)  [returns: Vec<CanaryRunRow>; sorted by ts_ms desc, empty if no viewer]
  element = CanaryRunRow (local struct @ commands_d7.rs:32)
    report_id: String
    ts_ms: i64
    overall_transfer_score: f64
    decision_kind: String
    dominant_fail_family: String
  consumed_by: CanaryPanel

get_canary_per_family_breakdown  (commands_d7.rs:100)  [returns: CanaryBreakdown; empty by_report if no viewer]
  DTO = CanaryBreakdown (local struct @ commands_d7.rs:54)
    by_report: BTreeMap<String, Vec<CanaryFamilyRow>>   (JSON object {report_id: [family rows]})
      CanaryFamilyRow @ commands_d7.rs:42: report_id:String, family:String, n_evaluated:u32, pass_count:u32, fail_count:u32, transfer_score:f64, baseline_score:f64
  consumed_by: CanaryPanel

================================================================================
commands_d9.rs  (D-9 locked keys / action_type lock)
================================================================================
get_locked_keys_status  (commands_d9.rs:86)  [returns: LockedKeysStatusDto; reads registry from disk]
  DTO = LockedKeysStatusDto (local struct @ commands_d9.rs:35)
    registry_path: String
    n_entries: u32
    registry_blake3: String
    loaded_ts_ms: i64
    entries: Vec<LockedKeysEntryDto>   (@:45 -> path:String, line_range:Vec<u32>, reason:String)
    error: Option<String>              (populated when registry missing/malformed; fail-soft)
  consumed_by: LockedKeysPanel

get_locked_keys_recent_audits  (commands_d9.rs:125)  [returns: LockedKeysRecentAudits]
  DTO = LockedKeysRecentAudits (local struct @ commands_d9.rs:74)
    runs: Vec<LockedKeysAuditRunDto>   (@:52 -> audit_id:String, ts_ms:i64, n_audited:u32, n_violations:u32, registry_blake3:String, registry_path:String)
    violations_by_audit: BTreeMap<String, Vec<LockedKeysViolationDto>>  (JSON object {audit_id: [violations]})
      LockedKeysViolationDto @:62 -> audit_id:String, ts_ms:i64, attempted_edit:String, attempted_lines:Vec<u32>, locked_key_path:String, reason:String
  consumed_by: LockedKeysPanel

================================================================================
commands_d10.rs  (D-10 claim routing)
================================================================================
get_recent_claim_routings  (commands_d10.rs:57)  [returns: RecentClaimRoutings; empty if no viewer]
  DTO = RecentClaimRoutings (local struct @ commands_d10.rs:42)
    rows: Vec<ClaimRoutedRowDto>       (@:30 -> claim_id:String, claim_type:String, mode:String, ts_ms:i64, ctx_summary_blake3:String)
    counts_by_mode: BTreeMap<String,u32>   (JSON object {mode: count})
    total: u32
  consumed_by: ModeRoutingPanel

================================================================================
commands_d11.rs  (D-11 checker registry)
================================================================================
get_checker_results_recent  (commands_d11.rs:124)  [returns: RecentCheckerResults; empty if no viewer]
  DTO = RecentCheckerResults (local struct @ commands_d11.rs:44)
    rows: Vec<CheckerResultRowDto>     (@:33 -> checker_id:String, checker_kind:String, result_kind:String, claim_id:String, ts_ms:i64, reason:String)
    counts_by_kind: BTreeMap<String,u32>   (JSON object)
    total: u32
  consumed_by: CheckerStatusPanel + CheckerRegistryPanel

get_checker_registry_state  (commands_d11.rs:179)  [returns: CheckerRegistryState; static kind table]
  DTO = CheckerRegistryState (local struct @ commands_d11.rs:72)
    kinds: Vec<CheckerKindEntry>       (@:60 -> kind:String, description:String, builtin_available:bool)
                                        (closed enum set: sympy/z3/compile_rust/regex_match/json_schema_validate/numeric_range/compile_py/registry_check)
  consumed_by: CheckerRegistryPanel

================================================================================
commands_d12.rs  (D-12 baby-step dispatch)
================================================================================
get_baby_step_dispatches_recent  (commands_d12.rs:66)  [returns: RecentBabyStepDispatches; empty if no viewer]
  DTO = RecentBabyStepDispatches (local struct @ commands_d12.rs:47)
    rows: Vec<BabyStepDispatchedRowDto>  (@:35 -> step_id:String, parent_task_id:String, checker_kinds_used:Vec<String>, outcome:String, ts_ms:i64)
    counts_by_outcome: BTreeMap<String,u32>  (pre-seeded pass/fail/inapplicable)
    counts_by_kind: BTreeMap<String,u32>
    total: u32
  consumed_by: BabyStepPanel

================================================================================
commands_d13.rs  (D-13 adversary)
================================================================================
get_adversarial_runs_recent  (commands_d13.rs:68)  [returns: RecentAdversarialRuns; empty if no viewer]
  DTO = RecentAdversarialRuns (local struct @ commands_d13.rs:54)
    rows: Vec<AdversaryRunRowDto>      (@:36 -> counter_id:String, claim_id:String, severity:f64, model_family:String, original_verdict:String, post_counter_verdict:String, ts_ms:i64)
    total: u32
    verdict_inversions: u32            (count where original_verdict != post_counter_verdict)
  consumed_by: AdversaryPanel

get_adversary_config  (commands_d13.rs:129)  [returns: AdversaryConfigDto]
  DTO = AdversaryConfigDto (local struct @ commands_d13.rs:114)
    adversary_model_family: String
    max_counter_attempts: u32
    severity_threshold: f64
  consumed_by: AdversaryPanel

================================================================================
commands_d15.rs  (D-15 novelty)
================================================================================
get_novelty_recent  (commands_d15.rs:72)  [returns: RecentNovelty; empty if no viewer]
  DTO = RecentNovelty (local struct @ commands_d15.rs:63)
    rows: Vec<NoveltyClassifiedRowDto>  (@:40 -> template_id:String, class:String, dkg_match_count:u32, near_duplicate_threshold:f64, ts_ms:i64, candidate_status:Option<String> [serde skip_if_none])
    distribution: NoveltyDistributionDto  (@:52 -> total:u32, search:u32, distant_search:u32, discovery_candidate:u32)
  consumed_by: NoveltyBreakdown + NoveltyPanel

get_novelty_breakdown  (commands_d15.rs:236)  [returns: NoveltyBreakdownDto]
  DTO = NoveltyBreakdownDto (local struct, commands_d15.rs; built by breakdown_from_distribution @:222)
    total: u32
    search: u32
    distant_search: u32
    discovery_candidate: u32
    search_pct: f64
    distant_search_pct: f64
    discovery_candidate_pct: f64
    dominant_class: Option<String>   (search|distant_search|discovery_candidate; None when total==0)
    limit_applied: u32
  consumed_by: NoveltyBreakdown

================================================================================
commands_d16.rs  (D-16 conformal refusal calibration)
================================================================================
get_conformal_recent  (commands_d16.rs:74)  [returns: RecentConformal; empty if no viewer]
  DTO = RecentConformal (local struct @ commands_d16.rs:63)
    tau_rows: Vec<ConformalTauRowDto>      (@:39 -> tau:f64, epsilon:f64, corpus_size:u32, source:String, ts_ms:i64)
    refusal_rows: Vec<ConformalRefusalRowDto>  (@:49 -> claim_id:String, claim_score:f64, tau:f64, decision:String, confidence:f64, ts_ms:i64)
    abstain_count: u32
    answer_count: u32
  consumed_by: ConformalCalibrationPanel

get_calibration_corpus_state  (commands_d16.rs:191)  [returns: CalibrationCorpusStateDto — UNRESOLVED: struct not read this session]
  consumed_by: ConformalCalibrationPanel
  NOTE: handler at :191 not transcribed; DTO CalibrationCorpusStateDto not located. UNRESOLVED-SHAPE.

================================================================================
commands_d17.rs  (D-17 honest refusal SLA)
================================================================================
get_refusal_sla_recent  (commands_d17.rs:70)  [returns: RecentRefusalSla; empty if no viewer]
  DTO = RecentRefusalSla (local struct @ commands_d17.rs:60)
    snapshot_rows: Vec<RefusalRateSnapshotRowDto>  (@:37 -> refusal_rate:f64, window_total:u32, abstain_count:u32, health_band:String, ts_ms:i64)
    breach_rows: Vec<RefusalSlaBreachRowDto>        (@:48 -> rate:f64, threshold:f64, breach_kind:String, ts_ms:i64)
    is_breaching: bool          (true when latest snapshot health_band != "healthy")
  consumed_by: RefusalRateGauge

get_refusal_rate_current  (commands_d17.rs:127)  [returns: UNRESOLVED — not read this session]
  consumed_by: RefusalRateGauge (secondary)
  NOTE: handler at :127 not transcribed. UNRESOLVED-SHAPE.

================================================================================
commands_d18.rs  (D-18 observable effect)
================================================================================
get_observable_effects_recent  (commands_d18.rs:139)  [returns: RecentObservableEffects; empty if no viewer]
  DTO = RecentObservableEffects (local struct @ commands_d18.rs:100)
    rows: Vec<ObservableEffectRowDto>  (@:38 -> effect_id:String, trigger_kind:String, observation_kind:String, sandbox_tier:String, outcome:String, confound_flags:String, elapsed_us:u64, ts_ms:i64)
    counts_by_outcome: BTreeMap<String,u32>      (pass|fail|skipped)
    counts_by_sandbox_tier: BTreeMap<String,u32> (no_sandbox|process_isolated|container_isolated|network_blocked)
    total: u32
    dispatched_rows: Vec<ObservableEffectDispatchedDto>  (#[serde(default)]; @:52 -> effect_id, trigger, context_kind, ts_ms)
    goodhart_flags: Vec<GoodhartFlagDto>                 (#[serde(default)]; @:62 -> effect_id, risk_kind, evidence, ts_ms)
    sandbox_observations: Vec<SandboxObservationDto>     (#[serde(default)]; @:72 -> effect_id, tier, outcome, evidence_count:u32, ts_ms)
    production_proposals: Vec<ProductionObservationProposalDto>  (#[serde(default)]; @:85 -> effect_id, instrument_file:Option<String>, requires_human_review:bool, ts_ms)
  consumed_by: ObservableEffectPanel

================================================================================
commands_d19.rs  (D-19 PRM bench) — BENCH-PENDING
================================================================================
get_prmbench_recent  (commands_d19.rs:48)  [returns: Vec<PrmBenchRow>; empty if no viewer]
  element = PrmBenchRow (local struct @ commands_d19.rs:32)
    seq: i64
    ts: i64
    total: u32
    accuracy: f64
    precision: f64
    recall: f64
    gating_verdict: String      (pass|fail)
    fixture_blake3: String      (64-hex of JSONL fixture)
    category_count: u32
  consumed_by: PrmBenchPanel

================================================================================
commands_d20.rs  (D-20 ARC bench) — BENCH-PENDING
================================================================================
get_arc_bench_recent  (commands_d20.rs:62)  [returns: RecentARCBenchRuns; empty if no viewer]
  DTO = RecentARCBenchRuns (local struct @ commands_d20.rs:45)
    rows: Vec<ARCBenchRunDto>   (@:31 -> total:u32, honestly_evaluated_count:u32, gating_verdict:String, fixture_blake3:String, dominant_mode:String, dominant_novelty_class:String, ts_ms:i64)
    total: u32
    last_verdict: String        (honestly_evaluated|game_with_memorization|refused; "" if no rows)
    last_dominant_mode: String
    last_dominant_novelty_class: String
  consumed_by: ARCBenchPanel

================================================================================
commands_d21.rs  (D-21 mode-routing bench) — BENCH-PENDING
================================================================================
get_mode_routing_bench_recent  (commands_d21.rs:45)  [returns: Vec<ModeRoutingBenchRow>; empty if no viewer]
  element = ModeRoutingBenchRow (local struct @ commands_d21.rs:30)
    seq: i64
    ts: i64
    total: u32
    accuracy: f64
    f1_macro: f64
    gating_verdict: String
    fixture_blake3: String
    fixture_row_count: u32
  consumed_by: ModeRoutingBenchPanel

================================================================================
commands_d22.rs  (D-22 conformal validation) — BENCH-PENDING
================================================================================
get_conformal_validation_recent  (commands_d22.rs:44)  [returns: Vec<ConformalValidationRow>; empty if no viewer]
  element = ConformalValidationRow (local struct @ commands_d22.rs:29)
    seq: i64
    ts: i64
    tau: f64
    epsilon: f64
    empirical_refusal_rate: f64
    gating_verdict: String
    fixture_blake3: String
    total: u32
  consumed_by: ConformalValidationPanel

================================================================================
commands_d23.rs  (D-23 sycophancy bench) — BENCH-PENDING
================================================================================
get_sycophancy_bench_recent  (commands_d23.rs:51)  [returns: Vec<SycophancyBenchRow>; empty if no viewer]
  element = SycophancyBenchRow (local struct @ commands_d23.rs:31)
    seq: i64
    ts: i64
    tp: u32
    fp: u32
    tn: u32
    fn_: u32                     (note: serde field name is "fn_")
    inconclusive: u32
    detection_rate: f64
    fp_rate: f64
    verdict: String              (pass|fail)
    total_fixtures: u32
    fixture_blake3: String
    runner_name: String
  consumed_by: SycophancyBenchPanel

================================================================================
commands_d26.rs  (D-26 archive lineage + combined view + verify)
================================================================================
get_archive_lineage  (commands_d26.rs:200)  [returns: ArchiveLineage; empty if id not found]
  DTO = ArchiveLineage (local struct @ commands_d26.rs:60)
    entries: Vec<LineageNode>    (@:47 -> template:ReasoningTemplateDto, self_hash:String, parent_hash:Option<String>, promoted_at_ms:Option<i64>, retired_at_ms:Option<i64>, retirement_reason:Option<String>, depth:usize)
    root_id: Option<String>
    depth: usize                 (parent-first: index 0 = root, last = queried node)
  consumed_by: LineageGraphView

get_template_archive_combined  (commands_d26.rs:213)  [returns: CombinedArchiveView]
  DTO = CombinedArchiveView (local struct @ commands_d26.rs:105)
    templates: Vec<TemplateSummary>      (@:69 -> id:String, domain:String, pattern_kind:String, archive_status:String, content_hash_hex:String, parent_hash_hex:Option<String>, created_at_ms:i64, body_len:usize)
    recent_tripwires: Vec<TripwireSummary>  (@:82 -> seq:i64, ts:i64, kind:String, tripwire_kind:Option<String>, candidate_id:Option<String>, observed_value:Option<f64>, threshold:Option<f64>)
    bai_arms: Vec<ArmSummary>            (@:94 -> template_id:String, alpha:f64, beta:f64, n_pulls:u32, mean_estimate:f64, was_promoted:bool)
  consumed_by: ArchiveExplorerView

get_archive_verify_status  (commands_d26.rs:459)  [returns: ArchiveVerifyStatus; default if no viewer; projected from archive_verify_result SSB row]
  DTO = ArchiveVerifyStatus (local struct; fields confirmed from projection @:446)
    verified: bool
    total_entries: <int>
    error_count: u32
    last_verified_ts_ms: Option<i64>
    errors: Vec<String>          (variant-name labels, e.g. "HashMismatch"; never the full error object)
  consumed_by: ArchiveVerifyGauge

================================================================================
commands_d27.rs  (D-27 mode-routing/refusal composite + discovery queue + sycophancy flags)
================================================================================
get_mode_routing_refusal_combined  (commands_d27.rs:171)  [returns: ModeRoutingRefusalView]
  DTO = ModeRoutingRefusalView (local struct @ commands_d27.rs:93)
    mode_breakdown: BTreeMap<String,u32>   (seeded 5 modes: constructive/falsifiability_cross_family/conformal_refusal/operational_deferred/conservative_fallback)
    refusal_rate_current: f64
    refusal_health_band: String            ("unknown" when no snapshot)
    discovery_queue: Vec<DiscoveryQueueRow>  (@:66 -> template_id:String, class:String, dkg_match_count:u32, near_duplicate_threshold:f64, ts_ms:i64, days_waiting:i64)
    recent_classifications: Vec<NoveltyClassifiedRowD27>  (@:83 -> template_id:String, class:String, dkg_match_count:u32, near_duplicate_threshold:f64, ts_ms:i64)
    dominant_mode: Option<String>
    limit_applied: u32
  consumed_by: DiscoveryQueueDrawer + ModeRoutingDashboardView

get_sycophancy_flags_recent  (commands_d27.rs:288)  [returns: Vec<SycophancyFlagRow>; empty if no viewer]
  element = SycophancyFlagRow (local struct @ commands_d27.rs:271)
    seq: i64
    ts_ms: i64
    claim_id: String
    flip_detected: bool
    delta: f64
    verdict: String              (convergent|divergent_sycophancy_flag|inconclusive)
  consumed_by: SycophancyFlagStrip

ratify_discovery  (commands_d27.rs:363)  [returns: () — write command, emits DiscoveryRatified SSB row]
  args: template_id:String, reviewer_id:String (non-empty required)
  consumed_by: DiscoveryQueueDrawer

reject_discovery  (commands_d27.rs:400)  [returns: () — write command, emits DiscoveryRejected SSB row]
  args: template_id:String, reviewer_id:String, reason:String (both non-empty required)
  consumed_by: DiscoveryQueueDrawer

================================================================================
commands_e1.rs  (E-1 memory notes + chat memory)
================================================================================
get_memory_notes  (commands_e1.rs:174)  [returns: MemoryNotesResponse; lazy-creates store]
  DTO = MemoryNotesResponse (local struct @ commands_e1.rs:96)
    store_path: String
    store_just_created: bool
    notes: Vec<MemoryNoteRow>    (@:53 -> note_id:String, scope:String[session|project|global], tags:Vec<String>, body:String, links:Vec<String>, link_count:u32, created_at_ms:u64, last_accessed_ms:u64, trust_score:f32)
  consumed_by: MemoryNotesPanel

get_chat_memory_state  (commands_e1.rs:225)  [returns: ChatMemoryState]
  DTO = ChatMemoryState (local struct @ commands_e1.rs:113)
    core_block_text: String
    core_block_bytes: u32
    last_reflection_ms: Option<u64>
    sleep_compute_active: bool   (hardcoded false currently — FsmSubstate not exposed via AppState)
    recent_notes_count: u32
  consumed_by: MemoryNotesPanel

================================================================================
commands_e5.rs  (E-5 memory facts / user memory)
================================================================================
get_memory_facts  (commands_e5.rs:138)  [returns: MemoryFactsResponse; lazy-creates store]
  DTO = MemoryFactsResponse (local struct @ commands_e5.rs:81)
    store_path: String
    store_just_created: bool
    facts: Vec<MemoryFactDto>
  consumed_by: MemoryFactsDrawer
  MemoryFactDto @ commands_e5.rs:34 (shared by get_memory_facts + get_memory_facts_with_history):
    fact_id: String
    user_id: String
    project_id: Option<String>
    scope: String
    fact_type: String
    key: String
    value: serde_json::Value     (arbitrary JSON)
    confidence: f32
    trust_score: f32
    trust_threshold: f32
    ingested_at_ms: u64
    valid_from_ms: u64
    valid_to_ms: Option<u64>
    decay_rate_per_day: f32
    last_observed_at_ms: u64
    evidence_pointers: Vec<String>
    superseded_by: Option<String>

get_trust_score  (commands_e5.rs:164)  [returns: TrustScoreResponse; not in dashboard Section-1 list, hover-card]
  DTO = TrustScoreResponse (local struct @ commands_e5.rs:97): fact_id:String, trust_score:f32, trust_threshold:f32, decay_rate_per_day:f32, last_observed_at_ms:u64, valid_to_ms:Option<u64>

get_semantic_contradictions  (commands_e5.rs:262)  [returns: SemanticContradictionsResponse; viewer_attached=false if no viewer]
  DTO = SemanticContradictionsResponse (local struct @ commands_e5.rs:216)
    viewer_attached: bool
    pairs: Vec<SemanticContradictionDto>  (@:201 -> fact_id_a:String, fact_id_b:String, conflict_kind:String, seq:i64, ts_ms:i64)
  consumed_by: MemoryFactsDrawer

memory_forget  (commands_e5.rs:350)  [returns: ForgetResponse; soft-close + emits MemoryGovernanceEvent::Forgotten]
  DTO = ForgetResponse (local struct @ commands_e5.rs:306): fact_id:String, soft_closed_at_ms:u64, ssb_seq:i64 (-1 if SSB writer unavailable)
  consumed_by: MemoryFactsDrawer

memory_pin  (commands_e5.rs:403)  [returns: PinResponse; emits Pinned/Unpinned beacon]
  DTO = PinResponse (local struct @ commands_e5.rs:315): fact_id:String, pinned:bool, ssb_seq:i64 (-1 if SSB writer unavailable)
  consumed_by: MemoryFactsDrawer

get_memory_facts_with_history  (commands_e5.rs:456)  [returns: MemoryFactsHistoryResponse]
  DTO = MemoryFactsHistoryResponse (local struct @ commands_e5.rs:324)
    store_path: String
    facts: Vec<MemoryFactDto>          (currently-valid)
    history_facts: Vec<MemoryFactDto>  (recently-superseded: superseded_by IS NOT NULL)
  consumed_by: MemoryFactsDrawer

================================================================================
commands_e6.rs  (E-6 MAST failure modes + iso-budget)
================================================================================
get_mast_breakdown  (commands_e6.rs:80)  [returns: MastBreakdownResponse; empty if no viewer]
  DTO = MastBreakdownResponse (local struct @ commands_e6.rs:61)
    rows: Vec<MastBreakdownRow>   (@:36 -> mast_mode:String, agent_role:String, count:u32)
    totals: Vec<MastModeTotal>    (@:50 -> mast_mode:String, total:u32)
    grand_total: u32
  consumed_by: MastFailurePanel + FailureModesDrawer

get_iso_budget_recent  (commands_e6.rs:206)  [returns: Vec<IsoBudgetMeasuredDto>; empty if no viewer]
  element = IsoBudgetMeasuredDto (local struct @ commands_e6.rs:177)
    task_fixture: String
    budget_tokens: u32
    full_pipeline_pass_rate: f64
    single_agent_wait_pass_rate: f64
    insufficient_lift: bool
    runpod_pending: bool
    mast_mode_counts: Vec<MastModeCountDto>   (@:198 -> mast_mode:String, count:u32)
    unique_catch_rate: BTreeMap<String,f64>        (#[serde(default)]; H-40)
    decorrelation_score: BTreeMap<String,f64>      (#[serde(default)]; H-40)
    insufficient_contribution: BTreeMap<String,bool>  (#[serde(default)]; per-role prunable, H-40)
  consumed_by: MastFailurePanel

================================================================================
commands_e10.rs  (E-10 critic decisions) — DARK(ELAI_CRITIC_OF_DESIGN_ENABLE)
================================================================================
get_critic_decision_state  (commands_e10.rs:61)  [returns: CriticDecisionState; empty if no viewer/flag off]
  DTO = CriticDecisionState (local struct @ commands_e10.rs:51)
    tiles: Vec<CriticTileState>   (@:30 -> phase:String, baseline_score:f64, challenger_score:f64, paired_bootstrap_p:Option<f64>, sample_size:u32, verdict_code:u8 [0=Keep,1=Simplify], ts_ms:i64)
    grand_total: u32
  consumed_by: CriticDecisionPanel

================================================================================
commands_e14.rs  (E-14 browser e2e / media validation)
================================================================================
browser_e2e_status  (commands_e14.rs:68)  [returns: BrowserE2eStatusDto; zeroed default if no viewer]
  DTO = BrowserE2eStatusDto (local struct @ commands_e14.rs:36)
    runs_started: u32
    assertions_passed: u32
    assertions_failed: u32
    snapshots_captured: u32
    last_journey_name_hash: Option<String>   (16-char hex blake3-prefix; None if no rows)
    last_ts_ms: Option<i64>
  consumed_by: MediaValidationPanel

================================================================================
commands_e18.rs  (E-18 tool reliability)
================================================================================
tool_reliability_status  (commands_e18.rs:88)  [returns: ToolReliabilityStatusDto; defaults+zeroes if not wired]
  DTO = ToolReliabilityStatusDto (local struct @ commands_e18.rs:54)
    inline_cap_bytes: usize
    default_retry_cap: u32
    spill_dir: String
    pass_count: u64
    reject_count: u64
    spill_count: u64
    escalate_count: u64
    spill_queue_bytes: u64
  consumed_by: ToolReliabilityInspector

list_tool_reliability_rows  (commands_e18.rs:151)  [returns: Vec<ToolReliabilityRowDto> (per contract sheet TSType)]
  element = ToolReliabilityRowDto (local struct @ commands_e18.rs:71)
    action_type: String
    tool_name_hash: String
    ts_ms: i64
    reject_reason_hash: Option<String>   (on tool_reliability_gate_rejected)
    spill_path_hash: Option<String>      (on tool_call_spilled)
    attempt: Option<u32>                 (on tool_call_retry_escalated)
    escalate_to_role_hash: Option<String> (on tool_call_retry_escalated)
    payload_bytes: Option<u64>           (on tool_call_spilled)
  consumed_by: ToolReliabilityInspector

================================================================================
commands_e29.rs  (E-29 paged transcript)
================================================================================
ssb_transcript_paged  (commands_e29.rs:257)  [returns: TranscriptPage; empty if no viewer]
  args: session_id:String, filter:Option<TranscriptFilter> (@:81 -> action_type_whitelist:Vec<String>, agent_role:Option<String>, limit:Option<usize>, before_seq:Option<i64>, after_seq:Option<i64>, total_count:Option<bool>, first_seq:Option<bool>)
  DTO = TranscriptPage (local struct @ commands_e29.rs:100)
    rows: Vec<TranscriptRow>     (@:59 -> seq:i64, ts_ms:i64, action_type:String, agent_id:String, role:String, body:String, redacted:bool [true when ELAI_OTEL_TRACE_CONTENT unset])
    total_count: Option<usize>   (None unless filter.total_count==true)
    first_seq: Option<i64>       (None unless filter.first_seq==true)
    has_older: bool
    has_newer: bool
  consumed_by: TranscriptDrawer
  (siblings in handler index, NOT in Section-1 panel list: ssb_transcript:170 -> Vec<TranscriptRow>; ssb_resume_from:388 -> ResumeResponse{replay_session_id,outcome,refusal_reason}; ssb_promote_to_dataset:442 -> PromoteResponse{dataset_name,promoted_count,skipped_duplicates}; ssb_backfill_signals:468 -> BackfillResponse{signal_label,candidate_count,sample_seq_ids:Vec<i64>})

================================================================================
commands_e30.rs  (E-30 incidents)
================================================================================
get_recent_incidents  (commands_e30.rs:80)  [returns: Vec<IncidentRow>; empty if no viewer; 8 action types]
  element = IncidentRow (local struct @ commands_e30.rs:51)
    seq: i64
    ts: i64
    kind: &'static str  (argv_destructive_denied|argv_approval_required|trust_dialog_hook_deferred|trust_dialog_env_deferred|mcp_pin_version_refused|auditor_lineage_collision|untrusted_memory_write_blocked|incident_record_created)
    argv_hash: Option<String>        (argv events)
    reason: Option<String>           (argv events)
    citation: Option<String>         (argv events)
    defer_reason: Option<String>     (trust dialog events)
    server_name: Option<String>      (mcp pin events)
    pinned_version: Option<String>   (mcp pin events)
    observed_version: Option<String> (mcp pin events)
    auditor_lineage: Option<String>  (lineage events)
    executor_lineage: Option<String> (lineage events)
    text_source: Option<String>      (memory events)
    incident_id: Option<String>      (postmortems)
    title: Option<String>            (postmortems)
    source_url: Option<String>       (postmortems)
  consumed_by: IncidentsDrawer

================================================================================
commands_e35.rs  (E-35 grammar compilations) — PRODUCER PENDING (placeholder json!)
================================================================================
get_grammar_compilations  (commands_e35.rs:39)  [returns: Value-json! — fixed placeholder until sidecar producer ships]
  rows: []                     (Array; empty until producer wired)
  producer_status: "pending_sidecar_wiring"
  expected_total: 64           (8 AgentRole * 8 BabyStepSpec blocks)
  note: <string>
  pin_status: "not_applicable" (I-34: cloud/sidecar-pending lane)
  (planned row shape per doc comment: schema_id, schema_hash, compile_status[compiled|cached|missing], last_used_ms, hit_count, pin_status[pass|violation|not_applicable])
  consumed_by: GrammarCompilePanel

================================================================================
commands_e37.rs  (E-37 safety / prompt armor)
================================================================================
safety_status  (commands_e37.rs:77)  [returns: SafetyStatusDto; zeroed if no viewer]
  DTO = SafetyStatusDto (local struct @ commands_e37.rs:36)
    injection_detected_count: u64
    injection_blocked_count: u64
    spotlight_wrapped_count: u64
    spotlight_breached_count: u64
    pin_mismatch_count: u64
    capability_rejected_count: u64
    provenance_tagged_count: u64
    prompt_armor_verdict_count: u64
    fixture_replayed_count: u64
    pin_dir: String
  consumed_by: PromptArmorPanel

list_safety_rows  (commands_e37.rs:119)  [returns: Vec<SafetyRowDto>]
  element = SafetyRowDto (local struct @ commands_e37.rs:54): action_type:String, ts_ms:i64  (aggregate-counts panel; per-row detail intentionally omitted)
  consumed_by: PromptArmorPanel

================================================================================
commands_e38.rs  (E-38 roadmap signals) — DARK(ELAI_AI_ROADMAP_SIGNALS_ENABLE)
================================================================================
get_roadmap_signals_state  (commands_e38.rs:87)  [returns: RoadmapSignalsState; empty if no viewer/flag off]
  DTO = RoadmapSignalsState (local struct @ commands_e38.rs:69)
    last_compliance: Option<ComplianceTile>   (@:30 -> kind:String[compliance_blocked|compliance_verified], assertion_label:String, hint:String, assertions_checked:u32, plan_session_id:String, ts_ms:i64)
    mcp_transport: Option<McpTransportTile>    (@:48 -> mcp_transport_current:String, mcp_transport_target:String, ts_ms:i64)
    posture_stale: Option<PostureStaleTile>    (@:59 -> posture_doc_path:String, posture_doc_mtime_ms:i64, age_days:u32, ts_ms:i64)
    mrcr_unverified_count: u32
    grand_total: u32
  consumed_by: RoadmapPanel

================================================================================
commands_e39.rs  (E-39 cost router / velocity cap / automix / free tier)
================================================================================
get_velocity_cap_status  (commands_e39.rs:175)  [returns: VelocityCapStatus; empty if no viewer]
  DTO = VelocityCapStatus (local struct @ commands_e39.rs:48)
    windows: Vec<VelocityWindowSnapshot>  (@:33 -> window:String[one_min|ten_min|one_hour|twenty_four_hour], count:u64, limit:u64, kind:String[requests|usd_cents], ts_ms:i64)
    grand_total: u32
  consumed_by: CostPanel

get_cost_router_recent  (commands_e39.rs:237)  [returns: Vec<CostRouterDecisionRow>; empty if no viewer; filters cost_token_event kind=cost_router_decision]
  element = CostRouterDecisionRow (local struct @ commands_e39.rs:55)
    session_id: Option<String>
    task_id: Option<String>
    difficulty: f64
    budget_remaining_usd: f64
    chosen_backend: String
    criticality: String          (default "medium")
    rationale: String
    ts_ms: i64
  consumed_by: CostPanel

get_automix_verdicts_recent  (commands_e39.rs:291)  [returns: Vec<AutomixVerdictRow>; filters kind=automix_verifier_verdict]
  element = AutomixVerdictRow (local struct @ commands_e39.rs:68): session_id:Option<String>, task_id:Option<String>, verdict:String, reason:String, ts_ms:i64
  consumed_by: CostPanel

get_free_tier_quotas  (commands_e39.rs:335)  [returns: Vec<FreeTierQuotaRow>; filters kind=free_tier_quota]
  element = FreeTierQuotaRow (local struct @ commands_e39.rs:78): backend:String, status:String, remaining_pct:u8, ts_ms:i64
  consumed_by: CostPanel

================================================================================
commands_e40.rs  (E-40 symbol graph)
================================================================================
symbol_graph_query  (commands_e40.rs:67)  [returns: SymbolGraphQueryReport]
  DTO = SymbolGraphQueryReport (local struct @ commands_e40.rs:33)
    kind: String                 (defs|refs|callers|callees)
    symbol: String
    result_count: u32
    results: Vec<SymbolMatch>    (1-hop type elai_orchestrator::symbol_graph::SymbolMatch — fields UNRESOLVED-SUBOBJECT)
    graph_path: String
    node_count: u32
  consumed_by: SymbolGraphWindow

symbol_graph_pagerank  (commands_e40.rs:104)  [returns: SymbolGraphPageRankReport]
  DTO = SymbolGraphPageRankReport (local struct @ commands_e40.rs:44)
    seed: String
    k: u32
    ranked: Vec<RankedSymbol>    (1-hop type elai_orchestrator::symbol_graph::RankedSymbol — fields UNRESOLVED-SUBOBJECT)
    graph_path: String
    node_count: u32
  consumed_by: SymbolGraphWindow

symbol_graph_status  (commands_e40.rs:127)  [returns: SymbolGraphStatusReport]
  DTO = SymbolGraphStatusReport (local struct @ commands_e40.rs:54): graph_path:String, graph_present:bool, node_count:u32, lru_capacity:u32
  consumed_by: SymbolGraphWindow

================================================================================
commands_e41.rs  (E-41 PR review)
================================================================================
pr_review_status  (commands_e41.rs:80)  [returns: PrReviewStatusDto]
  DTO = PrReviewStatusDto (local struct @ commands_e41.rs:69)
    recent_reviews: Vec<PrReviewRecentDto>   (@:29 -> pr_number_hash:u64, comment_kind_hash:u64, comment_kind_label:String, ts_ms:i64)
    last_watcher_tick: Option<PrCommentWatchedDto>     (@:40 -> pr_number_hash:u64, new_comments_count:u32, ts_ms:i64)
    last_slopsquat: Option<PrPackageVerifyResultDto>   (@:47 -> pr_number_hash:u64, slopsquat_count:u32, ts_ms:i64)
    last_semgrep: Option<PrSemgrepGateResultDto>       (@:54 -> pr_number_hash:u64, findings_count:u32, severity_max_hash:u64, ts_ms:i64)
    last_auto_merge_block: Option<PrAutoMergeBlockedDto>  (@:62 -> pr_number_hash:u64, block_reason_hash:u64, ts_ms:i64)
  consumed_by: GithubWindow (PR review section)

================================================================================
commands_e46.rs  (E-46 dual ledger / agentless / goal drift / entropy)
================================================================================
get_dual_ledger_state  (commands_e46.rs:99)  [returns: DualLedgerStateDto; empty if no viewer]
  args: filter:Option<String>
  DTO = DualLedgerStateDto (local struct @ commands_e46.rs:52)
    tasks: Vec<DualLedgerTaskDto>  (@:38 -> task_id:String, parent_id:String, completion_pct:f64, status:String, latest_blocker:String, last_update_ms:i64)
    n_complete: u32                (completion_pct >= 100)
    n_blocked: u32
  consumed_by: PlanningLedgerPanel

get_recent_agentless_shortcuts  (commands_e46.rs:166)  [returns: Vec<AgentlessShortcutRow>]
  element = AgentlessShortcutRow (local struct @ commands_e46.rs:62): task_id:String, reason:String, ts_ms:i64
  consumed_by: PlanningLedgerPanel

get_goal_drift_status  (commands_e46.rs:200)  [returns: GoalDriftStatusDto]
  DTO = GoalDriftStatusDto (local struct @ commands_e46.rs:80)
    firings: Vec<GoalDriftRow>     (@:70 -> initial_goal_blake3:String, drift:f64, threshold:f64, step:u32, ts_ms:i64)
    firings_by_canary: BTreeMap<String,u32>  (JSON object keyed by initial_goal_blake3)
    latest_drift: f64              (NaN replaced with 0.0)
    latest_threshold: f64
  consumed_by: PlanningLedgerPanel

get_entropy_history  (commands_e46.rs:374)  [returns: Vec<EntropyHistogramRow>; empty if no viewer]
  element = EntropyHistogramRow (local struct @ commands_e46.rs:350)
    parent_node_key: String
    entropy_ratio: f64             (H_children / H_parent, [0,1])
    h_parent: f64
    h_children: f64
    threshold: f64
    ts_ms: i64
  consumed_by: DecompositionEntropyHistogram

================================================================================
commands_e54.rs  (E-54 eval corpus / sycophancy matched pair / KOR)
================================================================================
get_eval_corpus_status  (commands_e54.rs:128)  [returns: EvalCorpusStatus; known=false if no rows]
  DTO = EvalCorpusStatus (local struct @ commands_e54.rs:46)
    known: bool                    (false when producer wire-dark / no rows at all)
    families: Vec<FamilyCount>     (@:35 -> family:String, shard_created_count:u32, rejected_count:u32)
    ngram_rejected_count: u32
    cosine_rejected_count: u32
    schema_violation_count: u32
    test_fails_reference_count: u32
    test_useless_count: u32
    generator_rotation_violation_count: u32
    perf_delta_count: u32
    ngram_n: u32                   (13-gram threshold echoed from crate const)
    cosine_threshold: f64
    grand_total: u32
    last_ts_ms: i64
  consumed_by: EvalCorpusPanel

get_sycophancy_matched_pair_results  (commands_e54.rs:218)  [returns: SycophancyMatchedPairResults; known=false if no rows]
  DTO = SycophancyMatchedPairResults (local struct @ commands_e54.rs:92)
    known: bool
    probes: Vec<MatchedPairProbeView>  (@:81 -> task_id:String, family:String, phase:String, gap_pp:Option<f64>, sandbagging_anomaly:bool, ts_ms:i64)
    framing_gap_count: u32
    sandbagging_anomaly_count: u32
    grand_total: u32
  consumed_by: SycophancyMatchedPairPanel

get_kor_orthogonality_status  (commands_e54.rs:294)  [returns: KorOrthogonalityStatus — NOT in Section-1 panel list]
  DTO = KorOrthogonalityStatus (local struct @ commands_e54.rs:112): known:bool, pairs:Vec<KorPairVerdict>(@:102 -> family_a:String, family_b:String, passed:bool, delta_pp:Option<f64>, ts_ms:i64), violated_count:u32, passed_count:u32, grand_total:u32, last_ts_ms:i64
  consumed_by: (handler index only; no Section-1 panel)

================================================================================
commands_c35.rs  (C-35 tool gate)
================================================================================
get_tool_gate_snapshot  (commands_c35.rs:102)  [returns: ToolGateSnapshotDto]
  args: fsm_state:Option<String> (default "planning")
  DTO = ToolGateSnapshotDto (local struct @ commands_c35.rs:36)
    fsm_state: String
    eager: Vec<String>           (tools rendered as full schemas)
    gated: Vec<String>           (tools rendered as 1-line summaries)
    denied: Vec<String>          (tools omitted)
    feature_enabled: bool        (built with mcp feature)
    note: String
  consumed_by: ToolGateInspector

list_tool_gate_rows  (commands_c35.rs:130)  [returns: Vec<ToolGateRowDto>]
  element = ToolGateRowDto (local struct @ commands_c35.rs:61)
    tool_name: String
    fsm_state: String
    prompt_tokens_saved: i64
    action_type: String          (tool_gated|tool_eager|tool_lazy_resolved|tool_denied|tool_filter_committed|untrusted_var_quarantined)
    ts_ms: i64
    commit_hash: Option<String>          (#[serde skip_if_none]; on tool_filter_committed)
    allowed_categories: Option<Vec<String>>  (on tool_filter_committed)
    denied_tool: Option<String>          (on untrusted_var_quarantined)
    taint_source: Option<String>         (on untrusted_var_quarantined)
  consumed_by: ToolGateInspector

================================================================================
commands_c40.rs  (C-40 GitHub PR)
================================================================================
github_pr_open  (commands_c40.rs:88)  [returns: PrSummary (elai_orchestrator::github_pr) — mutation]
  args (GithubPrOpenArgs @:55): repo:Option<String>, title:String, body:String(default), head:String, base:Option<String>, draft:bool(default)
  consumed_by: GithubWindow
  NOTE: PrSummary fields 1-hop in orchestrator::github_pr (not transcribed). UNRESOLVED-SUBOBJECT (at least number/url/title/state per doc).
github_pr_list  (commands_c40.rs:120)  [returns: Vec<PrSummary> — UNRESOLVED-SUBOBJECT]
github_pr_status  (commands_c40.rs:150)  [returns: PrStatus (orchestrator::github_pr) — UNRESOLVED-SUBOBJECT]
github_pr_close  (commands_c40.rs:178)  [returns: per orchestrator — mutation]
github_pr_get_active_url  (commands_c40.rs:204)  [returns: ActivePrUrlDto @:72 -> url:Option<String>, pr_number:Option<u32>, ts_ms:Option<i64>]
  consumed_by: GithubWindow (all via github.svelte store)

================================================================================
commands_c42.rs  (C-42 authz gateway)
================================================================================
gateway_authz_status  (commands_c42.rs:106)  [returns: AuthzStatusDto]
  DTO = AuthzStatusDto (local struct @ commands_c42.rs:41)
    policy_file: String
    default: String              (allow|deny|unavailable)
    roles: Vec<AuthzRoleDto>     (@:58 -> role:String, tools:Vec<String>, doc_owners:Vec<String>)
    feature_enabled: bool
    note: String
  consumed_by: AuthzPanel

gateway_authz_recent_denies  (commands_c42.rs:115)  [returns: Vec<AuthzDenyRowDto>]
  element = AuthzDenyRowDto (local struct @ commands_c42.rs:66): role_hash:u64, tool_hash:u64, deny_reason_label:String(authz|tool_gate), deny_reason_hash:u64, ts_ms:i64
  consumed_by: AuthzPanel

gateway_authz_run_fuzz  (commands_c42.rs:158)  [returns: AuthzFuzzReportDto]
  DTO = AuthzFuzzReportDto (local struct @ commands_c42.rs:79): passed:u32, failed:u32, failures:Vec<String>, budget:String, full_500k_status:String, paper:String
  consumed_by: AuthzPanel

================================================================================
commands_c43.rs  (C-43 scope freeze)
================================================================================
scope_freeze_status  (commands_c43.rs:84)  [returns: FreezeStatusDto; fail-soft (error set on missing/malformed manifest)]
  args: override_root:Option<String>
  DTO = FreezeStatusDto (local struct @ commands_c43.rs:29)
    clean: bool
    ratified_at_ms: u64          (0 when manifest fails to load)
    verified_at_ms: u64
    violation_count: u32
    violations: Vec<FreezeViolationDto>  (@:50 -> item_path:String, item_path_hash:u64, frozen_since_ms:u64, modified_at_ms:u64)
    error: Option<String>
  consumed_by: FreezeStatusBar

================================================================================
commands_c47.rs  (C-47 tool calibration)
================================================================================
tool_calibration_status  (commands_c47.rs:75)  [returns: Vec<ToolCalibrationStatusDto>; empty if no registry]
  element = ToolCalibrationStatusDto (local struct @ commands_c47.rs:40): tool_name:String, shape_count:u32, total_calls:u32, error_count:u32, last_ts_ms:i64
  consumed_by: CaseBaseInspector

list_tool_calibration_rows  (commands_c47.rs:112)  [returns: Vec<ToolCalibrationRowDto>]
  element = ToolCalibrationRowDto (local struct @ commands_c47.rs:54)
    tool_name_hash: String
    call_shape_hash: String
    action_type: String          (tool_call_logged|tool_calibration_hint_emitted|tool_calibration_drift_detected)
    ts_ms: i64
    post_call_audit_ok: Option<bool>  (on tool_call_logged)
    error_rate: Option<f32>           (on tool_calibration_drift_detected)
    total_calls: Option<u32>          (on tool_call_logged + drift_detected)
  consumed_by: CaseBaseInspector

================================================================================
commands_f2.rs  (F-2 security blocks + insecure rate)
================================================================================
get_security_blocks_recent  (commands_f2.rs:79)  [returns: SecurityBlocksResponse; empty if no viewer]
  DTO = SecurityBlocksResponse (local struct @ commands_f2.rs:124)
    rows: Vec<SecurityRuleRow>   (@:48 -> seq:i64, ts:i64, rule_id:String, cwe:String, file_path:String, language:String, candidate_blake3:String)
    env_active: bool             (false only when ELAI_VIBEGATE_DISABLED kill-switch set)
  consumed_by: SecurityRulesPanel

get_insecure_rate_latest  (commands_f2.rs:171)  [returns: Option<InsecureRateRow>; null if no viewer/no row]
  DTO = InsecureRateRow (local struct @ commands_f2.rs:146): suite:String, model_id:String, n_samples:u32, insecure_rate_before:f64, insecure_rate_after:f64, gate_recall:f64, ts:i64
  consumed_by: SecurityRulesPanel

================================================================================
commands_f5.rs  (F-5 staleness quarantine)
================================================================================
get_staleness_quarantine  (commands_f5.rs:84)  [returns: StalenessQuarantineResponse; empty if no viewer]
  DTO = StalenessQuarantineResponse (local struct @ commands_f5.rs:67)
    rows: Vec<StalenessQuarantineRow>  (@:47 -> seq:i64, ts:i64, fact_id:String, staleness_ratio:f64, dependents_demoted:u32, cosmos_artifacts_flagged:u32, dropped_count:u32)
  consumed_by: StalenessQuarantineDrawer

================================================================================
commands_f7.rs  (F-7 template ranking)
================================================================================
get_template_ranking  (commands_f7.rs:89)  [returns: TemplateRankingResponse; empty if no viewer]
  DTO = TemplateRankingResponse (local struct @ commands_f7.rs:73)
    rows: Vec<TemplateRankingRow>  (@:47 -> seq:i64, ts:i64, concept_class:String, plan_skeleton_hash:u64, utility:f32 [Laplace-smoothed], first_try_ratio:f32, sample_n:u32, chosen_pipeline_id:String)
  consumed_by: TemplateRankingDrawer

================================================================================
commands_f10.rs  (F-10 security monotonicity)
================================================================================
get_security_monotonicity_blocks  (commands_f10.rs:102)  [returns: SecurityMonotonicityResponse; empty if no viewer]
  DTO = SecurityMonotonicityResponse (local struct @ commands_f10.rs:73)
    rows: Vec<SecurityMonotonicityRow>  (@:49 -> seq:i64, ts:i64, loop_kind:String[refine|auto_advance|self_mod], loop_id:String, introduced_classes:Vec<String>, prior_classes:Vec<String>, delta_blake3:String, lang:String)
    env_active: bool             (false only when ELAI_SECURITY_MONOTONICITY_DISABLE set)
  consumed_by: SecurityMonotonicityDrawer

================================================================================
commands_f13.rs  (F-13 session archive)
================================================================================
get_session_archives  (commands_f13.rs:52)  [returns: Vec<SessionArchiveRow> -> json array; empty [] if no viewer]
  element = SessionArchiveRow @ ssb_viewer.rs:5366
    seq: i64
    ts_ms: i64
    session_id: String
    stale_version: i64
    archive_path: String
    manifest_blake3: String
    store_kind: String           (ssb | session_dag)
  consumed_by: SessionArchiveDrawer

================================================================================
commands_h11.rs  (H-11 filesystem mount + canvas activity)
================================================================================
read_dir  (commands_h11.rs:159)  [returns: Vec<FsEntry>; sorted dirs-first; scope-checked]
  args: root:String, rel_path:String
  element = FsEntry (local struct @ commands_h11.rs:42): name:String, path:String, kind:String(file|dir), size:Option<u64>
  consumed_by: FileExplorer (via fileTree store)

read_file  (commands_h11.rs:223)  [returns: String — file contents]
open_in_editor  (commands_h11.rs:242)  [returns: () — mutation/side-effect]
mount_root  (commands_h11.rs:282)  [returns: per handler — registers allowed root]
get_canvas_activity  (commands_h11.rs:311)  [returns: Vec<CanvasActivityEventDto>]
  element = CanvasActivityEventDto (local struct @ commands_h11.rs:56): kind:String, target_node_id:String, file_path:String, symbol_name:Option<String>, mode:String, label:Option<String>, task_id:String, ts:i64
  consumed_by: canvas activity store

================================================================================
commands_h13.rs  (H-13 canvas ops)
================================================================================
get_canvas_ops  (commands_h13.rs:66)  [returns: Vec<CanvasOpDto>; empty if no viewer]
  element = CanvasOpDto (local struct @ commands_h13.rs:39): kind:String(node_created|edge_connected|...), node_id:String, op_data:serde_json::Value, mode:String, task_id:String, session_id:String, ts:i64
  consumed_by: canvasDispatcher store
  (sibling capture_canvas_screenshot:118 — not in Section-1 list)

================================================================================
commands_h14.rs  (H-14 canvas recast / oid)
================================================================================
resolve_oid  (commands_h14.rs:109)  [returns: OidResolveResponse; Err if oid not registered]
  args: oid:String
  DTO = OidResolveResponse (local struct @ commands_h14.rs:58)
    oid: String
    file: String
    range: [u32; 4]              (JSON array [start_line, start_col, end_line, end_col], 1-based)
    scope: String
  consumed_by: DesignPageNode
  (siblings apply_recast_patch:145, start_dev_proxy:182, stop_dev_proxy:208 — not in Section-1 list; apply_recast_patch arg = RecastPatchRequest{oid,attr_name,new_value,kind})

================================================================================
commands_h15.rs  (H-15 canvas views)
================================================================================
get_canvas_views  (commands_h15.rs:115)  [returns: Vec<CanvasViewEventDto>; empty if no viewer]
  element = CanvasViewEventDto (local struct @ commands_h15.rs:58)
    kind: String                 (view_spawned|ui_action|state_checkpoint)
    view_type: String            (ViewSpawned only)
    view_id: String
    seed_op_ids: Vec<String>     (ViewSpawned only)
    node_id: String              (UiAction only)
    taint_flagged: bool
    task_id: String
    session_id: String
    seq: i64
    ts: i64
  consumed_by: canvasViewState store

canvas_select_node  (commands_h15.rs:167)  [returns: CanvasSelectNodeResponse]
  DTO = CanvasSelectNodeResponse (local struct @ commands_h15.rs:84)
    node_id: String
    content: String              (frontend must check taint_flagged before use)
    taint_flagged: bool
    taint_source: String         ("" if clean)
    allowed: bool                (false = gate BLOCKED handoff; do not pass content to model)
    block_reason: String         ("" if allowed)
  consumed_by: canvasViewState store

emit_canvas_checkpoint  (commands_h15.rs:220)  [returns: per handler — write/checkpoint]
  consumed_by: canvasViewState store

================================================================================
commands_h41.rs  (H-41 checker coverage)
================================================================================
get_checker_gap_backlog  (commands_h41.rs:65)  [returns: Vec<CheckerGapBacklogRowDto>; empty if no viewer]
  element = CheckerGapBacklogRowDto (local struct @ commands_h41.rs:34): claim_type:String, complexity_taxon:String, count:u32, priority:u8 (1=normal,2=HIGH)
  consumed_by: CheckerCoveragePanel

get_coverage_report  (commands_h41.rs:92)  [returns: CoverageReportDto; zeroed if no rows]
  DTO = CoverageReportDto (local struct @ commands_h41.rs:44)
    mode_a_fraction: f64
    constructive_count: u64
    total_factual_computational: u64
    session_id: String
  consumed_by: CheckerCoveragePanel

================================================================================
commands_h49.rs  (H-49 time travel) — gated ELAI_TIMETRAVEL_DISABLE
================================================================================
timetravel_undo  (commands_h49.rs:271)  [returns: TimeTravelResponse]   args: session_id:String, scope:Option<String>(default "code")
timetravel_redo  (commands_h49.rs:331)  [returns: TimeTravelResponse]   args: session_id, scope
timetravel_restore_files_only  (commands_h49.rs:498)  [returns: TimeTravelResponse]  args: target:String
  DTO = TimeTravelResponse (local struct @ commands_h49.rs:43, shared by undo/redo/jump/restore)
    disabled: bool               (true when kill-switch suppressed)
    no_op: bool                  (HEAD already at target)
    beacon_seq: i64              (0 = none)
    head: Option<String>         (current HEAD snap_id after op)
    summary: String
    divergence_detected: bool    (RV-1 fork-captured)
  consumed_by: CommandWindow
  (siblings timetravel_jump:387 -> TimeTravelResponse; timetravel_history:449 -> Vec<TimeTravelNode>{snap_id,parent_id,is_head,children_count,tag,created_at_ms,manifest_hash})

================================================================================
commands_i33.rs  (I-33 governance metrics)
================================================================================
get_governance_metrics  (commands_i33.rs:56)  [returns: Option<GovernanceMetricsPayload>; null if no viewer/rows]
  DTO = GovernanceMetricsPayload (local struct @ commands_i33.rs:28)
    cdl: f64                     (Cosmetic-Deadlock rate = CeflExternalized/total)
    diu: f64                     (Deferral Info Utilisation = I6qRejected/total)
    fsr: f64                     (Framing Success Rate)
    row_count: usize
  consumed_by: (I-33 governance tile — Section 4 planned; handler SHIPPED)

================================================================================
commands_i38.rs  (I-38 QDELTA) — BENCH-PENDING
================================================================================
get_qdelta_latest  (commands_i38.rs:81)  [returns: Option<QdeltaLatest>; null if no viewer/no row]
  DTO = QdeltaLatest (local struct @ commands_i38.rs:35)
    suite: String
    model_id: String
    n_tasks: u32
    seed_count: u32
    paired: bool
    on_defective_rate: f64
    off_defective_rate: f64
    delta: f64                   (off - on; positive = gates help)
    ci_low: f64
    ci_high: f64
    dollar_per_task: f64
    contamination_probe_result: String  (not_run|honestly_evaluated|game_with_memorization)
    surviving_mutants_per_task: f64      (I-20 stub; 0.0 until ships)
    corpus_blake3: String
    runpod_pending: bool
    ts: i64
  consumed_by: QdeltaPanel

================================================================================
commands_vibegate.rs  (vibegate blocks)
================================================================================
list_vibegate_blocks  (commands_vibegate.rs:83)  [returns: VibeGateBlocksResponse; empty rows if no viewer; 4 kind arms]
  DTO = VibeGateBlocksResponse (local struct @ commands_vibegate.rs:157)
    rows: Vec<VibeGateRow>       (@:49 -> seq:i64, ts:i64, kind:String[CodeVuln|Secret|DepHallucinated|Skill], rule_id:Option<String>, cwe:Option<String>, file_path:String, source_blake3:String, citation:Option<String>, fsm_state:Option<String>)
    env_active: bool             (false only when ELAI_VIBEGATE_DISABLED set)
  consumed_by: VibeGateDrawer

================================================================================
commands_empire.rs  (empire bridge)
================================================================================
get_empire_status  (commands_empire.rs:79)  [returns: EmpireStatusDto]
  DTO = EmpireStatusDto (local struct @ commands_empire.rs:39)
    state: String                (unavailable|disconnected|connecting|connected)
    feature_enabled: bool        (built with empire feature)
    factory_line_count: u32
    note: String
  consumed_by: EmpireBridgePanel

list_empire_factory_lines  (commands_empire.rs:92)  [returns: Vec<FactoryLineDto>; empty when unavailable/disconnected]
  element = FactoryLineDto (local struct @ commands_empire.rs:55): brain_id:String, engine_tier:String(MKI|MKII|MKIII), neuron_count:u32, last_heartbeat_ms:i64, status:String
  consumed_by: EmpireBridgePanel

================================================================================
media_validation.rs  (media validation entries)
================================================================================
get_media_validations  (media_validation.rs:419)  [returns: Vec<MediaValidationEntry>; sorted updated_at desc]
  element = MediaValidationEntry (struct @ state.rs:138)
    step_id: String
    request_id: String           (mirrors step_id when no explicit uuid)
    media_type: String
    file_path: String
    tier1: Option<Tier1Snapshot>          (#[serde(default)])
    tier2: Option<Tier2Snapshot>          (#[serde(default)])
    render_frames: Vec<CapturedFrameSnapshot>  (#[serde(default)])
    feedback_iterations: Vec<IterationSnapshot> (#[serde(default)])
    final_verdict: Option<String>  (Pass|Fail|HumanReview|Degraded)
    updated_at: u64              (ms)
    axtree: Option<AxtreeSnapshot>         (#[serde(default)]; E-16)
    sycophancy: Option<SycophancySnapshot> (#[serde(default)]; E-16: flip_detected:bool, delta:f64, verdict:String)
    swap_order_disagreement: Option<bool>  (#[serde(default)]; E-16)
    convergence_history: Vec<f64>          (#[serde(default)]; E-16 LPIPS-delta history)
  consumed_by: MediaValidationPanel + commands store
  NOTE: Tier1Snapshot/Tier2Snapshot/CapturedFrameSnapshot/IterationSnapshot/AxtreeSnapshot are nested structs in state.rs (1-hop; not fully transcribed). UNRESOLVED-SUBOBJECT.

get_media_validation_by_step  (media_validation.rs:431)  [returns: Option<MediaValidationEntry>; null if step unseen]
  args: step_id:String
  consumed_by: commands store

submit_media_validation_for_review  (media_validation.rs:448)  [returns: () — local snapshot patch; Err if step unknown]
  args: step_id:String, decision:ReviewDecision (PascalCase enum: Pass|Fail|RequestRevalidate)
  consumed_by: commands store

subscribe_media_ssb_events  (media_validation.rs:476)  [returns: () — confirms subscriber alive; Err if no SSB DB]
  consumed_by: media_validation store

================================================================================
commands.rs  (the core ~280-fn file; Section-1-consumed commands only)
================================================================================
get_fsm_state  (commands.rs:296)  [returns: Value-json!]
  state: String                 (v2 label via fsm_state_to_v2_label, e.g. EXECUTING; "IDLE" default)
  current_step_id: ""           (always empty string literal)
  progress: 0.0                 (always 0.0 literal)
  history: array of {from:String, to:String, ts:<int>, trigger:"phase_transition"}  (last 10 phase transitions)
  agent_substate: String|null   ("Framing" when IdeaFramingStarted without IdeaFramed; else null)
  _phase: String                (s.phase)
  consumed_by: fsm.svelte store (-> ActivityWindow, MonitorWindow)

get_plan_progress  (commands.rs:400)  [returns: Value-json!; phases:null if no viewer]
  phases: array of {num:int(1-5), label:String(Plan|Feasibility|Execute|Audit|Memorize), steps:u32, status:String(done|active|pending)} | null
  current_phase: int(1-5) | null
  total_pct: f64                (0.0..1.0)
  consumed_by: plan.svelte store

get_system_status  (commands.rs:465)  [returns: Value-json!; composite of other commands]
  fsm: <get_fsm_state shape>
  sidecar: {loaded:bool, model:String, fa2:bool, dola:bool, steering:bool, turboquant:String, tokens_per_sec:f64, latency_p50_ms:f64, latency_p99_ms:f64, context_window:int, mcp_bridge_active:bool}
  vram: array (get_vram)
  gate: <get_gate_status shape>
  active_agents: array (get_agents)
  session_id: String           (format "dash-v2-{pid}")
  uptime_ms: <int>
  consumed_by: (system status surface; not a distinct Section-1 panel)

get_gate_status  (commands.rs:925)  [returns: Value-json!]
  mode: String                 (AUTONOMOUS | INTERACTIVE)
  auto_approve_threshold: 0.85 (literal)
  pending_count: u32           (pending interventions)
  auto_advanced_count: 0       (literal)
  consumed_by: session.svelte store (SettingsWindow, CockpitWindow)

get_feature_flags  (commands.rs:1031)  [returns: Value-json!]
  autonomous_mode: bool        (cfg! feature)
  self_modification: bool      (cfg! feature)
  dashboard_offline: bool      (ELAI_DASHBOARD_OFFLINE env)
  consumed_by: feature_flags.svelte store

set_model_routing  (commands.rs:1049)  [returns: Value-json! — the full routing array after upsert]
  array of {role:String, target:String}
  consumed_by: SettingsWindow (via stub)

get_query_trace_recent  (commands.rs:1409)  [returns: Value-json! array; [] if no viewer]
  element: {seq:i64, ts:i64, query_id:Value(str|null), tier:Value(str|null), stage_count:u64, result_count:u64, total_latency_ms:u64}
  consumed_by: QueryDrawer

get_pagerank_top  (commands.rs:1728)  [returns: Value-json!; {known:false, rows:[]} if no viewer]
  known: bool                  (false on viewer absent or query error)
  rows: array of {symbol:String, path:String, score:f64}
  consumed_by: CanvasWindow + PagerankPanel

get_graph_view_data  (commands.rs:2636)  [returns: VizData (elai_knowledge::visualization::types::VizData) — typed]
  args: project_id:Option<String>, mode:String
  shape: VizData (1-hop in knowledge crate; nodes/edges/layout — NOT transcribed here, hand-authored TS mirror is api/types.ts VizData)
  consumed_by: GraphWindow + CanvasWindow + canvasActivity store
  NOTE: VizData fields UNRESOLVED-SUBOBJECT (knowledge crate type; TS VizData in api/types.ts is the authored mirror).

get_impact_overlay  (commands.rs:2687)  [returns: Value-json!; empty shape if no DKG]
  files: array               (resolved_files)
  by_depth: object {depth: [files]}
  total_files: <int>
  max_depth_reached: <int>
  truncated: bool
  consumed_by: GraphWindow

export_graph_mermaid  (commands.rs:2753)  [returns: String — mermaid flowchart text; always non-empty]
  (ArchMode.svelte asserts TS type {mermaid:string} but handler returns a bare String; GraphWindow asserts string)
  consumed_by: GraphWindow + ArchMode

get_decomposition_tree  (commands.rs:3768)  [returns: Value-json!; {} if no viewer; serde of DecompositionTreeSnapshot]
  shape: serde_json of DecompositionTreeSnapshot @ ssb_viewer.rs:248 (top-level "concepts" array; nested phases/steps/babysteps)
  consumed_by: plan.svelte store
  NOTE: DecompositionTreeSnapshot full field set UNRESOLVED-SUBOBJECT (ssb_viewer.rs:248; has at least "concepts":[...]).

get_idea_framing_state  (commands.rs:8552)  [returns: Option<IdeaFramingStateDto>; null if no viewer/no framing]
  DTO = IdeaFramingStateDto (local struct @ commands.rs:8525)
    session_id: String
    framed_idea: Option<serde_json::Value>   (arbitrary JSON framed-idea blob)
    critic_rounds_used: u32
    is_converged: bool
    is_active: bool
  consumed_by: IdeaFramingWindow

get_stagnation_events  (commands.rs:8657)  [returns: Vec<StagnationEventDto>; empty if no viewer; flattens 8 SSB variants]
  element = StagnationEventDto (local struct @ commands.rs:8634)
    timestamp_ms: i64
    action_type: String
    agent_id: String
    pattern: Option<String>      (spinning|oscillation|no_drift|diminishing_returns)
    persona: Option<String>      (lateral-thinking events)
    drift_score: Option<f32>     (goal-drift events; Jaccard)
    elapsed_ms: Option<u64>      (watchdog timeout)
    detail: String
  consumed_by: OrchestrationDrawer

get_context_state  (commands.rs:8798)  [returns: ContextStateDto]
  DTO = ContextStateDto (local struct @ commands.rs:8750)
    fsm_state: String
    substate: Option<String>
    token_budget_pct: f32
    mcp_think_topics: Vec<TopicBudgetDto>   (@:8743 -> topic:String, remaining_calls:u32, remaining_chars:u32)
    last_precompact_thresholds: Vec<f32>
    prompt_injection_check_status: String   (passed|blocked|not_run)
    runtime_choice_badge: String            (local|cloud_billed|"")
    current_role: Option<String>            (E-3 agent role)
    current_runtime: Option<String>         (E-3 backend name)
  consumed_by: ContextStatePanel

get_trust_telemetry  (commands.rs:8981)  [returns: TrustReportDto; zeroed if gate off]
  DTO = TrustReportDto (local struct @ commands.rs:8774)
    tasks_per_hour: f64
    findings_per_task: f64
    revert_rate: f64
    ungrounded_count: u64
    stagnation_dist: HashMap<String,u64>   (JSON object {pattern: count})
  consumed_by: TrustTelemetryPanel

list_artifacts  (commands.rs:9232)  [returns: Value-json! — {artifact_names:[String]} per ArtifactsPanel TSType]
  args: db_path:String
  consumed_by: ArtifactsPanel
  (siblings search_artifacts:9264, index_artifacts:9334, remove_artifacts:9366 — Value shapes not transcribed)

list_entry_points  (commands.rs:9396)  [returns: Value-json! — {entry_points:[BackendEntryPoint], summary} per entrypoints_map.ts]
  args: include_tests:Option<bool>
  consumed_by: canvas/entrypoints_map.ts
  NOTE: BackendEntryPoint element shape UNRESOLVED-SUBOBJECT (built from SymbolGraphCache; not read this session).

get_ambiguity_state  (commands.rs:9529)  [returns: Value-json!]
  score: f32
  milestone: String            ("initial" default)
  threshold: f32               (AMBIGUITY_THRESHOLD const)
  brownfield: bool
  components: object {
    goal_clarity: {score:f32, weight:f32, floor:f32}
    constraint_clarity: {score:f32, weight:f32, floor:f32}
    success_criteria_clarity: {score:f32, weight:f32, floor:f32}
    context_clarity: {score:f32, weight:0.15, floor:0.60} | null
  }
  floor_failures: [String]     (failed dimension names)
  streak: {current:u32, required:u32}
  qualifies_for_advance: bool
  consumed_by: GatesPanel + OrchestrationDrawer

get_per_level_verdicts  (commands.rs:9978)  [returns: Vec<LevelAuditVerdictJson>; empty if no viewer]
  args: task_id:String ("active" default)
  element = LevelAuditVerdictJson (local struct @ commands.rs:9851)
    mode: String
    tier_node_id: TierNodeIdJson (@:9861 -> tier:String, id:u64)
    findings: Vec<AuditFindingJson> (@:9867 -> category:String, severity:String, message:String, source:Option<String>[skip_if_none]) — typically empty [] (findings persisted to disk, not SSB; cold-start path)
    verdict: String
    respawned_once: bool
    cascade_to_parent: bool
  consumed_by: AuditTimelineWindow

accept_change  (commands.rs:10740)  [returns: Value-json! {ok:true, seq} — emits ChangeDecision "accepted"]
  args: path:String, hunk_id:Option<String>
decline_change  (commands.rs:10752)  [returns: {ok,seq} — emits "declined"]
retry_audit  (commands.rs:10764)  [returns: {ok,seq} — emits retry verdict; arg note:Option<String>]
  consumed_by: ReviewWindow

get_research_node_history  (commands.rs:10703)  [returns: Value-json! — serde of Vec of research_node_stages rows; [] if no viewer/non-numeric node_id]
  args: _session_id:String, node_id:String (parsed as u64)
  consumed_by: ResearchDrawer
  NOTE: element shape = viewer.research_node_stages row type (1-hop orchestrator). UNRESOLVED-SUBOBJECT.

get_optimizer_run  (commands.rs:11425)  [returns: Value-json!; empty envelope if no viewer]
  rows: array of {seq:i64, ts:i64, phase:String, iteration_index:u64, val_lift_pct:f64|null, held_out_lift_pct:f64|null}
  latest_phase: String|null
  latest_val_lift_pct: f64|null
  latest_held_out_lift_pct: f64|null
  overfit_risk: bool           ((latest_val - latest_held_out) > 2.0)
  reflection_lm_too_small: bool
  consumed_by: OptimizerRunPanel

get_lora_ship_gate_status  (commands.rs:11527)  [returns: Value-json!; wire_dark envelope if no viewer]
  gate_status: String          (wire_dark | the latest row "kind" e.g. refused)
  gate_enabled: bool
  ship_gate_token: String      (LORA_SHIP_GATE_DEFERRAL_TOKEN default)
  confirm_user_understanding: bool
  rows: array                  (per-row gate events; element built from LoraShipGateEvent payload — fields beyond loop not transcribed)
  consumed_by: LoraShipGatePanel

credit_attribution_status  (commands.rs:11090)  [returns: Value-json! — synthetic until runpod pilot]
  rows: array of {role:String, score:f32}
  ts_ms: u64
  runpod_pending: true         (literal; synthetic harness)
  consumed_by: OrchestrationDrawer

routing_decisions_status  (commands.rs:11169)  [returns: Value-json! — synthetic until runpod pilot]
  rows: array of {task_id:String, difficulty:f64, alpha:f64, dispatched:[{role:String, primitive:String}], is_lazy_mode:bool}
  (+ runpod_pending flag in tail, not shown)
  consumed_by: OrchestrationDrawer

get_mcp_security_status  (commands.rs:261)  [returns: Vec or Value per McpStatusPanel TSType McpSecurityStatus[] — NOT read this session]
  consumed_by: McpStatusPanel
  NOTE: handler body at :261 not transcribed. UNRESOLVED-SHAPE.

get_pending_proposals  (commands.rs:1144)  [returns: Value-json! array of SelfModProposalRow; [] if no viewer / self-modification feature off]
  element = SelfModProposalRow @ ssb_viewer.rs:5325: proposal_id:String, table:String, fpr_estimate:Option<f32>, occurrences_seen:Option<u32>, pipeline_mode:String, applied_by:Option<String>, ts_ms:i64
  consumed_by: ProposalsPanel
get_applied_proposals  (commands.rs:1162)  [returns: Value-json! array of SelfModProposalRow — same element shape]
  consumed_by: ProposalsPanel
  (apply_proposal_from_user:1274, reject_proposal:1302 — mutation commands)

get_msi_status  (commands.rs:1315)  [returns: Value-json! array of SelfModMsiSnapshotRow; [] if no viewer]
  element = SelfModMsiSnapshotRow @ ssb_viewer.rs:5698: pipeline_mode:String, msi_value:f32, status:String, ts_ms:i64
  consumed_by: MsiPanel

get_steering_status  (commands.rs:1324)  [returns: Value-json! — fixed placeholder literal]
  active:false, vector_label:"freshness:modern_api_preference", last_computed_ts:0, magnitude:0.0, applied_count:0
  consumed_by: MsiPanel
  (recompute_steering_vector:1335 — mutation)

get_grc_budgets  (commands.rs:2817)  [returns: Value-json! array; spec-default 8-role array if empty]
  element: {role:String, calls_remaining:u32, chars_remaining:u32, total_calls:u32, total_chars:u32, calls_used:u32, chars_used:u32}
  consumed_by: grc.svelte store (GRC surfaces in MonitorWindow)

get_indexing_progress  (commands.rs:6540)  [returns: Value-json!; "unknown"/zeroed if no metadata DB]
  status: String               (indexing_status sql_str | "unknown")
  project_id: String
  project_path: String         (present when meta loaded)
  indexed_files: u64
  total_files: u64
  batch_size: <int>            (present when meta loaded)
  batches_processed: u32
  batches_total: int | null
  last_checkpoint_at: i64
  schema_version: <int>        (present when meta loaded)
  consumed_by: IndexingProgressPanel

get_indexing_resilience  (commands.rs:6627)  [returns: Value-json!]
  auto_resume_enabled: bool
  watch_locks_held: u32
  last_checkpoint_unix_secs: i64
  last_resume_from_batch: int | null
  last_resume_files_skipped: int | null
  lock_contested_count: u32
  shutdown_deadline_secs: u64
  last_batch_duration_ms: u64 | null
  consumed_by: IndexingProgressPanel


================================================================================
commands.rs — REMAINING SECTION-1 COMMANDS (shape NOT individually transcribed; see notes)
================================================================================
These Section-1-consumed commands.rs handlers were not opened individually this session (budget).
Most are settings setters/ack-mutations (return () or a small ack) or return CmdResult<Value> whose
TS type is asserted inline at the call site (api/stub.ts / types.ts). Listed here so the redesign
knows they exist and where the assertion lives; treat as UNRESOLVED-SHAPE until each json! body is read.
  - SETTINGS setters/getters: set_gate_mode:947, set_local_max_context:4769, set_local_min_prompt:4787,
    set_mcp_enabled:4801, set_execution_mode:4807 / get_execution_mode:4840, set_cloud_model:5181 /
    get_cloud_model:5167, set_webhook_url:5354 / get_webhook_url, fetch_settings_get:6999 / fetch_settings_set:7014
    (mostly () or echo-ack; fetch_settings_* return a settings Value blob)
  - CLOUD: get_cloud_spend:5217, get_cloud_route_status:5242 (Value)
  - OAUTH: get_oauth_status:4952, start_oauth_login:4982, do_oauth_logout:5141 (Value/ack)
  - SIDECAR: get_sidecar_settings:7525 / set_sidecar_settings:7547 / restart_sidecar:7573,
    start_remote_sidecar:7692 / get_remote_sidecar_progress:7784 / cancel_remote_sidecar:7800 (Value/ack)
  - RUNTIME: list_runtimes:10228, test_runtime:10311, set_role_runtime:10378 (Value/ack)
  - CHAT/TASK: start_chat_stream:5839 (streaming; token events), cancel_chat_stream:6242, run_research:6857,
    submit_task:499, continue_task:4757, query_knowledge_engine:4630, run_explore_query:4706 (Value)
  - INTERVENTION: submit_intervention_response:532, respond_to_confirmation:563, submit_credential_response:585 (ack)
  - INTENT PAIRS: get_intent_pairs:11067 (TS IntentPairSetLite), add_intent_pair:10988, confirm_intent_pairs:11017;
    query_focus_tunnel_per_baby_step:8385 (TS FocusTunnelPerStep) — both TS types asserted inline, Value on Rust side
  - PLAN: get_phase_detail:10580, get_baby_step_detail:10604, get_phase_progress:10629 (Value)
  - SESSIONS: get_session_branches:3413, fork_session_branch:3584, merge_session_branch:3677, close_session_branch:3701 (Value/ack)
  - PROJECTS: list_projects:10842, create_project:10880, open_project_from_folder:616 (Value/ack)
  - GRC: get_grounding_sweep:2868, get_csq_pending:2917, get_csq_log:2986 (Value; zero-shape defaults)
  - REINDEX: reindex_now:6388, cancel_indexing:6505 (ack)
  - get_reflect_state:8340 (Value)


================================================================================
SECTION: UNRESOLVED / PARTIAL (do NOT invent fields for these in the redesign)
================================================================================

FULLY UNRESOLVED (handler/DTO not opened this session — read before wiring):
  - get_mcp_security_status (commands.rs:261) — McpStatusPanel; TS type McpSecurityStatus[]; body not read.
  - get_calibration_corpus_state (commands_d16.rs:191) — ConformalCalibrationPanel; DTO CalibrationCorpusStateDto not located.
  - get_refusal_rate_current (commands_d17.rs:127) — RefusalRateGauge secondary; body not read.
  - The "REMAINING SECTION-1 COMMANDS" block above (settings/cloud/oauth/sidecar/runtime/chat/intent-pairs/
    plan-detail/sessions/projects/grc-sweep/csq/reflect) — each returns CmdResult<Value> or an ack; the json!
    body / TS type was not opened individually. Each is UNRESOLVED-SHAPE.
  - get_pinned_rules (PinnedRulesPanel) — per CONTRACT-SHEET 5d, NO #[tauri::command] of that name exists in
    src-tauri/src/*.rs (grep-confirmed in the contract sheet). Panel will error/empty. RESOLVE before wiring.
  - get_isolation_metric_status (referenced in api/stub.ts) — per CONTRACT-SHEET 5d, no Rust handler found.
    Likely stub/mock-only wrapper. Verify before relying.

PARTIAL — top-level shape transcribed, but a sub-object is a 1-hop type NOT expanded here (UNRESOLVED-SUBOBJECT):
  - get_graph_view_data -> VizData (elai_knowledge::visualization::types::VizData). TS mirror is api/types.ts VizData (authored).
  - get_decomposition_tree -> DecompositionTreeSnapshot (ssb_viewer.rs:248). Has top-level "concepts":[...]; nested
    phase/step/babystep node fields not expanded. TS mirror: api/types.ts DecompNode.
  - get_archive_recent / get_archive_lineage -> ReasoningTemplateDto (orchestrator::reasoning_template). Carries id (hex) + domain/pattern/body per TemplateSummary; full field set not expanded.
  - symbol_graph_query/pagerank -> SymbolMatch / RankedSymbol (orchestrator::symbol_graph). Element fields not expanded.
  - github_pr_open/list/status -> PrSummary / PrStatus (orchestrator::github_pr). Element fields not expanded.
  - list_entry_points -> BackendEntryPoint[] (built from SymbolGraphCache). Element fields not expanded.
  - get_research_node_history -> research_node_stages row (orchestrator viewer). Element fields not expanded.
  - get_media_validations -> MediaValidationEntry sub-snapshots: Tier1Snapshot / Tier2Snapshot / CapturedFrameSnapshot /
    IterationSnapshot / AxtreeSnapshot (all in state.rs). Top-level MediaValidationEntry IS fully transcribed; these nested Option/Vec elements are not.

NOTE ON CmdResult<Value> HANDLERS THAT WERE FULLY SHAPED:
  Where a handler returns serde_json::Value, the shape above is transcribed from the EXACT json!{...} literal in the
  handler body (field names + value sources), not from a TS guess. Where a handler does serde_json::to_value(rows) over
  a Vec<SomeRow>, the row struct (cited file:line) IS the wire shape — serde serializes Rust field names verbatim
  (confirmed by the per-module doc comments). serde attributes that change the wire shape are noted inline:
  #[serde(default)] (field omittable on old rows), #[serde(skip_serializing_if="Option::is_none")] (field absent when None),
  BTreeMap<K,V> serializes as a JSON object, Vec<(A,B)> tuples serialize as JSON arrays [a,b].
  PascalCase enums (e.g. ReviewDecision) carry #[serde(rename_all="PascalCase")].
