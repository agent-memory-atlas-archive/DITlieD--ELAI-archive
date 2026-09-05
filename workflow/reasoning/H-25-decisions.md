H-25 result manifest re-derivation gate (T-9) -- decisions
Session: s-8404 | 2026-06-05 | SCHEMA_VERSION 229 -> 230, TOTAL_VARIANTS 506 -> 507

WHAT
A SCHEMA_REQUIRED result manifest binding each reported empirical NUMBER (Sharpe /
P&L / plot-derived stat) to its provenance, plus a Tier-1 verifier
(result_manifest::verify_result) that RE-PARSES the number from the
orchestrator-captured output and RE-HASHES the pinned inputs, asserting equality. A
hand-edited number whose captured-output digest does not re-derive -> Block. The
anti-fabrication complement to the H-24 sim-to-real parity gate; closes CLUSTER E's
"a reported result was never produced by the pinned inputs" gap (the Replit
faked-tests + fabricated-4000-users failure class generalised to a number).

KEY DECISIONS

D-1 (GD-1 scope -- the RRA "reuse" is NARROW; NEW typed struct, NOT an RRA
  extension). The plan body's "the manifest IS the RRA bundle" is WRONG. RRA
  RunArtifact.manifest is Vec<GateVerdictHash> (rra.rs:131/220) -- it enumerates
  GATE-VERDICT rows (VibeGateBlocked etc), NOT reported result NUMBERS, and carries
  none of {value, captured_output_blake3, parser_expr, data_hash}. So H-25's
  ResultManifest is a GENUINELY NEW typed struct in
  crates/orchestrator/src/result_manifest.rs. Extending RunArtifact would break the
  frozen canonical_bundle_bytes tuple + RRA_FORMAT_VERSION + the ts-rs RunArtifact
  bindings (the exact cross-stack drift H-23 hazard-C avoided). What H-25 actually
  REUSES from RRA (GD-1): (a) the blake3 16-hex prefix convention
  (crate::rra::RRA_HEX_PREFIX_LEN) for every hash; (b) the F-4 None-hash anti-fab
  DISCIPLINE (evidence.rs:845 .filter(|r| r.passed && r.stdout_hash.is_none());
  gate.rs:441 "a passing row with None hash is the fabrication signature") --
  generalised from a missing stdout_hash to a missing/trivial manifest; (c) the
  DriftComparator first-divergence (determinism.rs:317) over the declared vs
  re-parsed result sequences. The gate shape mirrors H-23 (rerun_equality.rs)
  almost exactly. The F-4 stdout_hash leg is REUSED, not re-implemented (GD-1).

D-2 (the manifest fields + the re-parse/re-hash verification). ResultEntry binds
  {metric_name, value (kept as its EXACT string repr so re-parse equality is
  byte-exact, not float-eq), captured_output_blake3, parser_expr, data_hash,
  code_commit, env_lock_hash, n_trials, dsr_pvalue, pbo}. verify_result order
  (FAIL-CLOSED first): (1) trivial/empty manifest => FabricationDetected
  reason=trivial_manifest; (2) missing captured output (rederived=None) =>
  FabricationDetected reason=missing_captured_output; (3) per-entry RE-PARSE: apply
  the restricted parser to the matching captured bytes, compare re-parsed token vs
  declared value via the reused DriftComparator first-divergence (names the FIRST
  divergent result), mismatch => value_mismatch (or unparseable_result); (4)
  per-entry RE-HASH: captured_output/data/env/commit must match the declared
  digests, mismatch => hash_mismatch; (5) all re-derive over a non-trivial set =>
  ReDerived PASS. One ResultManifestVerified SSB row on EVERY decision (R4 off-agent
  artifact = the captured output + the re-parsed value the agent did not author).

D-3 (DSR/PBO/N are BOUND from H-17/H-18, never recomputed -- GD-1). ResultEntry
  carries n_trials (H-17 trial_count), dsr_pvalue + pbo (H-18) as DATA fields the
  manifest binds. verify_result does NOT recompute them; the result manifest is the
  PLACE those numbers are bound to the reported value, not where they are computed.
  H-17 trial_ledger.rs::trial_count + H-18 deflation_gate.rs dsr_pvalue confirmed
  SHIPPED (H-25-grounding.md lines 13-14).

D-4 (parser_expr SAFETY -- RESTRICTED grammar, NO string-eval; the load-bearing
  safety bit). A parser_expr that was arbitrary code could "re-derive" any value (a
  malicious parser hands back exactly the declared number, defeating the gate). So
  the parser is a FIXED enum (ParseStrategy), NOT a string-eval: the only Tier-1
  strategy is `labeled-float:<label>` -- find the first occurrence of <label> in the
  captured output and parse the next float token after it. There is no interpreter,
  no regex injection, no callback. An unrecognised parser_expr is a hard
  FabricationDetected (a number whose re-parse cannot be performed is not
  re-derived; unit test unparseable_parser_expr_is_fail_closed_block + contract
  test (4)-adjacent). The grammar is closed; adding a strategy is a code change and
  is gate-frozen via the gate-manifest. This resolves the plan's open-question
  (lines 157-159) and the context-package VALIDATION WARNING (MED).

D-5 (the monkeypatch AST ban + the frozen harness -- TD-2). FIN-RESMANIFEST-002
  (AstPattern, python sexp) bans strategy code from monkeypatching/redefining a
  RESULT-EMITTING fn (setattr(obj,"compute_sharpe",...) or a module-level rebinding
  of compute_sharpe/report_pnl/emit_metric/record_result) to FAKE a number. SCOPED
  to the result/metric/report surface via the execution_path:<csv> predicate
  (rule_engine.rs:401 scope_matches -- a module-token CONTAINS predicate, NOT a path
  prefix; a blanket ban would false-block legitimate DI/config setattr). The harness
  is gate-manifest-frozen (TD-2). H-24 LESSON (R6): an AstPattern that never FIRES
  is wire-dark. The contract test part_a_monkeypatch_ban_fires_on_in_scope_result_fn_rebind
  RUNS the real DefaultRuleEngine + TreeSitterAstEvaluator over a monkeypatch
  fixture and asserts Block; part_a_monkeypatch_ban_does_not_fire_on_out_of_scope_benign
  asserts no-fire on a benign config setattr. Red-when-inert verified: reverting the
  execution_path scope to a path-prefix makes the firing test go Allow (red), per
  the scope_matches starts_with-vs-CONTAINS split (the same red-when-inert proof
  H-24 shipped for FIN-SIMREAL-001). MonkeypatchBlocked is emitted by the SEEDED
  rule-engine path, NOT by verify_result -- same producer split as H-24 (only the
  pure verdicts come from the gate fn; the AST/schema arms come from the seeded
  path), so verify_result returns only ReDerived / FabricationDetected while the
  3-arm Kind still carries MonkeypatchBlocked.

D-6 (pure-vs-seeded, mirror H-23/H-24). The verify_result re-parse+re-hash is PURE
  (no rules.db threshold). The SchemaRequired manifest-shape rule
  (FIN-RESMANIFEST-001) + the monkeypatch AstPattern (FIN-RESMANIFEST-002) live
  SEEDED, mutable=0, under crates/rules/seed/repro/result_manifest.toml, seeded by
  project_rule_seeder.rs::seed_result_manifest_rules and called from the production
  caller runtime_setup.rs (after the H-24 seed block) -- without that caller the
  rows would be test-only / wire-dark.

D-7 (TOML path + domain -- seed/repro/, domain="trading"). The plan WIRING_CONTRACT
  said seed/finance/result_manifest.toml + domain=finance; the SHIPPED H-23/H-24
  seed TOMLs live under crates/rules/seed/repro/ and use domain="trading"
  ("finance" is NOT in DOMAIN_KEYWORDS -- trading_nulls.toml:3 / rule_engine.rs
  detects "trading" via [trading,backtest,signal,pipeline]). Used
  seed/repro/result_manifest.toml + domain="trading" (the explorer DRIFT finding).

D-8 (entrypoint -- replay --verify --result-manifest, ADDITIVE). The plan named a
  `manifest-check`-style CLI; instead used a `replay --verify --result-manifest
  <demo>` sub-flag, sibling-consistent with H-23 --metric-digest / H-24 --golden and
  routing through the existing run_replay dispatch (no new top-level verb). Modes:
  complete => ReDerived/PASS; tampered => FabricationDetected/BLOCK (hand-edited
  value); incomplete => FabricationDetected/BLOCK (trivial manifest,
  anti-sandbagging). Production caller run_verify_result in rra_cmd.rs exits non-zero
  on FabricationDetected (CI gate). The "no reactor" footgun (Ssb::new spawns its
  write-loop) is handled exactly like H-23/H-24: build a multi_thread runtime and
  construct the Ssb + drive verify_result INSIDE block_on.

COMPLETENESS HONESTY (TD-5, the single most important honesty point -- do NOT
  overclaim). Tier-1 is SOUND for (a) declared-manifest RE-DERIVATION (each DECLARED
  entry re-parses to its declared value AND re-hashes to its declared digest), and
  (b) a NON-TRIVIAL guard (an empty/trivially-small manifest => Block;
  MIN_DECLARED_RESULTS=1, mirror H-23 MIN_DECLARED_METRICS / H-24 MIN_GOLDEN_EVENTS).
  Tier-1 proves the declared results are REAL + NON-TRIVIAL; it does NOT prove the
  declaration is COMPLETE (no HIDDEN output the run produced but did not declare, no
  cherry-picked-but-nonempty subset). There is NO existing "declared vs
  actually-produced output" cross-check in the codebase (grep found no
  enumerate-actual-outputs primitive); the structural floor blocks the
  trivially-empty manifest, but a TRUE produced-vs-declared diff needs the deferred
  Tier-2 Python runner that ENUMERATES the orchestrator capture sink and diffs the
  count -- RunPod-pending (cloud-only override), no stub shipped, cross-lang-skip
  header on result_manifest.rs. This is the SAME necessary-not-sufficient residue as
  H-23/H-24. in-toto / Sigstore / Rekor / DVC are "design-inspired-by" ONLY: H-25
  inherits NO non-repudiation / append-only / supply-chain-attestation guarantee
  from them (that is the separate, already-shipped G-8 attestation); they are cited
  as the provenance-binding precedent, not as inherited semantics.

FAIL-CLOSED (R1/R2): missing manifest / 0-or-trivial entries (anti-sandbagging, the
F-4 None-hash analog) => FabricationDetected; missing captured output (rederived
None) => FabricationDetected (R2 missing-evidence=FAIL); any value re-parse mismatch
or input re-hash mismatch => FabricationDetected; an unrecognised parser_expr =>
FabricationDetected (a number whose re-parse cannot be performed is not re-derived).
One SSB row on EVERY decision including ReDerived.

SSB LOCKSTEP (+ResultManifestVerified, Pattern-2 + inner 3-arm
ResultManifestVerifiedKind ReDerived/FabricationDetected/MonkeypatchBlocked).
14 sites moved together: ssb.rs SCHEMA 229->230 + bump-comment, CHECK list (trailing
comma after 'sim_real_parity' then 'result_manifest_verified'), SsbActionType enum +
as_sql_str + from_sql_str, SsbPayload enum + action_type() match, the
ResultManifestVerifiedKind enum + ResultManifestVerifiedPayload struct substrate
block, all_test_payloads demo row + len 426->427; action_type_lock.rs
TOTAL_VARIANTS 506->507 + ALL_VARIANTS entry + exhaustive match; e45:294 506->507 +
the ResultManifestVerified presence assert.

FILES
- CREATE crates/orchestrator/src/result_manifest.rs (ResultManifest + ResultEntry +
  ReDerivedInputs + verify_result + the restricted ParseStrategy grammar + 8 unit
  tests). cross-lang-skip header.
- CREATE crates/rules/seed/repro/result_manifest.toml (FIN-RESMANIFEST-001
  SchemaRequired + FIN-RESMANIFEST-002 monkeypatch AstPattern, mutable=0,
  domain="trading").
- CREATE crates/contracts/tests/h25_result_manifest_contract.rs (6 groups incl the
  evaluator-RUNNING monkeypatch fire/no-fire pair).
- CREATE workflow/reasoning/H-25-decisions.md (this file).
- MODIFY crates/orchestrator/src/ssb.rs (the 10 in-file SSB sites).
- MODIFY crates/orchestrator/tests/action_type_lock.rs (3 sites).
- MODIFY crates/contracts/tests/e45_cross_language_drift_contract.rs (1 site).
- MODIFY crates/orchestrator/src/lib.rs (pub mod result_manifest).
- MODIFY crates/rules/src/project_rule_seeder.rs (seed_result_manifest_rules).
- MODIFY crates/orchestrator/src/runtime_setup.rs (the production seeder caller).
- MODIFY crates/cli/src/args.rs (ReplayAction::VerifyResultManifest +
  ResultManifestDemoMode + --result-manifest parse).
- MODIFY crates/cli/src/rra_cmd.rs (run_verify_result + result_manifest_demo_inputs
  + dispatch arm + use import + constants).
- MODIFY .claude/gate-manifest.txt (freeze result_manifest.rs + .toml; --lock
  deferred OUT-OF-BAND per R8, exactly as the H-23/H-24 manifest comments state).

NEXT
H-26 (runtime differential / shadow execution + financial-invariant MR library)
catches WRONG VALUES static + wiring gates cannot see. H-25 + H-26 close CLUSTER E.
The deferred Tier-2 (produced-vs-declared enumeration) reuses the same Python runner
RunPod-pending across H-23/H-24/H-25.
