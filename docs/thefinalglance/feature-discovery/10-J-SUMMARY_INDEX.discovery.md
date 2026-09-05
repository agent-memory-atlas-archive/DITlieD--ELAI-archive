# J-plan exhaustive feature discovery

## Coverage and reading ledger

- Direct source: docs/thefinalglance/J-plan/J-SUMMARY_INDEX.md.
- Coverage: all 2,386 physical lines read, from line 1 through the explicit end marker at line 2,386.
- Entry coverage: 112 distinct level-3 J headings; J-1 through J-112 are each present exactly once. No numbered J entry is missing from the body.
- Orientation only: docs/thefinalglance/ELAI-SPEC-OVERVIEW.md, especially its J synopsis at lines 294-349. It is not treated as a substitute for the J summary.
- Source-status notation: [E] means explicitly stated in the J summary; [I] means a bounded inference made here; [A] means the summary itself is ambiguous, stale, conditional, or internally inconsistent.
- Anchors below are physical line ranges in J-SUMMARY_INDEX.md. External figures are design rationale unless an entry explicitly requires an ELAI-measured result.
- Structural anomalies: J-32 precedes J-31; J-99..J-102 precede J-92..J-98; J-112 precedes J-111. This is ordering only, not missing coverage.
- [A] Line 2 claims detailed entries stop at J-91 and J-92..J-98 were not backfilled. That note is stale inside the same file: J-92..J-94 appear at lines 2,020-2,050 and J-95..J-98 at lines 2,062-2,123. All corresponding numbered source files also exist.
- [A] Status words such as SHIPPED, BUILT, DARK, PARTIAL, PLANNED, DEFERRED, and WIRED are preserved as design/source assertions; this discovery did not independently inspect code or numbered originals.

## Cross-cutting behavior and invariants

- [E] The model proposes or generates; deterministic harness code, frozen checks, independent receipts, or a human disposes. Model self-report, stdout alone, same-family self-evaluation, and LLM-judge verdicts are repeatedly forbidden on load-bearing paths.
- [E] Missing, malformed, unmappable, unverified, chain-failed, or unknown evidence fails closed or degrades loudly; it never becomes a silent pass, empty success, or fabricated default score.
- [E] The append-only single-writer SSB journal is the canonical runtime store. New surfaces should derive from or extend existing rows; parallel JSON/state stores and duplicate action types are disfavored.
- [E] Every state-changing or replay-relevant action—including inserted, suppressed, edited, redirected, voice-routed, and external-response actions—must be journaled in total order and reproducible from the RRA where the lane permits it.
- [E] Harness-only .claude/workflow features use JD-1 off-agent evidence (real hook/tool execution plus logs/artifacts) because they have no SSB. Their negative arm must make an unwired/dead path scream.
- [E] New gates normally begin OBSERVE-ONLY; promotion waits for measured false-positive/quality evidence. Default-off features require an allowlist/wire-dark row and promotion plan; disable flags must surface their no-op.
- [E] Thresholds, knees, model capability, cost, retrieval value, and external benchmark deltas must be measured on ELAI fixtures per model/lane. Paper/vendor numbers are not production thresholds or product claims.
- [E] Frozen gate inputs/manifests, auditor re-runs, writer isolation, and generator/auditor separation prevent the judged run from rewriting its judge.
- [E] Runtime backend, model, lane, and action modality are replaceable policy choices behind shared interfaces; per-turn or per-sequence locks prevent unsafe mid-flight swaps.
- [E] Local/cloud distinctions remain explicit: local can use decoder control, KV caching, and deterministic token paths; opaque cloud/OpenAI-compatible paths use validation/repair/fallback and honest reproducibility downgrades.
- [E] Security invariants recur at every ingress and sink: taint survives transformations/processes/memory, secrets and key material never enter rows, high-risk actions require authoritative gates, and config/instruction artifacts are re-admitted after change.
- [E] No-removal and no-rearchitecture discipline: ablations measure but do not prune; alternative backends are additive; existing traversal, stores, routers, gates, and dispatchers are extended rather than forked.
- [E] User consent is preserved for new dependencies, destructive/high-risk operations, grill invocation, policy-engine adoption, model/backend removal, and speculative product coupling.
- [E] Frontend pairing is mandatory when runtime state needs a user surface; otherwise the plan must explicitly justify N/A or reuse an existing panel/drawer.

## Exhaustive atomic inventory

### J-1 — perception gateway + /see skill [the only user idea with no lane in B..I] [L105-L127]

- [E] Preprocess every screenshot, diagram, or mockup before any model call: resize to the target model sweet spot, convert PNG to RGB JPEG q85, and tile dense UIs with a thumbnail.
- [E] Compose UI pixels with AXTree/DOM text; route documents to E-36 rather than treating them as UI screenshots.
- [E] Use two calls: literal elements to structured JSON at temperature 0, then coding-model reasoning over description plus image.
- [E] Probe a small VLM first, escalate on low confidence, and support image-prompt caching.
- [E] Route visual diffs to deterministic pixel diff, never a VLM; reuse B-24 Tier-1 veto.
- [E] Provide a thin /see skill that locally runs preprocess; verify Pillow before use and degrade to passthrough.
- [E] Integrate at every image-bearing dispatch through orchestrator perception modules plus Python sidecar heavy legs; flag remote-3090 hardware before boot.
- [E] Emit PerceptionScaffoldEvent and surface perception rows in an existing drawer.
- [E] Done gate: journal row plus raw-control-beating fine-text answer, seeded click coordinate error under 2%, and a broken --raw-image arm.

### J-2 — elai-as-MCP-server control plane [security-gated; WAVE 4 deliberately] [L129-L151]

- [E] Expose a long-lived local MCP server with task_submit, task_status, fsm_get_state, session_fork, session_rewind, plan_status, rra_show, and intervention_respond.
- [E] External clients include Claude Code, dashboard, and scripts; stdio/local/same-user is first, TCP is out of scope.
- [E] Every verb rides the existing primitive and the same FSM/security gates as interactive use; verbs never bypass authorization, destructive gates, session/RRA, or intervention flows.
- [E] Authorization defaults DENY-ALL for unlabeled verbs; dependency pins/re-pins and taint/destructive protections apply.
- [E] Reuse C-39 MCP client/glue rows instead of parallel bookkeeping; add at most one control-plane event.
- [E] Surface control-plane session rows in an existing drawer unless a W-6 absence is justified.
- [E] Pre-gate live re-checks C-42, E-24, G-4, E-30, and G-3 before implementation; G-4 was specifically noted as potentially dark.
- [E] Done gate compares MCP-submitted and interactive journal sequences; unauthorized and ungated destructive calls block; --skip-authz must demonstrate failure.

### J-3 — harness undo + per-session file backups [harness-safety KEYSTONE] [L153-L176]

- [E] A new PreToolUse hook for Edit, Write, and NotebookEdit copies the pre-image before the write.
- [E] Backup path is session-scoped, content-addressed by hash prefix and relative path with sequence; deduplicate content and gzip over a size threshold.
- [E] undo.py supports list, diff, restore, and prune.
- [E] Restore refuses divergence when current disk bytes no longer match the recorded post-image and shows a diff.
- [E] Mark the last version before an audit PASS as last-good; prune keeps last-good plus newest.
- [E] Use keyed stdlib blake2b with an algorithm prefix for later blake3 migration; do not port H-49 CAS.
- [E] Register a new hook without editing pre_tool_check.py; keep it non-blocking and harness-only.
- [E] Done gate proves backup-before-edit ordering and exact-byte restore; unregistered hook yields zero backups and a loud explanation.
- [E] The stale F:-hook bug remains explicitly out of scope.

### J-4 — handoff file ledger [L178-L195]

- [E] Machine-derive one row per session-edited file: path, edit count, optional J-3 last-good sequence, and audit verdict.
- [E] Read existing PostToolUse change records, both state.json changes and hook-executions.log shapes, defensively.
- [E] Add exactly one ledger line/block to the handoff template; retain one-file-per-problem behavior.
- [E] Never reconstruct FILES TOUCHED from model recollection or hand tally when the tool exists.
- [E] Missing session state emits visible 'ledger unavailable', not a silent empty section.
- [E] Harness-only, stdlib-only, no frontend; spot-check output against hook logs.

### J-5 — litegame harness port [checklist, not a build — JC-6] [L197-L213]

- [E] This is a port checklist, not a new architecture.
- [E] First re-verify seven liteGAME compatibility files after the F:-move to avoid wrong-root ghost behavior.
- [E] Port multi-root-safe local-rag MCP registration, J-3 backup/undo, J-4 ledger, F-1 compound PreToolUse shape, and skillspector --no-llm import gating.
- [E] Copy and adapt paths; never share mutable config between roots.
- [E] Live done gate runs in the liteGAME root: local backup, local ledger, correct mcp_fsm_get_state resolution, and refusal of a deliberately wrong root.
- [E] Findings belong in that root's reasoning log; frontend is N/A.

### J-6 — incremental-eval skill [WAVE 1; validation run = the session that authored J] [L215-L233]

- [E] For N at least 3 artifacts, process exactly one at a time and never batch-read.
- [E] Append a bounded anchor plus 3-8-line verdict block to one topic-keyed workflow/reasoning file.
- [E] Per cycle context is task spec, verdict-file index, and next artifact; reread only the verdict file head.
- [E] Explicit anti-patterns: batch reads, unbounded verdicts, full-file rereads, and unanchored verdicts.
- [E] Package SKILL.md with positive and negative JSONL evals; frontmatter must pass agentskills_validator.py.
- [E] Validation is a live invocation producing the incremental file; the J prereading run is the recorded 14-section example.

### J-7 — ext-steal intake closeout [the +1; source = SRC-2, pending user yes] [L235-L258]

- [E] Run skillspector against an internal skill with --no-llm JSON, parse per-finding severity/count, and append a scan-hash TSV decision row.
- [E] Mark skillspector resolved in the import policy; demote mcp-scan to optional; state that rug-pull stubs are not covered and G-4 remains authoritative.
- [E] Verify impeccable package metadata, trial its fast JSON detector on dashboard-v2, and add one dashboard-smoker checklist line.
- [E] Insert about nine source-file pointer lines into named consuming plans; future reimplementation must read the named source, not only summaries.
- [E] Keep importing impeccable's 23-command skill on HOLD.
- [E] Negative malicious-skill fixture must yield HIGH/CRITICAL and BLOCKED; missing citations or aggregate-only scores are insufficient.
- [A] Header says 'pending user yes'; dependency/action authorization must be rechecked in the original.

### J-8 — vlm perception research upgrade pack [post-J-7 extension; depends J-1 DONE] [L260-L281]

- [E] Add per-model resize DATA with native-resolution passthrough.
- [E] Enforce normalized 0-1000 coordinates in scaffold schema v2.
- [E] Move counts-from-AXTree/DOM-never-VLM into the runtime composer.
- [E] Multi-image input becomes tiled composite/B-24 Mode B; micro-text uses tiling then typed refusal with residues.
- [E] Assert cache placement composes with I-10.
- [E] Add evaluate-not-ship comparisons for dots.ocr and Qwen3-VL-Embedding against E-36, gated on resolved candidates/hardware.
- [E] Provide H-14 component-inventory prompt shape as a library template and seed J-9 vision-ladder data.
- [E] Never edit the J-1, E-36, H-14, E-51, or B-24 plan files.
- [E] Emit/reuse VLM perception event and reuse J-1 UI; legacy pixel coordinates and unresolved candidate names must fail.

### J-9 — hardware detection + per-lane model eligibility catalog [post-J-7 extension] [L283-L307]

- [E] Reuse shipped GPU/system probe, VRAM classification, capability profile, and can_fit_llm; probe once.
- [E] Resolve TOML catalog data into coding, vision, OCR, and embedding eligibility by quantized VRAM estimate.
- [E] Every ineligible row carries a reason; zero-VRAM yields explicit cloud-only residue, never empty output.
- [E] Lift Gemma 4/Qwen3-32B box-specific bans into tier-conditional catalog rows with the prior-rejection note retained.
- [E] Catalog is informational only; routing, allowlists, and CLAUDE.md must not consume it, enforced by grep.
- [E] Emit CatalogBuilt/HardwareModelCatalogEvent at startup.
- [E] Defer CLI and HardwareModelCatalogPanel together as Tier 2, registered rather than dropped.
- [E] Contract tests reject eligibility above fixture VRAM and restrict Gemma-4 to at least 24GB fixtures.

### J-10 — insight backpropagation tree memory [the Arbor steal-list's one net-new capability] [L309-L334]

- [E] Extract a fixed-contract 1-3 sentence insight from every audit/experiment report using a small LLM call.
- [E] On each verdict, resynthesize every ancestor from child insights up to the root global memory.
- [E] Inject lineage insights downward into executor prompts.
- [E] Render ROOT INSIGHT priors, PRUNED LESSONS, and VALIDATED FINDINGS as a constraints block for ideation.
- [E] Bind insight text to hashes of source SSB rows; the model writes text but never the trust verdict.
- [E] Store on SessionDag/receipts; loser and pruned branches teach; reuse I-5 contrastive constraints and avoid duplicate I-11 bookkeeping.
- [E] Fire on audit verdict, execution dispatch, and I-4 ideation; add InsightPropagated and an insight/lineage UI column.
- [E] Forged hashes become UNTRUSTED and are not injected; --no-insight should reproduce the pruned proposal.

### J-11 — arbor / combench / denovoswe steal intake closeout [the S-2..S-8 amendments; the J-7 analog] [L336-L357]

- [E] Amend existing plans with source-cited contracts rather than writing runtime code or a new subsystem.
- [E] Held-out merge gate reruns B_test off-agent in a fresh detached worktree.
- [E] Explicitly forbid Arbor fail-opens: unset eval command falling back to LLM score and mutable eval config in tree metadata.
- [E] Add deterministic score parsing, five worktree lifecycle mechanics, and selected prompt phrases without porting architecture.
- [E] Add ContextManager layers 1-3 as prior art; reject layer 4 at 32K.
- [E] Add ComBench checker contract, register-time reference fixture gate, and an explicit demotion clause.
- [E] Add difficulty-banded inclusion floor and leakage threats.
- [E] Each amendment requires source file:line; uncited or DO-NOT-COPY-as-pattern amendments fail review.

### J-12 — runtime intervention harness [Life-Harness 4-layer port; LLM-free at runtime] [L359-L377]

- [E] Add pre-tool protocol rules to the existing rules lifecycle and PreToolUse chain; violations return Block(message) so an agent can self-correct.
- [E] Never block merely on ambiguity.
- [E] Embed constraints into tool descriptions so they are resent every request.
- [E] Add read-only per-tool result annotators returning optional text.
- [E] Retrieve at most the BM25 top-1 trusted skill with K1=1.5/B=0.75 and a precision guard.
- [E] Offline failure taxonomy priority is action-realization, contract-mismatch, trajectory-degeneration, residual; regression checks feed I-12.
- [E] Reuse rules/hooks/skills/anti-stagnation seams; add HarnessInterventionEvent and an existing-drawer surface.
- [E] SHADOW rules observe only; zero-rule registry is a provable no-op; disabling intervention should reproduce the protocol error.

### J-13 — local-lane token determinism [the seed slot is already there, dark] [L379-L394]

- [E] Wrap the sidecar generate path once with pinned batch_invariant_ops; avoid nested wrapping; warm Triton at boot; cover bf16 NF4 addmm.
- [E] Add SeededGumbelProcessor to the existing logits chain.
- [E] Thread sampling_seed from ELAI_SEED through Rust client into the RRA so replay reissues it.
- [E] State honest residue: SDPA/FA2 attention is not covered; mode is graded opt-in and cloud is out of scope.
- [E] Measure overhead before default flip.
- [E] Add LocalDeterminismEvent and runtime-panel status.
- [E] Same-seed local RRA runs must have identical token-stream blake3; mode-off divergence is recorded, never hidden.

### J-14 — repro-pack + rra verify upgrade [tiered, self-describing, locates the first divergence] [L396-L414]

- [E] Verify tiers T0 action-faithful, T1 token-faithful, and T2 score-faithful.
- [E] Capability-gated downgrade is recorded; strict mode aborts.
- [E] Bundle determinism_applied describing controls actually active.
- [E] Locate first divergence at token position and feed model-drift diagnostics.
- [E] Persist file pre/post hashes; keep CAS ownership with H-49.
- [E] Always scrub auth/secrets; storing auth headers is explicitly forbidden.
- [E] Add content-keyed rra export-fixture with per-fixture blake3 for I-49.
- [E] Guard crystallized pipeline fingerprints; drift de-crystallizes and falls back to LLM path.
- [E] Add observe-only entropy-scan CI; missing determinism_applied fails verify.

### J-15 — determinism steal closeout [the J-11 analog; zero SSB, zero runtime code] [L416-L433]

- [E] Add llguidance Rust grammar compile, validate, and blake3-pin to I-34; new dependency requires confirmation; reject full-Rust masking option B.
- [E] Journal recorded escalation hops in E-31/H-7.
- [E] Add exit-code script steps, MCP config delegation, and transparent proxy shape to J-2.
- [E] Add bounded solver-error-feedback reformulation to D-11 while checker verdict remains authoritative.
- [E] Add event-id anchored forks and mutate-then-replay adversarial rider to H-49.
- [E] Bind I-49 to the J-14 fixture contract.
- [E] Upsert roughly 40 explicit rejections, including empty/irrelevant/name-collision repositories, into the feature map.
- [E] No runtime code or SSB; every amendment needs source citation and dependency flag.

### J-16 — blast-radius / anti-regression gate [KEYSTONE — the user's #1 priority] [L442-L460]

- [E] Turn B-29 impact analysis into an auditor-owned enforced gate on every accepted edit batch.
- [E] Compute reverse-dependency cones for changed symbols.
- [E] Enforce declared diff scope plus forced dependents; flag out-of-scope/unrequested edits.
- [E] Select and rerun existing tests over the cone; a pre-pass to post-fail transition blocks.
- [E] Flag undeclared net-negative symbol deletion.
- [E] Deterministic, no LLM adjudication, observe-only first.
- [E] Add BlastRadiusEvent and audit-timeline panel.
- [E] Empty cones with known dependents and unmappable diffs fail rather than pass.

### J-17 — plan-level compositional safety [the SIF gap per-step gates can't catch] [L462-L477]

- [E] Before dispatch, aggregate capability sets across the concepts-to-phases decomposition DAG.
- [E] Deny jointly dangerous goals even when each subtask is individually benign, using rules-engine composite predicates and G-1 capabilities.
- [E] Add remote-sidecar sandbox configuration audit; gVisor remains a note, not a rewrite.
- [E] Observe-only first; do not build SAFEFLOW-scale IFC.
- [E] Fragmentation finding names the contributing subtasks and appears before any dispatch.
- [E] Jointly benign plans clear; unresolvable DAGs fail loudly.

### J-18 — P1 retrieval hardening [CodeRankEmbed 768d swap + LSP channel + agentic-grep] [L479-L494]

- [E] Swap the 768-dimensional code embedder to official CodeRankEmbed INT8 ONNX with exact dimension and an index_version rebuild.
- [E] Do not use the low-download community quant.
- [E] Add an LSP fourth channel through Serena MCP or rust-analyzer stdio for go-to-definition/find-references; tree-sitter remains fallback.
- [E] Add index-first agentic grep repair after retrieval.
- [E] Feed compiler-grade macro/trait/dynamic-dispatch edges into J-16 blast cones.
- [E] Emit/reuse RetrievalAugment/QueryTrace channel breakdown in KnowledgeDrawer.
- [E] LSP init failure degrades through existing cascade; dimension mismatch aborts migration without corrupting index.

### J-19 — vLLM-V1 alternative backend [the lone REPLACE, handled ADDITIVELY] [L496-L514]

- [E] Add RuntimeBackend::VllmV1 behind B-37; use native /v1/messages without a new Rust client.
- [E] Add AWQ-Marlin and automatic prefix caching; use VLLM_BATCH_INVARIANT on SM8.6.
- [E] Maintain a logits portability map: DoLa unsupported/HF-only, XGrammar native, ETC/seeded Gumbel custom-portable, prompt lookup not portable.
- [E] Measure HF versus vLLM; backend replacement/removal remains a user decision.
- [E] Reject SGLang because of the cited RCE surface and protocol mismatch; reject nonexistent Qwen3-Coder-14B.
- [E] Reuse backend-selection rows and show comparison in RuntimePanel/CloudPanel.
- [E] DoLa request on vLLM and AWQ build without calibration data fail capability/build checks.

### J-20 — self-improve non-stationary bandit + security-invariant gate [supersedes plain-TS] [L516-L532]

- [E] Supersede plain Thompson sampling with discounted/sliding-window non-stationary TS and change-point posterior reset.
- [E] Detect subdomain shifts such as Python gains masking Rust regression.
- [E] Run F-2/F-10 CWE security invariants as a mandatory blocking pre-promotion gate.
- [E] Wire, do not rebuild, a cross-family verifier.
- [E] Place behind ELAI_DARWINIAN_LOOP_ENABLE; extend proposal/mutation archive surfaces.
- [E] Security regression blocks despite fitness gain.
- [E] Stationary TS, same-family verifier, or skipped security suite must fail contract tests.

### J-21 — memory semantic-validation + poisoning hardening [citation-valid != semantically-valid] [L534-L552]

- [E] Bind stored code facts to commit plus AST identity and rederive them before injection; existence at file:line is not enough.
- [E] Add retrieval-time authorization, typed claim/evidence, and write quarantine so delayed poison cannot become trusted at retrieval.
- [E] Keep a provenance-linked raw-evidence tier and escalate to it when fact summaries are insufficient.
- [E] Replace fixed TTL expiry with activation decay so rare but important invariants survive.
- [E] Extend H-31/H-32/E-5, including auditor typed-evidence firewall and J-18 symbol identity.
- [E] Emit/reuse MemoryValidationEvent and show provenance in memory hub.
- [E] Refactor/behavior change makes a fact STALE; unresolvable identity becomes UNVERIFIED; unchanged facts inject normally; high-activation expired facts remain.

### J-22 — journal payload-schema lock + writer-isolation contract [the one new P0-journal item] [L554-L571]

- [E] Extend golden action-type-name lock to payload field shape.
- [E] Apply serde deny_unknown_fields on payload structs.
- [E] Snapshot field names/types while ignoring field order.
- [E] Add a contract test proving a single writer/sole commit point.
- [E] Do not migrate to Protobuf; preserve serde plus snapshot.
- [E] Add no action type; guard all existing payloads.
- [E] Field rename/type/addition and a second writer fail; order-only change does not false-fail.

### J-23 — multi-language review-gate catalog [Fallow is JS/TS-only; build the catalog] [L573-L591]

- [E] Declare Fallow JS/TS-only and build a language/tool catalog as DATA.
- [E] Tier 1 ingests SARIF 2.1.0 into gate verdicts from Semgrep CE, PMD-CPD, Psalm, and detekt, respecting CodeQL/paid cross-file licensing.
- [E] Tier 2 TOML declares invoke command plus parser mode json, exit_code, or text_regex for dependency-cruiser, Knip, Deptrac, and Periphery.
- [E] Gate ELAI's own Rust, Python, and JS/TS with cargo-modules/udeps/clippy, vulture/ruff, and knip/dependency-cruiser.
- [E] Add deterministic UI floor: token drift plus axe-core accessibility-tree checks; keep taste rubric advisory.
- [E] Start with cross-language SARIF core and grow only with demand.
- [E] Missing binaries are TOOL_MISSING, license-ineligible CodeQL is refused, and clean components pass.

### J-24 — non-prompt steering + context-frugal tools [answers the user's steering/tools questions] [L593-L612]

- [E] Activate the dark sidecar activation-addition steering endpoint so a vector shapes behavior without role-prompt tokens.
- [E] Build vector extraction and apply-on-generate, measured against an equivalent prompt.
- [E] Wire Tool Search and deferred MCP loading so schemas load on demand; state 'lazy/deferred', never 'zero tool tokens'.
- [E] Add deterministic BACM observation-size preassessment NONE/SELECTIVE/ALL before context insertion.
- [E] Exclude neural SWE-Pruner skimming; trigger existing compaction if size requires it.
- [E] Emit BehaviorSteeringEvent or reuse MCP-deferred rows; show steering/token budget in runtime/context UI.
- [E] Vector-off must revert behavior; no measured win claim before SAFEDELTA.

### J-25 — mimocode context+memory mechanics [L616-L639]

- [E] Add an 11-section checkpoint template with per-section budgets and dedup validation.
- [E] Section-aware reads always preserve the skeleton.
- [E] Before any trim boundary, keep tool_use/tool_result pairs and thinking siblings together.
- [E] Use a four-level pressure ladder as ContextPolicy trigger.
- [E] Coalesce digest writes newest-wins into one slot and freeze fork snapshots.
- [E] On agent return, deterministically downgrade completion to partial when open task rows remain; reject the LLM-judge half.
- [E] Add relative score floor and fingerprint reconciliation to memory retrieval.
- [E] Reuse HandoffLifecycleEvent; malformed orphan boundaries must be repaired into a valid Anthropic request.

### J-26 — sia evaluator ABI + task packs [L641-L657]

- [E] Run fitness evaluators as subprocesses with --gen-dir.
- [E] Evaluator writes results.json inside generation directory; exit 0 means it ran, not that a default score exists.
- [E] Missing results.json or nonzero exit is hard FAIL, never warning/default/SKIP.
- [E] Freeze evaluator files under R8, correcting Sia's missing freeze.
- [E] Standardize task packs: data/public task/evaluator/fixtures, data/private held-out, and reference.
- [E] Use packs for H-43 frozen corpus, E-54 CLI ingest, D-6 private rows, and Sia interop.
- [E] Reuse FitnessEvaluated; golden scalar must equal results.json.

### J-27 — harness protected-blocks + webcache + dispatch amendments [L659-L681]

- [E] PreToolUse(Read) replaces opt-in protected regions with content-hashed placeholders before model visibility.
- [E] PostToolUse(Edit/Write) restores protected bytes or blocks on mangled/missing placeholders; unannotated files are unchanged.
- [E] WebFetch cache revalidates with conditional HEAD; only validators permit storage and HTTP 304 counts as fresh verification.
- [E] Auditor input strips executor claims and contains artifact plus contract only.
- [E] Add NOTICED-BUT-NOT-TOUCHING, mandatory out-of-scope/rejection lines, and four-question orchestration fan-out checklist.
- [E] Import no external skill; add new hook wrappers and user-reviewed settings registration.
- [E] Mangle blocks writes; validator-less responses are not cached; second ETag fetch logs 304.

### J-28 — token ledger: usage unification + per-turn event + rollup core [L692-L716]

- [E] Fix streaming-local usage capture so MessageDelta produces nonzero input tokens.
- [E] Normalize usage while retaining semantic lane tags: local input is full prompt; cloud input is uncached suffix and cannot be naively summed.
- [E] Emit one UsageTurnEvent per turn from both lanes with identity, lane, backend, model, task, session, four counters, estimated cost micros, and elapsed milliseconds.
- [E] Price via model-specific models.toml rather than Sonnet rates for every model.
- [E] Make spend rollup session-scoped with Option semantics; tag estimator fallback.
- [E] Account for compaction and provide CLI dollar report plus elai usage grouped rollup.
- [E] No-usage streams record EstimateApplied, not zero; wrong lane sum or wrong model price fails.

### J-29 — usage window: dedicated surface + live mid-stream meter + canvas badge [L718-L741]

- [E] UsageWindow shows per model/task/session/lane, cache effectiveness, tool/file-operation counts, and per-turn data.
- [E] Resolve BackendCostAccrued hashes to names and consume the existing pricing rollup/gate honestly.
- [E] Show a non-journaled in-process mid-stream token gauge in chat plus CLI tick before turn end.
- [E] Local usage displays tokens, not a misleading zero-dollar value.
- [E] Derive autonomy ratio solely from journal arithmetic.
- [E] Add an E-40 canvas usage badge and absolute counts to ContextStatePanel.
- [E] Add Tauri commands and lockstep TS/Svelte types, but no SSB type.
- [E] Empty journal, unknown backend hash, and observability-off each have explicit noncrashing states.

### J-30 — provider byok: key store + custom endpoints + verified configure + turn lock [L743-L770]

- [E] Key lookup precedence is exact environment variable then restricted 0600/ACL file.
- [E] Save keys atomically with temp, rename, and fsync; OS keyring is optional and user-gated.
- [E] custom_providers.toml declares anthropic-compatible or openai-compatible engines behind B-37, including vLLM/LiteLLM/self-hosted URLs.
- [E] Generalize base URL while retaining ANTHROPIC_BASE_URL compatibility; probe unsupported request fields.
- [E] Descriptor-driven CLI and thin panel configure flow must make a live test call before save.
- [E] Runtime inventory shows configured, verified_at, models, and stale.
- [E] Turn-scoped backend lock rejects/logs a mid-sequence swap; sanctioned cloud escalation remains allowed between boundaries.
- [E] Store/event payloads never carry key material; dead endpoint saves nothing; missing key ref fails at load.

### J-32 — elai front-door router [chat = ELAI, not a model] [L779-L806]

- [E] Every message resolves to a closed-set lane or CLARIFY; CHAT is the safe default.
- [E] Tier 0 slash commands bypass; Tier 1 deterministic regex; Tier 2 local-3B LMFE label at temperature 0.
- [E] Tier 2 output is lane, confidence, and byte spans copied from user text—not paraphrased instructions.
- [E] Mutating lanes use keyboard-operable confirmation chip; LOW auto, MEDIUM chip, HIGH always confirm.
- [E] Autonomous mode approves once through GateMode and plan-approval card; high destructive actions still block.
- [E] Deterministic ACK card deep-links to canvas; progress uses compact journal-derived digests; RESULT card includes gate/file/evidence-row receipts.
- [E] Confirmation/rejection decisions become labeled calibration tuples.
- [E] Same intent as slash command must produce identical dispatch rows; Esc produces none; sidecar failure clarifies rather than guesses.

### J-31 — canvas system-composition mode [nodes are bound buildable units; edges are verified wiring claims] [L808-L837]

- [E] Add SYSTEM canvas mode where nodes are user-code components, not runtime flow steps.
- [E] Node types are PlanUnit, Contract, and McpService; binding is mandatory after DRAFT and state exists only in SSB rows.
- [E] Edge claims contain produces, consumes, reachable_from, stub_ban, and observable_effect.
- [E] Checker-derived ladder is declared, wired, verified, broken; LLM cannot set verdict.
- [E] Node lifecycle is DRAFT, FRAMED, BUILDING, WIRED, VERIFIED, DRIFTED.
- [E] MCP connector nodes show pin/re-pin and taint verdicts; do not recreate n8n integration treadmill.
- [E] Build dispatches bound unit through normal FSM, never a canvas-side runtime.
- [E] State rederives byte-identically; no evidence stays grey; deleted consumer or rug pull breaks/untrusts edges.

### J-33 — mineru document-parsing lane [wire the shipped-dark PDF backend] [L845-L873]

- [E] Activate existing dark ParserBackend::MinerU through a DocParserClient to MinerU FastAPI POST /file_parse.
- [E] Branch PDF/DOCX artifact indexing into structured parse, existing chunkers, ArtifactStage, and Cosmos.
- [E] Use parser output as deterministic first leg of E-36; VLM remains escalation.
- [E] Add pdf_ingest.py so paper ingestion converts real PDFs to markdown before MCP ingest.
- [E] Pipeline backend is CPU-capable Tier 1; 1.2B VLM/8GB VRAM is remote Tier 2.
- [E] Default-off config includes allowlist and promotion plan; external service dependency requires confirmation.
- [E] Require Apache-2.0-era MinerU attribution/model-card verification; ban Flash-MinerU AGPL and vendored forbidden code.
- [E] Service-down yields ParseFailed and fallback recommendation, never garbage indexing; unconfigured behavior remains today's behavior but is surfaced.

### J-34 — empirical context calibration [measured curves replace folklore thresholds] [L875-L906]

- [E] elai bench context-fill runs verified fixture tasks at stepped fill levels with deterministic real-repository padding.
- [E] Score by gate outcomes, never LLM judgment; lorem padding is invalid.
- [E] Produce per model/lane degradation curve and knee.
- [E] Extend ModelCapabilityProfile with fill_degradation_curve, calibrated_trim_fraction, and positional_value_curve.
- [E] Prefer measured trim fraction over hardcoded 0.85.
- [E] Extend five-position rot probe to depth-stepped constraint recall and use measured positional value to order 3E slots.
- [E] Keep layout fixed per model at calibration time to preserve cache stability.
- [E] Key artifacts by model identity; G-10 drift marks stale; add J-28-keyed regression later.
- [E] Missing artifact logs fallback to existing constants; per-call layout churn and LLM-judged curves fail.

### J-35 — intent-control layer [the user>elai>model forward leg: typed spec + premise-check + route + decision-card] [L916-L957]

- [E] Distinguish fidelity from aim; aim is private and cannot be 'verified', so surface and confirm rather than gate it.
- [E] Reconstruct one visible low-entropy IntentSpec before decomposition and retrieval: goal, constraints, hidden assumptions, non-goals, validation contract.
- [E] Reuse existing BabyStepSpec/IntentPair/WiringContract shapes.
- [E] Force a format-independent premise subpass, deterministic-first against DKG and grounding verifiers.
- [E] Ask only if information improves execution and action is costly/irreversible.
- [E] Autonomous defaults come from revealed SSB outcomes, never population majority; assumptions are journaled; high-risk still escalates.
- [E] Decision card shows goal, assumptions, and forks, not raw reasoning.
- [E] Premise/aim rules self-improve through existing SHADOW to ACTIVE lifecycle at sparse measured thresholds.
- [E] Observe-only behind ELAI_INTENT_CONTROL_ENABLE with allowlist/promotion; premise contradiction halts decomposition until confirmed.
- [E] Hidden goal rewrite, majority default, format-conditioned checking, or an 'aim verified' verdict fails.

### J-36 — freeze-the-head local prefix-KV reuse [keep the system+rules+anchor prefix hot+cached on the STREAMING path, minimize only the tail] [L959-L988]

- [E] Key the stable system/rules/anchor head at the existing marker boundary with blake3.
- [E] Extend PrefixKVCache lease and proven crop logic to streaming generation.
- [E] On head hit, prefill only fresh tail; lease head even when total input exceeds one chunk.
- [E] Never trim, reorder, or unload frozen head; local context minimization applies to tail.
- [E] Eligibility predicate reads model profile; TurboQuant, unknown architecture, and sliding-window models are ineligible and cold-prefill with a reason.
- [E] Use J-34 measured tail budget.
- [E] Emit HeadCacheHit/Miss, TailReprefill, or CacheIneligible; no dark flag because conservative eligibility is the gate.
- [E] Hit on an ineligible model fails; TurboQuant coexistence remains deferred.

### J-37 — STALE staleness adjudication [state-schema propagation + 4-state verdict + UNKNOWN-blocks-premise, extending H-31/E-5] [L990-L1025]

- [E] Hard-depend on a real production consumer of crates/memory; stop if store is wire-dark.
- [E] Define TOML state schema Omega mapping domain, slot, and depends-on.
- [E] Candidate conflicts are direct, dependency-affected, and global-k, not exact-key only.
- [E] Verdicts are KEEP, STALE, REPLACE, UNKNOWN; tier and deterministic NLI are inputs.
- [E] KEEP/REPLACE may ground; STALE is history-only and never reinjected; UNKNOWN blocks use as premise.
- [E] UNKNOWN routes to D-16 conformal abstention.
- [E] Extend contradiction/retrieval and add STALE corpus metrics for state resolution, premise resistance, and implicit-policy adaptation.
- [E] Equal-tier silent coexistence, STALE prompt injection, confident UNKNOWN answer, or missed propagated dependency fails.

### J-38 — agentic retrieval loop [mid-task callable hybrid search + SpIDER graph-neighbor fusion + context-sufficiency gating] [L1027-L1060]

- [E] Expose hybrid_search(query, top_k, mode) as an executor-callable mid-reasoning tool.
- [E] Use FAST tier with optional reranker for latency.
- [E] Expand top-K through DKG call/import neighbors up to depth 2, then rerank.
- [E] Deterministic context sufficiency compares symbol coverage from IntentPair plus surfaced symbols.
- [E] Search only when below threshold and enforce maximum rounds/no-progress cap.
- [E] Keep amortized pre-task retrieval; loop is additive and unbounded querying/LLM sufficiency are forbidden.
- [E] Emit/reuse retrieval round, graph expansion, sufficiency, and cap events; show rounds in KnowledgeDrawer.
- [E] Above-threshold context triggers no search; empty neighbors add no rank filler; disabled loop must miss the seeded span.

### J-39 — decomposition node-admission gate [bound-and-escalate sizing against the measured ceiling] [L1062-L1078]

- [E] Model proposes node; deterministic gate disposes using B-29 cone input lower bound against J-34 per-model/lane ceiling.
- [E] Decisions are Admitted, SplitForced via community re-split, or EscalatedToCloud.
- [E] Output limit is only a circuit breaker, not a predictive gate.
- [E] A cross-cutting atomic-intent escape avoids pointless splitting of a huge cone.
- [E] First run EXP-1 observe-only to test cone size versus spend; do not predict total output cost.
- [E] Emit one NodeAdmissionEvent per node with decision, cone_token_floor, lane_ceiling, and lane.
- [E] Fit does not prove local capability; escalation remains load-bearing.

### J-40 — research-modes extractive compression [model selects / harness fetches+compresses; speed tiers] [L1080-L1095]

- [E] Model formulates queries and ranks pages only; harness fetches in parallel and extractively compresses.
- [E] Keep about 200 verbatim tokens per source; never generatively summarize on this path.
- [E] Raw page bytes never enter reasoning context; serialize step state externally.
- [E] Extend A-RAG snippet extraction with query-aware web extraction.
- [E] Offer FAST/STANDARD/DEEP over FlashResearch.
- [E] Replace StubSearchBackend with SearXngBackend already REPL-wired.
- [E] Exclude full Search-as-Code and KV reuse as default mechanisms.
- [E] Reuse QueryTrace with per-source compression ratio and ResearchMode; add no event.

### J-41 — cheap grounding stack [three-tier extractive -> small claim model -> closed-set citation] [L1097-L1115]

- [E] First run deterministic extractive fingerprint matching: substring, fuzzy, then semantic; meaningful units are sentence or n-gram at least 5.
- [E] Activate F-12 ONNX judge using MiniCheck/Paladin-like binary-grounding adapter rather than treating it as generic 3-class NLI.
- [E] Add ClosedSet decoding over frozen EvidenceAnchorSet; invalid citation tokens receive negative infinity.
- [E] Out-of-set or unsupported claims route to D-16 abstention.
- [E] EXP-2 uses 100 balanced ELAI code-domain source/claim pairs and requires measured transfer before authority.
- [E] Exclude token-membership hash, NLI-only gate, and forced citations without refusal.
- [E] Reuse CitationAnchorsBound; unsupported or ghost citation cannot pass.

### J-42 — within-step distill-reset reasoning [InftyThink sawtooth within one GRC/planning/research step] [L1117-L1145]

- [E] Within one reasoning/planning/research step, model produces a segment and proposes a compact free-form distillate.
- [E] Harness, not model, triggers checkpoint on context fill or GRC budget.
- [E] Frozen admission requires bounded size, nonempty content, and preserved uncertainty.
- [E] Reset context and reseed pinned original goal plus admitted distillate; extend reset_topic into checkpoint_and_reset.
- [E] Add reset-mode to deep/local B-39 auditor and PipelineGRC paths; cloud keeps cache/append mode initially.
- [E] Failed admission escalates or continues per policy; hard tasks can go to cloud.
- [E] EXP-3 compares equal-budget straight-line and distill-reset across local models/tasks, including abstention quality.
- [E] Emit trigger, distillate tokens, uncertainty flag, pass index, and Reset/Reject/Escalate outcome.

### J-43 — doctrine-propagation keystone [Track 1, the LENS: meta-rule + thin frozen floor] [L1156-L1174]

- [E] Add determinism-doctrine hard rule containing deterministic subset, D1-D5 qualitative catalog, unknown=FAIL, and auditor ownership.
- [E] doctrine_floor_check detects gate reading own output, skip on missing evidence, same-family critic as block candidates; LLM verdict on gate and constrained reasoning warn initially.
- [E] Exit 2 on missing targets; observe-only until per-class false positives are bounded.
- [E] Add doctrine debt ledger; a marker downgrades only if backed by a real row and cannot become a graveyard.
- [E] Auditor reruns floor off-agent and freeze-checks manifest; wrap rather than edit existing gates.
- [E] A run editing its own doctrine gate blocks; optional DoctrineDebtEvent only.

### J-44 — reasoning-method artifact-shape [Track 2, the METHOD: shape gate + the substance audit] [L1176-L1197]

- [E] Generalize reasoning-method rule to all artifacts and require M2/stance/M7/M10-style sections and load-bearing claim tags.
- [E] Shape checker requires mandated sections; missing ATTACK or untagged claim fails.
- [E] Add a random held-out substance audit by a non-author because shape-complete boilerplate can pass mechanically.
- [E] Track remediation velocity and boilerplate rate.
- [E] Inject decision ledger and doctrine pre-gates into spec-eval, planning, idea intake, and dispatch seams.
- [E] Never use LLM quality score on a gating path and never equate section presence with quality.
- [E] Demonstrate that empty-but-shaped artifact passes checker and is caught only by the panel.

### J-45 — cross-family advisory calibration [the D5 leg: calibrated, with escalation teeth] [L1199-L1219]

- [E] Serialize boundary/proxy snippets to a different model family with no repository access.
- [E] Advisory vote never grounds a critical path.
- [E] Build 100-300 trace human-labeled calibration corpus; require Cohen kappa over 0.6, below 0.5 unusable.
- [E] Uncalibrated advisory is explicitly decorative and does not satisfy D5.
- [E] Findings escalate to orchestrator/user handoff decisions, not a passive dashboard.
- [E] Same-family dispatch is refused.
- [E] Cross-family unavailability records single-family-degraded doctrine debt instead of pretending independence.

### J-46 — doctrine-debt-rate measurement [D4 turned on the propagation itself] [L1221-L1243]

- [E] Compute doctrine debt rate as open debt rows divided by reviewed decisions, per subsystem over time.
- [E] Track remediation velocity, not a score or bare row count.
- [E] Sweep WIRED features first, seed the J-45 corpus, and burn down full backlog later.
- [E] Stamp every feature-map upsert clean, debt(id), or unreviewed; unreviewed counts as debt.
- [E] Flag implausible zero debt at nontrivial review volume as suspect.
- [E] Judge delta, never leaderboard rank; fix denominator definition for comparison.
- [E] Output must be recomputable off-agent; no new gate or SSB.

### J-47 — program falsifier + per-axis discipline [M9 at the program altitude; V5/V4] [L1245-L1267]

- [E] Run 20-50 small-scope A/B/C experiments: hybrid doctrine, no doctrine, and simulated comprehensive scorer.
- [E] Measure human-audited discretion leaks, bug density, and time to ship.
- [E] Falsify if hybrid does not lower leaks versus no-doctrine while costing less than comprehensive scorer.
- [E] Nothing becomes blocking-default-on before EXP-DP5 passes.
- [E] New axis doctrine requires a recorded failure and live axis count is capped.
- [E] Existing decomposition/research/grounding/reasoning doctrine is not reauthored; possible new axes are memory, routing, eval, UX, security, and meta.
- [E] Leak audit itself must be non-author/cross-family.

### J-48 — local serving tier router [adopt-the-engine; the coarse outer policy only] [L1271-L1297]

- [E] Decide among ServeLocal, AdmitQueued, HibernateThenServe, and SpillToCloud before RuntimeOrchestrator completion.
- [E] Use named per-tier concurrency caps and warm/cold whole-model lifecycle; pin small models.
- [E] Treat 32B as exclusive mode on the cited 24GB topology and permit tensor-parallel use where configured.
- [E] Use coarse FIFO with no more than three priority classes.
- [E] Busy/cold spill reuses I-7 and existing cloud cost controls.
- [E] Explicitly leave continuous batching, chunked prefill, KV paging, token preemption, and MLFQ to vLLM.
- [E] Reuse RouteDecisionEvent and J-29 read-side.
- [E] Empty spill reason or serving above cap fails; multi-resident contention smoke is remote-gated.

### J-49 — multi-seat admission + per-seat quota / fair-share / spend isolation [the documented "revisit only if multi-tenant" trigger firing] [L1299-L1321]

- [E] Presence of seats.toml activates multi-seat; absence preserves one local seat with no new enable flag.
- [E] Put a weighted token-bucket fair-share filter before J-48 so one seat cannot starve another.
- [E] Reuse C-5 spend caps keyed by identity for per-seat ceilings.
- [E] Aggregate existing UsageTurnEvent identity into per-seat raw token and USD rollups.
- [E] Do not create synthetic credits, dynamic model registry, new cap engine, or MLFQ.
- [E] Take seat identity as input; authentication provenance remains J-2's concern.
- [E] J-29 supplies UI breakdown; over-cap seat queues/denies while under-cap seat serves.

### J-50 — recovery-trajectory sft exhaust [EXTENSION of I-30: failed-then-recovered chains, not winning-paths-only] [L1325-L1346]

- [E] Extend I-30 export to include failed-then-recovered chains, not only winners and unrecovered negatives.
- [E] Select error/veto then corrective continuation then promoted verdict from sealed RRAs.
- [E] Emit label=recovery JSONL with the same provenance hash.
- [E] Target recovery fraction 0.35 with 0.30-0.40 range.
- [E] Never synthesize or relabel to meet target; under-target is a warning with actual fraction.
- [E] Do not add DPO/GRPO or duplicate the walker/writer/hash; no conflict with inference-time I-5 exclusion.
- [E] Extend SftExhaustEvent kind and dataset card fields; zero recovered runs must emit fraction zero, not fabricate.

### J-51 — tool-call adherence floor + role gate [measure per-local-model SEMANTIC tool-call reliability under the grammar mask] [L1348-L1375]

- [E] Benchmark semantic tool selection and arguments under the existing XGrammar format mask.
- [E] Score against deterministic tool/argument ground truth, never LLM judge.
- [E] Write per-model tool_call_adherence into ModelCapabilityProfile.
- [E] Advisory role gate prefers higher-adherence models for tool-heavy roles; sub-floor can spill through I-7.
- [E] Default-on behavior is disabled by ELAI_TOOLCALL_ADHERENCE_GATE_DISABLE; do not add an enable flag or hard kill.
- [E] Do not remeasure JSON format validity or bypass the mask.
- [E] Emit AdherenceMeasured, RoleAdvised, FloorBreached and show per-model tile.
- [E] Missing artifact must log fallback; disabled gate must not be silent.

### J-52 — return-time + per-edit enforcement hooks [SubagentStop evidence gate + per-edit LSP daemon block-feed + path-keyed rule injection] [L1377-L1402]

- [E] At subagent return, block DONE unless it contains a real nonempty off-agent artifact.
- [E] Harden evidence paths against realpath, symlink, and traversal tricks.
- [E] Retry up to bounded MAX_EVIDENCE_NAGS, then escalate; stdout alone never suffices.
- [E] Keep auditor authoritative: evidence gate proves artifact existence, not correctness.
- [E] Feed LSP daemon diagnostics after every edit through PostToolUse, extending E-52 from batch audit.
- [E] Reuse daemon; transient diagnostics are not recorded, permanent diagnostics are.
- [E] Inject path-keyed conditional rules into context Rules slot, deduped and priority-sourced.
- [E] Add HookEnforcementPanel and either three events or one collapsed event.
- [E] Daemon unreachable must not be recorded as a valid diagnostic; rule double-injection and silent disabled gates fail.

### J-53 — OpenAI-compat tool-input repair [the unmasked-decoder seam the local XGrammar mask cannot cover; cloud today, any local OpenAI-compat backend later] [L1406-L1443]

- [E] Scope repair to unmasked OpenAI-compatible parsing; do not touch decoder-controlled XGrammar local path.
- [E] Valid arguments ship byte-identically unchanged.
- [E] For invalid input, walk tool input schema and apply ordered repairs: omit null, parse stringified array, unwrap object, wrap bare string.
- [E] Add typed-path Markdown auto-link unwrap.
- [E] Apply offset/limit relational defaults at dispatcher and tell the model/tool result about the default.
- [E] Unrepairable input becomes a model-readable error then uses E-18 one-shot retry; remove silent raw-blob behavior.
- [E] Avoid greedy preprocessing that corrupts JSON-shaped writeFile bodies; no new dependency unless separately approved.
- [E] Emit provider/model repair rate; disabling repair reproduces and logs legacy passthrough, not a hidden repair.

### J-54 — free-API rotation pool [user-assignable-per-role FreePool backend; Liberty's rotation/quota/heal ported natively; verification-lane trust-marker] [L1447-L1474]

- [E] Add FreePool RuntimeBackend assignable per role through existing PerRoleRuntime.
- [E] Persist per-key/model daily quota, honor 429 cooldown headers by priority, sleep until reset when whole pool is exhausted, rotate multiple keys, and allow keyless overflow.
- [E] Hand-roll Rust provider/catalog/router/quota/heal; do not introduce LiteLLM.
- [E] Key fingerprints use blake3; raw keys never enter state/logs.
- [E] Keep model catalog static in models.toml; reject dynamic sync and Python g4f tier.
- [E] If executor/auditor uses free pool, mark RRA non-reproducible and suppress G-10 canary with notice.
- [E] Respect provider groups; no cross-group fallback for uncensored group.
- [E] RuntimePanel chooses pool and shows trust downgrade; quota state lives under user ELAI state.

### J-55 — model-strength modality router [KEYSTONE of win-both: route the action surface (CodeAct vs constrained tool-call) per model strength] [L1476-L1505]

- [E] Deterministically choose ActionModality by model profile, role, and tier.
- [E] Use CodeAct for capable lanes.
- [E] Use XGrammar-constrained single-tool calls with compiled minimal schemas for weak/sub-14B local lane.
- [E] Use SEARCH/REPLACE diff universally when applicable to reduce output tokens.
- [E] Progressive schema disclosure reuses Tool Search/MCP defer.
- [E] elai bench modality measures the per-model/lane crossover by deterministic verdicts and writes ModelCapabilityProfile.
- [E] Do not route via LLM or folklore threshold; do not send full registry to weak lane.
- [E] A Qwen-class 7B choosing CodeAct or a whole-file write where a diff applies fails the keystone guard.

### J-56 — living eval corpus + failure-ingest + CI governance [ACTIVATION of the dark E-54 + never-invoked G-6] [L1516-L1531]

- [E] Add elai eval ingest from a caught session failure into a categorized corpus row.
- [E] Seed problem taxonomy from failure_taxonomy.py.
- [E] Add elai eval run --scope subset or full; subset on change, full on merge.
- [E] Score only deterministic gate outcomes, not LLM judgments.
- [E] Reuse E-54 13-gram contamination filter, D-6 held-out boundaries, SAFEDELTA data, and J-16 cone subset selection.
- [E] Emit RowIngested, RowRejected, SuiteRun and reuse SAFEDELTA UI.
- [E] Held-out overlap, uncategorized input, silent subset-to-full expansion, or LLM scoring fails.

### J-57 — eval-gated prompt-template promotion + rollback [prompts-as-code: test->staging->prod over the shipped hash-chain] [L1533-L1548]

- [E] Add PromptStage state machine Test, Staging, Prod over existing hashed/versioned stores.
- [E] Promotion runs J-56 subset and advances only on no regression against current Prod.
- [E] G-10 drift auto-rolls back to last-good prompt hash.
- [E] Deterministic eval verdict owns promotion; no 'better prompt' LLM judge.
- [E] Add CLI elai prompt promote --to and stage/provenance badge.
- [E] Test-to-Prod skip, regressing Prod, or rollback without recorded last-good hash fails.

### J-58 — pre-execution sprint-contract negotiation loop [the one Anthropic-talk mechanism I-21 lacks] [L1550-L1568]

- [E] Before I-21 approval, generator and isolated evaluator iteratively re-propose the done contract.
- [E] Contract criteria may only tighten and should become more granular.
- [E] Bound rounds and escalate at cap to D-13 adversary.
- [E] Pass sealed contract plus structured objection only; evaluator never sees generator scratchpad.
- [E] Reuse SprintContractEvent with Renegotiated arm and UI round/criteria delta.
- [E] Execution cannot start without Approved; loosening, unbounded rounds, or scratchpad leakage fails.

### J-59 — per-seat memory + trust isolation [the multi-seat data-foundation/governance slice J-49 deferred] [L1570-L1587]

- [E] Namespace memory by J-49 seat identity across capture, retrieval, facts, and trust tier.
- [E] Hard-isolate reads, writes, and poisoning across seats.
- [E] Extend auditor-objectivity firewall by seat and role; auditors receive zero memory.
- [E] Isolation-first v1 forbids cross-seat sharing.
- [E] Reuse MemoryGovernanceEvent with seat discriminator but no memory content/secret.
- [E] Defer per-seat audit-view UI; backend contract is the proof.
- [E] No seats.toml must preserve single-seat behavior exactly.

### J-60 — layered observability canvas [the WATCH half: ELAI's own FSM + agents as a journal-derived graph; per-agent worktree tabs] [L1591-L1596]

- [E] Add PROCESS altitude showing ELAI FSM states and agent-pool workers as journal-derived nodes.
- [E] Per-agent worktree tab is a filtered journal view reached by semantic zoom, never independent state.
- [E] Until I-2, derive worktree diff from H-49 op stream.
- [E] Surface is emit-only/read-only and uses deixis to descend, avoiding node soup.
- [E] Reuse spawn_view to open tabs and add process layer renderers.
- [E] Two-agent run must rederive byte-identical graph; model-sourced status fails and zero-row agent renders nothing.

### J-61 — canvas redirect loop [the STEER half: edits are journaled intents, the LLM enacts, breakage shows deterministically, the user preempts] [L1598-L1603]

- [E] Do not hand-author/add nodes in canvas; new nodes originate through chat.
- [E] Closed verbs are CONNECT, CUT-EDGE, REORDER, REBIND, and AGENT-CONTROL.
- [E] Each gesture becomes CanvasRedirectIntent to the single SSB writer; total order is conflict model, with no locks/CRDT.
- [E] LLM proposes wiring while J-31 checker/Kahn derives verdict and breakage.
- [E] Translate across altitudes and allow user to preempt same-node in-flight LLM action with rollback.
- [E] Queue semantics on collision, direct mutation, LLM verdict, or ADD-NODE attempt fails.

### J-62 — environment-constraint skill cards + pre-write mandatory-read gate [the Fable mandatory-env-skill-read steal: author the missing env-card, gate it before the write] [L1605-L1610]

- [E] Create dashboard-v2 environment skill for actual Svelte 5, Vite, and Tauri v2 constraints, with positive/negative evals.
- [E] Add per-surface registry.
- [E] Reuse J-52 PathRuleInjector; extend source class to skills and inject before first matching write.
- [E] Card enters context Rules slot and deduplicates; already-resident and no-match are explicit outcomes.
- [E] Do not reimplement J-52 resolver or restate taste/visual-loop docs.
- [E] React-framed card, write without injection, or silent disabled gate fails.

### J-63 — local-lane Claudeception [the Fable keyless-AI-artifact steal, local-keyed: ELAI-generated apps run AI-powered on the user's own model] [L1612-L1618]

- [E] Hypothetical generated apps call a keyless loopback-only broker that forwards through RuntimeOrchestrator/J-48, not a second inference path.
- [E] elai artifact run launches an app under Tier C and injects ELAI_GENERATED_APP_BASE_URL.
- [E] Broker contains no cloud key; any cloud request opens existing trust dialog.
- [E] Apply Tier-C filesystem isolation, G-1 taint screening, and J-48 quota.
- [E] Emit broker, taint, quota, cloud-gate outcomes and add GeneratedAppPanel.
- [E] Non-loopback bind, ungated cloud, FS escape, secret prompt, or unthrottled runaway fails.
- [A] Capability fit is explicitly unresolved; decide before build and require security review.

### J-64 — ACP transport for the agent bridges [the goose transport edge, additive: one ACP client reaches every ACP agent without dropping ELAI's broader, more-isolated direct-CLI coverage] [L1622-L1627]

- [E] Keep existing direct vendor CLI adapters and authentication; no auth gap is claimed.
- [E] Add ACP as an optional RuntimeAdapter transport over an isolated spawned ACP adapter.
- [E] Reuse the same Tier-C subprocess and CliSubprocessSpawned event.
- [E] Deny ACP bidirectional file/permission callbacks by default; jail them and use trust dialog for escalation.
- [E] Per backend choose direct-cli or acp; direct CLI remains default; ELAI_ACP_TRANSPORT_DISABLE is kill switch.
- [E] Add agent-client-protocol dependency only with user confirmation.
- [E] Adapter absence fails loudly without direct fallback; callback outside jail must be denied and logged.

### J-65 — power-of-ten generated-code gate [HEADLINE; the user's "NASA-grade generated code"] [L1639-L1644]

- [E] Frozen auditor script checks flagged safety-critical generated code for acyclic call graph, statically bounded loops, assertion density, checked returns, and zero warnings.
- [E] Rust/Python/TS mappings are approximations; binary P10 rules 4/7/10 block while approximate 2/9 warn.
- [E] Scope to modules marked P10-SAFETY-CRITICAL; do not globally tighten maintainability rule.
- [E] Observe-only until false positives bound hard subset.
- [E] Reuse B-29 call graph and freeze manifest; no runtime/SSB.
- [E] Unparseable changeset fails rather than skips; nonflagged 70-line function does not block; auditor rerun is authority.

### J-66 — SSB temporal-monitor engine [EXTENSION(I-35); the compilation algorithm I-35 lacked] [L1646-L1651]

- [E] Compile past-time LTL monitors into constant-work recurrences on each committed SSB row.
- [E] Initial properties include write-after-allow and no-egress-after-widen; do not rebuild F-4 test/stdout temporal gate.
- [E] Rewrite future-time session obligations and evaluate at MEMORIZING-to-COMPLETE.
- [E] Obligations include every agent returns and every blocking finding closes.
- [E] Undischarged obligations block completion.
- [E] Feasibility lint rejects wall-clock/liveness predicates at authoring.
- [E] Refuse to run on a J-69 integrity-failed journal; emit PastTimeViolation, ObligationUndischarged, FeasibilityRejected.

### J-67 — multi-mode gate actions [EXTENSION(J-12); Block -> {Accept,Suppress,Insert,Block}] [L1653-L1658]

- [E] Extend boolean intervention into Accept, Suppress, Insert, Block, with Edit represented as suppress plus insert.
- [E] Suppress drops dangerous call and continues.
- [E] Insert adds mandatory approval/declassification before side effect.
- [E] Edit can redact a secret from tool output.
- [E] Every suppression/insertion/edit is journaled or replay contract fails.
- [E] Deterministic enforceability classifier chooses legal action; model never does.
- [E] Disable flag falls back visibly to block-only; silent suppression or edit on nonsafety policy fails.

### J-68 — formal verification of ELAI's own critical code [Kani + SPARK contracts + TLA+] [L1660-L1665]

- [E] Add Kani proofs for serializer roundtrip, FSM no-panic, fail-closed predicates, parser no-panic, and counter no-underflow.
- [E] Counterexample or missing harness blocks.
- [E] Add SPARK-style pre/post contracts and absence-of-runtime-error discipline for privileged supervisor functions; optional Creusot.
- [E] Add ElaiFSM.tla/TLC invariants over nine-state FSM; privileged FSM change requires spec delta and passing TLC.
- [E] Treat bounded model checking as complement to DST, not proof beyond its bound.
- [E] Dependencies require user confirmation; auditor CI rerun/freeze is authoritative.
- [E] Vacuous zero-state TLA spec and HARNESS_NOT_FOUND fail.

### J-69 — SSB log-integrity hash chain + watermark monotonicity [EXTENSION(I-18 + J-22); W12 confirmation] [L1667-L1672]

- [E] Reuse I-18 row-chain producer and J-22 writer isolation; do not author competing chain.
- [E] At session open and rra verify, scan prev_hash=blake3(previous canonical bytes) before replay.
- [E] Enforce new-session watermark not below prior committed watermark.
- [E] If I-18 is absent, minimally add chain_prev_hash with DDL epoch bump, no action type.
- [E] One-byte mutation, reordering, truncation, absent chain field, or regressed watermark blocks.
- [E] Show chain integrity in RRA surface.

### J-70 — gate-manifest traceability schema + IV&V independence gap-closure [DO-333 + NASA-STD] [L1674-L1679]

- [E] Every frozen-gate manifest row declares objective_covered, technique, evidence_artifact, and satisfaction REPLACE/AUGMENT/PARTIAL.
- [E] Missing field and UNKNOWN result fail gate_freeze_check.
- [E] Add --trace requirements-to-gate-to-evidence matrix.
- [E] Reserve auditor budget pool that orchestrator cannot raid.
- [E] Narrow H-10 audit-skip exception for safety-critical changesets while retaining scoped nonsafety skip.
- [E] No SSB/UI beyond report; orchestrator raid or safety-critical skip is blocked and logged.

### J-71 — policy-as-code spike (Cedar) [SPIKE-FIRST; decision deferred, may be declined] [L1681-L1686]

- [E] Time-box one-agent read-only spike; do not commit to adoption or production wiring.
- [E] Reexpress one capability gate using principal=role, action=tool/transition, resource=file/SSB scope, context=FSM state.
- [E] Demonstrate SMT policy-equivalence across a refactor.
- [E] Measure authoring cost, SMT runtime, expressiveness fit, and second-source-of-truth cost.
- [E] Report keep Python, adopt Cedar, or decline.
- [E] Add no dependency, .elai_cc change, SSB, or real gate; premature test scaffold is removed or treated as scaffold.
- [E] Adoption recommendation missing second-source cost or unverified dependency fails the spike contract.

### J-72 — in-domain retriever adaptation [KEYSTONE] [L1702-L1707]

- [E] Fine-tune exactly one retriever offline on ELAI issue/PR/edit and sealed-RRA traces.
- [E] Variant A adapts 768-dimensional encoder with length-weighted contrastive training and folds into J-18 migration.
- [E] Variant B distills an approximately 2M-parameter scoring head to replace generic cross-encoder.
- [E] Export ONNX; CPU inference/hot path has zero LLM calls and unchanged architecture.
- [E] Build deterministic ELAI retrieval gold set and bench MRR/Recall; paper headline numbers are explicitly unverified.
- [E] Do not add in-product training or community quant.
- [E] Dimension/geometry/shape mismatch fails; missing ONNX degrades to stock slot.

### J-73 — retrieval-abstention gate [L1709-L1714]

- [E] Add should_retrieve before the cascade.
- [E] Repurpose calibrated abstention score with per-task threshold near query-tier router; model does not adjudicate.
- [E] Local sidecar special-token head is a remote-gated Tier-2 upgrade.
- [E] Above threshold can skip retrieval; uncertain/below threshold always retrieves.
- [E] Observe-only first records would-skip until false-positive precision is bounded.
- [E] Emit/reuse QueryTrace decision and optional KnowledgeDrawer skip rate.

### J-74 — graph-as-retrieval channel + mid-task action [L1716-L1721]

- [E] Wire shipped AST/dataflow edges into E-40 RRF as an additional channel.
- [E] Expose B-29 k-hop ego graph through J-38 mid-task callable retrieval.
- [E] Do not build new graph construction; RRF k remains 60.
- [E] Serialize flat context within depth 2 and budget cap.
- [E] Empty/irrelevant neighbor channel contributes nothing, never rank-k filler.
- [E] Reuse E-40 QueryTrace and J-38 event kinds.

### J-75 — RRF ablation harness (measure-then-simplify) [L1723-L1728]

- [E] Add bench-only flags to drop PageRank, second dense, or nav-only graph variants.
- [E] Reuse J-72 gold set and output per-config MRR/Recall@5 delta.
- [E] Never include cross-encoder in prune set.
- [E] Harness measures only; production config does not change.
- [E] Any removal needs a separate user-confirmed decision citing measured report.
- [E] Reuse existing bench report/SSB and UI.

### J-76 — context-rot calibration + local _compact_history recipe [L1730-L1735]

- [E] Cap retrieval chunk count, place valuable chunks at measured head/tail, and prefer focused context to full dump.
- [E] Consume J-34 positional-value curve rather than imported paper thresholds.
- [E] For local lane at 85% window, summarize while preserving intermediate numerics and rebuild system+initial+summary+continue.
- [E] Cloud lane remains deterministic masking and does not generatively summarize cache prefix.
- [E] Model without measured curve falls back conservatively.
- [E] SproutRAG hierarchical attention tree is deferred.
- [E] Reuse ContextCalibrationEvent/compaction path and J-29 ContextStatePanel.

### J-77 — perception faithfulness via the trust spine [L1749-L1754]

- [E] Bind each described visual region to deterministic detection using H-53 citation prebinding; ghost region becomes unverified/refusal.
- [E] Add G-1 taint source per element.
- [E] Emit layout as DKG-shaped graph for text-only reasoner.
- [E] Record canonical-frame transforms in RRA-style ledger and reject coordinate drift.
- [E] Fuse typed merge graph with one authoritative owner for counts; captions never own count.
- [E] Add tile size, 12.5% overlap, and owner-tile data; strict schema includes source/confidence.
- [E] Evaluate candidate VLM/OCR models under hardware and anti-slopsquat gates; use dual-OCR refusal.
- [E] Drop unverified instruction-before-image folklore.
- [E] Replace abstract gate with six-metric fixture; duplicate-region over 1% or count accuracy under 90% fails config.

### J-78 — context-access spec (the Conductor access_list, formalized) [L1756-L1761]

- [E] For every dispatch, orchestrator declares which prior SSB rows, sibling outputs, and memory may be read.
- [E] Model cannot choose access; I-5 failed-sibling exclusion and H-31 zero-memory auditor become explicit cases.
- [E] Context builder consumes spec only and journals it for exact RRA reproduction.
- [E] Missing/empty spec yields minimal safe context, never full dump.
- [E] Out-of-spec reads are first observed then blocked.
- [E] Reject learned/latent hidden-state routing; text-level portable spec is the feature.

### J-79 — signal-triggered grill interlace [L1763-L1768]

- [E] Keep grill consent: automation may offer but never auto-run it.
- [E] Clarify rung 0 uses existing chip for one missing slot.
- [E] Rung 1 offers grill when B-40 blocks, B-38 ambiguity is high, or C-29 has fewer than three intent pairs.
- [E] Accepting grill feeds resolved decisions back to lower ambiguity, supply pairs, and clear block.
- [E] Autonomous escalation names grill for ambiguity-grade causes.
- [E] Start with strongest B-40 trigger; widen only after false-offer rate bounded on at least three real runs.
- [E] Reuse IntentRouted calibration tuple; clear task proceeds after decline.

### J-80 — worktree pool lifecycle [EXTENSION(I-2); MAP FIX I-2 PLANNED->PARTIAL] [L1784-L1799]

- [E] Keep existing deterministic barrier merge, escape check, and Windows prune behavior.
- [E] Derive persistent pool-state projection from SSB, never external JSON.
- [E] Lease owner identity includes PID and started_at to resist PID recycling.
- [E] On acquire/list, heal dead-owner or vanished entries and never heal active leases.
- [E] Warm-reset and reuse worktrees.
- [E] Wire orchestration workers to acquire distinct roots and unlock J-60 tabs.
- [E] Dead owner self-heals; conflict/missing merge escalates; reset-hard and Treehouse JSON are forbidden.

### J-81 — autonomous supervision hardening [EXTENSION(auto-orchestrate + E-39); deterministic, no new subsystem] [L1801-L1815]

- [E] Detect process wedge when output signature is unchanged for N cycles, distinct from reasoning anti-spiral.
- [E] Persist a wake queue across supervisor crash, composing singleton lock and coalesced signals.
- [E] Wire dark E-39 into mid-loop task runner to abort current step on deterministic cost, iteration, or wall-clock cap.
- [E] Existing orchestration envelope counts dispatches; new caller controls mid-run token/cost.
- [E] Never use LLM self-judged stopWhen.
- [E] Add harness log evidence and reuse E-39 rows/panels; include commit-repair/backoff rider.

### J-82 — deterministic token-efficient emission [NEW (measure-first); composes J-24/H-4/mcp_budget] [L1817-L1833]

- [E] Benchmark a compact format such as TOON for heavy tool results.
- [E] Dictionary-encode only repetitive logs and only when dropped volatile details are journal archived.
- [E] Protected-span tokenizer prioritizes fence, inline, URL, datetime, version, path, identifier, prose.
- [E] Preserve structured/must-keep spans byte for byte; never alter code facts.
- [E] Extend mcp_budget from tool list to tool output and compose context formatting/masking.
- [E] Deterministic/reversible, no LLM, measured per leg; marginal delta may legitimately remain off.
- [E] Reject log collapse with unarchived volatiles and verify external packages/grammar before use.

### J-83 — GRC-v3 BeliefState reasoning carrier [KEYSTONE; EXTENSION(I-11); completes the condense seam into the carrier] [L1854-L1869]

- [E] Extend I-11 into one typed carrier with claim, external receipt, abstention/confidence bit read from H-39, and DKG provenance.
- [E] At each step, route to domain receipt and commit only on pass; otherwise abstain.
- [E] Freeze committed entries to prevent later generator rewrite.
- [E] Carry typed state through local-to-cloud escalation and local unload, not raw history.
- [E] Use one carrier across code, vision, trading, and other domains; only validator differs.
- [E] Emit/reuse BeliefStateEvent and trust/context surfaces.
- [E] Failed receipt never commits; model cannot set confidence bit; rewritten frozen entry drops and logs.

### J-84 — 2-axis reasoning-effort classifier [EXTENSION(H-42+I-26); the user's complexity-classifier + effort-toggle] [L1871-L1883]

- [E] TYPE axis reuses D-10: reason, retrieve/recall without chain, or validate.
- [E] COST proxy combines confidence, case base, retrieval coverage, and intent pairs.
- [E] Route trivial to zero reasoning, easy/medium to serial depth, hard to parallel width, very hard to escalate or abstain.
- [E] Window size is not an effort tier.
- [E] Lambda effort control can override automatic tier.
- [E] Extend H-42/I-26 budgets and size J-83 carrier.
- [E] Recall routed to reasoning and model-proposed tier both fail; knee comes from measured K-15.

### J-85 — deterministic overthinking guard [NEW — the ONE net-new mechanism of the wave; measured by K-15] [L1885-L1899]

- [E] Enforce per-call reasoning ceiling from measured model/task knee.
- [E] Early-stop on three deterministic signals: answer oscillation, frozen hesitation markers, declining confidence; flip_ratio above 1 stops.
- [E] Distinguish drift from repetition; run beside anti-spiral.
- [E] Freeze receipt-confirmed state so overthinking cannot overwrite it.
- [E] After about three failed receipted retries, change modality to retrieve/escalate/abstain rather than reason more.
- [E] Never hardcode paper 6K/7K knee and never let LLM choose stop.
- [E] Freeze hesitation list in gate manifest; repetition and drift fixtures must fire only their proper guard.

### J-86 — receipt-gated candidate ladder activation [ACTIVATION(I-4+D-2+I-24+I-26); the user's candidate system] [L1901-L1915]

- [E] Activate branching BeliefState candidates on SessionDag/I-4.
- [E] Selector hierarchy is external receipt first; if no receipt and task is not recall, vote over independent samples; never LLM self-evaluation.
- [E] Receipt-passing candidate commits/freezes; others backtrack to last passing entry.
- [E] Add convergence early stop instead of fixed N.
- [E] PRM is Tier 2 under receipt and guarded against overoptimization.
- [E] Recall always uses retrieval, not vote; no MCTS-first.
- [E] Reuse SearchNode/PRM events and wire-dark activation rows.

### J-87 — journal-completeness substrate [LFG KEYSTONE; the ONE genuine net-new build] [L1944-L1956]

- [E] Prove not merely that rows were unaltered, but that real side effects produced rows.
- [E] Mediate a small declared high-value effect set through Tier-C seccomp user notification so effect and SSB commit are atomic.
- [E] Bound seccomp pointer/TOCTOU claim to atomicity; do not overclaim.
- [E] Independently observe remaining Linux effects using eBPF/LSM-class feed.
- [E] Normalize both feeds and reconcile; a missing row fails.
- [E] Monitor the monitor: dropped counters and impaired observer are first-class fail-closed verdicts.
- [E] Enable observe-only with ELAI_JOURNAL_COMPLETENESS_ENABLE; emit JournalCompletenessEvent.
- [E] State host-trust, schema false-positive, and Linux/Windows asymmetry residues.
- [E] Feed later RRA, canvas, and rules consumers.

### J-88 — the two Layer-0 enablers: runtime-observed DKG edges (E2) + machine-derived-first audit lane (E3) [L1958-L1965]

- [E] Add typed observed_calls DKG edges from action stream, coverage, and traces.
- [E] Runtime edges can widen but never shrink static cones.
- [E] Feed J-90 cross-language/dynamic blast radius.
- [E] Binary verdict lane consumes only machine-derived coverage, SARIF, syscall class, and digest artifacts.
- [E] Never feed executor-authored source into binary verdict lane; LLM auditor prose remains advisory.
- [E] Add no SSB, only edge kind and input routing.

### J-89 — objective + plan-DAG prevention gate (L1 + L8) [RUTHLESS-3] [L1967-L1975]

- [E] Promote spec-grounding output to typed taint-tracked spec bundle.
- [E] Deterministically check coverage.
- [E] Add cross-family spec jury only at planning time, advisory, and distinct from auditor.
- [E] Freeze bundle through I-45/G-8.
- [E] Lift wiring producer/consumer symmetry to plan-DAG goal coverage, artifact reachability, and protocol realizability.
- [E] Stop wire-dark plans before code.
- [E] Observe/mechanism first; enforcement waits for J-34 knee.
- [E] Admit Rice/halting and structural-not-semantic residue; abstain to human; local jury can degrade to checker substitute.

### J-90 — cross-lang + dynamic blast cone (L5, USER'S #1) + usage-weighted reachability (L4) + cant-lie x mutation floor (L7) [RUTHLESS-3] [L1977-L1985]

- [E] Fuse all-language SCIP elai.map edges, cross-language seams, and J-88 runtime edges into J-16 cone.
- [E] Opaque seams widen or broaden and log; never narrow.
- [E] Weight wiring exercisedness using J-56 corpus; require minimum weighted score and at least two scenario families.
- [E] Select safe regression set.
- [E] Scope assertion-validity lint, changed-cone mutation, and anti-vacuity across F-4/C-28/I-20.
- [E] Enforce usage/mutation legs only after J-34 measurement.
- [E] Preserve hidden holdout and report equivalent-mutant lower-bound residue.

### J-91 — re-executable RRA (L3, the wedge technical leg) [J-14 EXTENSION; K-20's companion] [L1987-L1995]

- [E] Capture taint-ingested external response bytes in content-keyed VCR cassette.
- [E] Pin execution environment with OCI digest, lockfile, and CAS fixtures.
- [E] Let a third party reexecute actions against recorded external responses and compare I-45-sealed verdict.
- [E] Report fidelity tiers T0/T1/T2 with honest downgrade.
- [E] World drift never hides; nonsnapshot effects remain action-faithful only.
- [E] Extend J-14 verify and reuse ReproPackEvent kind.
- [E] Completeness depends on J-87.

### J-99 — time-as-context plane (T-1 timestamp slot + T-2 agent-time + T-3 H-51 resume header) [PLANE 1] [L2004-L2006]

- [E] Add ConversationMessage.created_at from OsClock.
- [E] Populate volatile TimeContext slot strictly after last cache breakpoint with absolute time, day rollover, and since-last-user delta.
- [E] Journal time block as actual model input for RRA fidelity; do not use SeededClock for real context time.
- [E] Add elapsed-since-turn and elapsed-since-last-tool on multi-step reentry.
- [E] Add resume header 'N days since last session'.
- [E] Large delta triggers fact/staleness re-verification rather than model assumption.
- [E] Use minute-local when timezone known, otherwise explicitly UTC.
- [E] Time context can never drive gates/transitions; prefix byte-stability is the cache regression gate.

### J-100 — time-as-control plane (T-4 async-tool loop + T-5 watchdog promotion) [PLANE 2] [L2008-L2010]

- [E] Expose background_task_id/BackgroundTaskManager as model-facing async loop.
- [E] After issuing background work, model continues other work and receives completion/failure/watchdog notification.
- [E] Notification is primary; task_status polling is fallback; TaskStop remains available.
- [E] Supervisor, not generator, owns process lifetime.
- [E] Promote WatchdogTimeout stub with primary order-based step/no-material-progress signal.
- [E] Autonomous-only wall-clock watchdog is secondary, recorded but outside seeded replay evaluation; interactive is human-paced.
- [E] Emit Idle, NoMaterialProgress, or Safety; compose per-tool timeout and bad-tool protections.
- [E] Explicit-background is v1; auto-background only observe flag; no cron/scheduler.

### J-101 — transformation-surviving taint binding + cross-process taint + E-37 perimeter (WI-1 keystone + WI-2 + WI-3) [E-37/G-1 EXTENSION] [L2012-L2014]

- [E] Strengthen session trifecta gate: untrusted context plus destructive sink plus egress capability forces HITL/quarantine regardless of verbatim match.
- [E] Add span-bounded same-turn taint window marking post-ingest arguments DERIVED-UNTRUSTED.
- [E] Keep substring matching as fast path.
- [E] Carry taint through Tier-C postcard IPC, verifying global process slot first.
- [E] Complete E-37 output sanitizer/classifier, egress allowlist, and AgentDojo CI perimeter.
- [E] Treat recall after untrusted ingest as derived-untrusted.
- [E] Observe-first; full CaMeL interpreter is deferred.
- [E] Done gate blocks paraphrased injection while ten benign cases remain green; SAFEDELTA metric belongs to K-13.

### J-102 — config-ingress admission gate (WI-6a/b/c/d) [E-43/E-24 EXTENSION] [L2016-L2018]

- [E] At load time, blake3-manifest all instruction Markdown: rules, agents, CLAUDE/AGENTS, and skill SKILL.md.
- [E] New or hash-changed artifact quarantines before Rules slot/context/dispatch.
- [E] Deterministic scanner checks secrets, dangerous/capability-escalation directives, known injection phrases, zero-width/homoglyph/encoded payloads, and frontmatter/schema.
- [E] Cross-family semantic scan is advisory only.
- [E] Add MCP content checks for dangerous FS path, hardcoded secret, approval field, and risk score as DATA with no pip dependency.
- [E] Promote E-43 skill admission from soft to hard PreToolUse refusal.
- [E] Enforce on LOAD and after every modification, not only add; changed hash requires readmission.
- [E] Reuse J-52 intercept; observe-only first; emit Admitted, Quarantined, ReadmissionRequired.

### J-92 — orchestrator-FSM dispatch ordering (L6, the logged P0) [L2020-L2027]

- [E] Promote prose dispatch ordering into first-class FSM substates with EvidenceCheck.
- [E] Editing before context compiler evidence is structurally illegal.
- [E] Mint unforgeable per-phase capability token only from prior phase evidence on supervisor plane.
- [E] J-66 monitors ordering and J-68 TLA proves invariant.
- [E] Trust bottoms out in proven code or explicit USER_OVERRIDE.
- [E] Observe-only first and preserve override path.

### J-93 — the seven Tier-2 composition wiring fills [L2029-L2039]

- [E] A: run deterministic MAST supervisor in-loop over live SSB; warn before halt.
- [E] B: add SessionDag compensation registry; reverse invertible actions and surface noninvertible ones to human.
- [E] C: thread G-1 provenance/taint through memory and H-51 handoff.
- [E] D: enforce per-step citation entailment through SOURCE_CHECK at J-83 receipt grain.
- [E] E: detect autonomous preference-versus-capability drift by diffing dropped constraint against I-45 seal; safety class escalates instead of auto-retry.
- [E] F: derive non-LLM rule labels from complete SSB outcomes, depending on J-87.
- [E] G: mechanically derive feature-map state from I-39, grep, and elai.map.
- [E] Every slice gets one deterministic gate and begins observe-only; prefer existing events.

### J-94 — Tier-3 honesty fixes + THIN measure-first [L2041-L2050]

- [E] Route by offline measured sufficiency, tightening D-10.
- [E] Add modality-choice retrieval router that widens on doubt and remains distinct from J-73's when-to-retrieve.
- [E] DEGRADED stamps context-incomplete and raises audit strictness.
- [E] RRA is T1 only under batch-invariant attention, otherwise honest T0.
- [E] Show read-only plan-versus-execution drift from sealed skeleton versus actions.
- [E] Ship immune-tier-to-frozen-manifest hardening for self-mod checker.
- [E] Defer until measured: BSP barrier, phase-keyed routing, local-lane checker-as-cross-family, and canvas shadow anomaly nodes.
- [E] Avoid gate proliferation and do not build deferred THIN items on faith.

### J-95 — context-activation wires + sequencing track [the CMG technical half; ACTIVATION not invention] [L2072-L2084]

- [E] Wire shipped adaptive_reduce relevance ladder into live turn-overflow before lossy conversation compaction.
- [E] Route context slot assembly through pristine_path_filter so failed-path bursts become contrastive constraints rather than raw text.
- [E] Compose J-78 access spec and begin that leg observe-only.
- [E] Verify and, only if dark, wire BeliefState condense across I-7 escalation and cross-tier handoff; if fully wired, add contract test only.
- [E] Add sequencing pointers rather than reauthoring: J-34 measure first; J-16/J-38 context selection; J-54/H-7 routing; I-1 structural; J-19 engine.
- [E] Prefer existing rows and add no action type.
- [E] Bind no-paper-knee, cache-stability, and context-lifecycle constraints.

### J-96 — verified case graph CORE (mint-on-proof typed case node + deterministic self-wiring edges + recurrence-hit-rate) [VCG SAFE half, observe-only] [L2086-L2097]

- [E] Mint typed case node only after F-4-proven red-to-green with matching stdout hash and sealed G-2 RRA.
- [E] Node contains trigger signature, failure-taxonomy class, blast-radius cone, fix delta, proof receipt, provenance hash, and 1-3 sentence insight.
- [E] Derive edges deterministically from cone overlap, cooccurrence, same taxonomy, and supersession; LLM sets no edge or weight.
- [E] Compute recurrence hit rate when a new failure matches an existing trigger without influencing generation.
- [E] Never inject cases into prompts in this safe phase.
- [E] Never expose case memory to auditor/critic/adversary.
- [E] Hard-filter superseded cases and rederive against current commit.

### J-97 — VCG gated retrieval-time REUSE + episodic->skill->rule promotion ladder [the RISKY leg, default-OFF, ships LAST] [L2099-L2111]

- [E] Retrieve similar proven cases via existing BM25, dense, reranker, RRF, and optional J-38.
- [E] Before injection, revalidate commit/AST, TTL/drift, and supersession.
- [E] Inject top valid case as planner/executor demonstration; never inject into auditor.
- [E] Default off under ELAI_CASE_REUSE_ENABLE plus allowlist and promotion plan.
- [E] Score usefulness off-agent by whether reuse causes a gate-proven held-out recurrence pass.
- [E] Require cross-family and abstention-aware evaluation; reject byte hash, keyword grep, LLM helpfulness, and mere retrieval as fitness.
- [E] Promotion ladder is episodic case to skill after k held-out verified reuses, then rule only with human-gated L3 approval.
- [E] Ship after recurrence value is real; K-18 reports ON/OFF iteration and cost delta.

### J-98 — canvas MEMORY/CASE altitude [VCG visibility; the third canvas altitude beside J-31 system + J-60 process] [L2113-L2123]

- [E] Add memory canvas mode beside system and process.
- [E] Render only proof-born J-96 case nodes with F-4 receipt; no evidence-less grey case is allowed.
- [E] Render deterministic evidence edges and episodic-to-skill-to-rule promotion lighting as evidence accrues.
- [E] Journal is sole store and view rederives byte-identically; no parallel canvas state.
- [E] Reuse J-61 closed steering/FSM; canvas is never second runtime.
- [E] Promote-to-rule is a human-gated proposal.
- [E] Apply per-seat isolation and auditor firewall.
- [E] Sequence core CLI/journal value first and canvas visibility last.

### J-103 — read-access ladder + Read-tool interceptor + savings meter [the research-STRONGEST cell; NEW policy layer] [L2147-L2161]

- [E] Graduated read rungs: SymbolCard, Skeleton, HotPathExcerpt, CodeWindow.
- [E] Rules engine adds READ-ARTIFACT predicate returning approve rung, downgrade, or deny.
- [E] Deny only on security/capability overlap; economy can downgrade but never hard-deny.
- [E] Widen I-46 interceptor to first-class Read and require agent to name expected identifiers.
- [E] On missing named identifier, J-38 sufficiency escalates to next rung through full file; never serve a thin incomplete rung.
- [E] Enforce J-78 access scope when available.
- [E] Measure would-have-read versus served tokens through J-29/CatRAG and require outcome parity.
- [E] Default off with ELAI_READ_LADDER_ENABLE, allowlist, observe-only.
- [E] Done gate is zero served rungs missing a named identifier; no external savings number is claimed.

### J-104 — live-index overlay-before-base read + idle checkpoint [EXTENDS B-31; in-session edit visibility] [L2163-L2173]

- [E] Feed an in-memory overlay from existing file operation signals after edit/rename/delete.
- [E] Tombstone deletes.
- [E] Retrieval consults overlay before base and overlay wins every conflict, preventing retrieval of just-deleted symbols.
- [E] Drain overlay into durable DKG through existing B-31 lock and cascade.
- [E] Trigger drain by replay-stable order/no-progress signal, not wall clock.
- [E] Drain atomically: crash leaves old-consistent or new-consistent, never partial.
- [E] Post-drain store must equal from-scratch index byte-for-byte.
- [E] Default off with allowlist; reuse B-31 row or add OverlayDrained.

### J-105 — counterfactual memory-ROI attribution + ROI-driven curation (+ G-3 centrality observe-only) [EXTENDS J-21/K-18/H-43/H-39] [L2175-L2188]

- [E] Attribute success/failure causally to each retrieved memory entry, not aggregate memory use.
- [E] Off-agent scorer uses execution outcome and abstention shape, never LLM judgment.
- [E] Attribution drives J-21 activation decay, trimming, and demotion.
- [E] Apply code-graph centrality rerank only to graph-shaped queries classified by D-10.
- [E] Centrality is downgraded/default-off observe-only because evidence is marginal; measure separately.
- [E] Preserve H-31 zero-memory auditor/critic/adversary firewall.
- [E] Both features default off with allowlist; centrality remains observe-only longest.
- [E] Done gate measures causal-entry accuracy, fixes degradation from naive accumulation, and does not import paper numbers.

### J-106 — output-side meta-commentary enforcement (op-emission roles only) [the OUTPUT twin of I-32, HD-3-bounded] [L2190-L2201]

- [E] Scope strictly to operation-emission roles using I-32 classification; reasoning roles are untouched.
- [E] Frozen banned-phrase detector rejects and performs bounded retry.
- [E] Require work-log grammar READ, FOUND, DECISION, ACTION, RESULT.
- [E] Evaluate action density by tool actions per prose block and words per message on corpus.
- [E] Never constrained-decode reasoning; decoding freedom is preserved there.
- [E] Violations use I-47 loud failure path.
- [E] Default off with ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE and allowlist; observe until reasoning-corpus false positives are zero.

### J-107 — `elai bench longcot` recursive-scaffold-vs-flat measurement arm [the one clear build; GATES the wave] [L2216-L2229]

- [E] Add elai bench longcot on reused K-15/I-27 runner.
- [E] Compare FLAT-OFF to CONSTRAINED-SCAFFOLD-ON.
- [E] Measure receipts, abstention rate, tokens per correct, and per-model/lane knee over at least five paired seeds.
- [E] Treat abstention as positive on unreachable tasks and use deterministic scorer only.
- [E] Test whether recursive slices lift local Qwen-class lane without CodeAct collapse.
- [E] Gate J-108 on the measured result.
- [E] Reuse ConvergenceCurveMeasured and ConvergencePanel.
- [E] A result without curve rows or scaffold with worse local fabrication fails.

### J-108 — multi-hop DEPTH operator: HippoRAG-PPR primary + constrained abstain-on-budget RetrievalNavigator fallback [the measured "merge"; GATED on J-107] [L2231-L2249]

- [E] Primary path routes entity-linked multi-hop queries through shipped HippoRAG PPR, then one atomic model extraction.
- [E] Fallback for missing entity linkage is constrained RetrievalNavigator.
- [E] Fixed NavOp set is Grep, Read, Expand, SubQuery depth at most 2, Answer.
- [E] Candidate set comes from query_with_tier; adaptive tool journals named ops.
- [E] Model proposes one op and reads one atomic entity; deterministic executor adjudicates.
- [E] Budget exhaustion yields BudgetCollapsed/winner none and abstention, never fabricated answer.
- [E] Reuse SearchLadder handle/journal/backtrack idiom and SessionDag.
- [E] Reject free Python RLM everywhere, model-chosen orchestration, and local CodeAct.
- [E] Wire real task_runner retrieval entrypoint and allowlist; op outside fixed set or model-executed code fails.

### J-109 — learned-artifact admission hardening: immutable-surface span dedup + counterfactual replay fitness at the write seams [L2264-L2282]

- [E] One ordered admission pipeline fronts memory add, playbook intake, and case reuse.
- [E] AD-1 exact match updates existing artifact.
- [E] AD-2 compares candidate spans to immutable instruction surface and rejects relearning system prompt; index fingerprint is blake3 and rebuilds if stale.
- [E] AD-3 splits duplicate band by trigger Jaccard into merge or coexist.
- [E] AD-4 replays byte-exact originating prompt with candidate on original lane N=3.
- [E] Admit only with zero harmful and at least two of three flips; execution receipt is authoritative where runnable, judge advisory only.
- [E] Embedder failure falls back to lexical with labeled verdict, never blind accept.
- [E] Every decision journals; thresholds are starting values and must be ELAI recalibrated.
- [E] Observe-only until false positives bounded on at least three real runs; no-verdict write or receiptless flip fails.

### J-110 — abstention timeliness (SPL) + abstention-warranted eval supply: score WHEN the agent stops [L2284-L2305]

- [E] Fold typed D-16/I-8/L-7 signals to first_abstain_step; do not rederive from prose.
- [E] Compute SPL as oracle_step divided by max(first abstain, oracle), timely recall/pass@k only on abstention-warranted subset.
- [E] Report false-abstain rate on solvable twins.
- [E] Deterministically mutate corpus rows with exactly one labeled defect: DKG-witnessed false premise, contradiction, or underspecified intent.
- [E] Build delayed pack and record oracle step by construction, never LLM oracle.
- [E] Bench FLAT versus GATES-ON with at least five paired seeds.
- [E] Use measured knee to calibrate J-84 very-hard behavior.
- [E] Late-stop flags may later feed I-12 through J-109; cross-size transfer remains unverified.
- [E] Missing oracle step cannot default to perfect SPL; false-premise witness must verify in DKG.

### J-112 — spoken render + narration dispatcher [the voice output plane: every utterance is a rendered SSB row] [L2331-L2349]

- [E] Deterministic per-event spoken-render table formats typed journal fields with seeded variants.
- [E] Add spoken target to existing H-36 NotificationDispatcher; never create a second dispatcher.
- [E] Queue priority lets needs-input preempt progress.
- [E] Rate-limit at step grain and support silent, summary, verbose deterministic row-class filters.
- [E] Use local TTS; Kokoro/Piper candidates require dependency confirmation.
- [E] Event-path narration never uses model-generated text.
- [E] Only model slot is on-demand digest; it carries row-id receipts and deterministic row-count footer appended outside model output.
- [E] Add elai voice --status and NotificationManager read-side.
- [E] Utterance without row, footer/body mismatch, or substep chatter fails.

### J-111 — voice ingress + spoken readback-confirm [speech is a second transducer into the J-32 front door] [L2351-L2373]

- [E] Local Python daemon defaults push-to-talk; open mic is default off because audio is unauthenticated ingress.
- [E] Stream local ASR through user-confirmed candidate dependencies.
- [E] Journal VoiceUtterance with transcript, confidence, audio blake3, ASR model pin, and Untrusted taint before routing; never store audio bytes.
- [E] Route through the exact J-32 FrontDoorRouter call, with no second grammar/router.
- [E] Address sessions by phonetically distinct call signs; unresolved address gets spoken clarify, never guess.
- [E] MEDIUM and above speak readback generated from routed intent/J-35 card, never transcript echo.
- [E] Explicit confirm token journals as J-32 chip tuple; HIGH risk is never voice-only.
- [E] Confidence floor, call signs, and routed-intent readback are deterministic mis-transcription controls.
- [E] Production CLI is elai voice; default off with ELAI_VOICE_ENABLE and allowlist.
- [E] Confirmed voice and typed slash route to identical dispatch rows; missing confirm, transcript-only readback, or misaddress dispatch fails.

## Composed workflows and interaction map

### Front door to verified execution

1. [E] Typed input enters J-32; voice input first becomes a tainted, audio-hash/model-pinned J-111 row, then enters the same J-32 router.
2. [E] J-32 deterministically or constrainedly selects a lane; ambiguity returns a chip, and J-79 may offer—never auto-run—grill from a measured signal.
3. [E] J-35 reconstructs visible IntentSpec, checks premises before retrieval/decomposition, surfaces aim uncertainty, and uses journal-derived assumptions only where reversible/risk policy permits.
4. [E] J-89 validates typed objective bundle and plan-DAG reachability before code; J-39 sizes each node against measured context ceiling.
5. [E] J-92 capability tokens and FSM substates enforce dispatch order; J-78 constrains exactly what each worker may see.
6. [E] J-83 carries grounded claims, J-84 sizes effort, J-85 stops drift, and J-86 selects candidates by receipt before any vote.
7. [E] J-58 freezes a negotiated, monotone-tightening sprint contract before execution.
8. [E] Results return through receipt-bearing J-32 cards and may be spoken by J-112 only as deterministic renders of rows.

### Perception, documents, retrieval, and read access

1. [E] Images pass J-1 preprocessing/composition/router, then J-8 schema/tiling rules, then J-77 region citation, taint, transform, and layout-graph faithfulness.
2. [E] Documents use J-33 structured parser first and E-36 VLM escalation only when required.
3. [E] J-104 overlay prevents stale in-session symbols before any base retrieval.
4. [E] J-73 decides whether retrieval is needed; if yes, J-18/J-72 provide adapted vector/LSP/reranker, J-74 graph channel, and existing hybrid cascade.
5. [E] J-103 initially serves the cheapest read rung that contains named identifiers and climbs on a miss.
6. [E] During reasoning, J-38 can call hybrid/graph retrieval in bounded rounds; J-108 handles true multi-hop by PPR first and constrained navigator fallback.
7. [E] J-40 extractively compresses web evidence, J-41 binds claims to a closed evidence set, and unsupported claims abstain.
8. [E] J-34/J-76 calibrate placement, trim, and local compaction; raw pages and irrelevant failed paths do not bloat reasoning context.

### Edit, validation, safety, and evidence

1. [E] Before a write, J-102 admits changed instruction/config artifacts, J-62 injects the environment card, J-27 hides protected spans, and J-3 captures pre-image.
2. [E] J-52 runs per-edit diagnostics and return-time artifact checks; J-12/J-67 can block, suppress, insert, or edit through one hook chain.
3. [E] J-16 computes the symbol cone; J-90 widens it across languages/runtime edges and applies usage/mutation anti-vacuity.
4. [E] J-23 language review tools, J-65 safety-critical static rules, and J-68 critical-core proofs provide distinct verification layers.
5. [E] J-101 preserves taint across paraphrase/process; J-17 catches jointly dangerous plan capabilities.
6. [E] J-87 mediates/observes/reconciles real effects so journal completeness is testable; J-66 evaluates temporal obligations only after J-69 integrity succeeds.
7. [E] J-14 seals applied determinism and locates divergence; J-91 adds external-response cassette and environment so third parties can reexecute.
8. [E] J-4 derives the handoff ledger from hook evidence; J-70 maps each objective to independent frozen evidence.

### Runtime, model, cost, and multi-seat routing

1. [E] J-9 declares hardware/lane fit; J-34 declares measured context knees; J-51 declares tool-call adherence.
2. [E] J-55 selects CodeAct versus constrained minimal-schema tool calls; J-36 caches a frozen local head where architecture allows.
3. [E] J-48 manages coarse local tier residency/admission and spills through existing cloud cascade; J-19 supplies additive vLLM engine behavior.
4. [E] J-49 applies seat fair share and spend ceilings before serving; J-59 isolates each seat's memory/trust.
5. [E] J-54 rotates free providers but marks verification-lane runs nonreproducible; J-30 handles verified BYOK/custom endpoints with turn lock; J-64 offers ACP transport without removing direct CLI.
6. [E] J-13 records seeded local token determinism; J-28 normalizes honest per-turn usage; J-29 renders live and journaled totals.
7. [E] J-63, if product-fit/security approval succeeds, lets generated apps consume only the same policy-governed local inference path.

### Memory, learning, and self-improvement

1. [E] J-37 adjudicates new facts as KEEP/STALE/REPLACE/UNKNOWN before read; J-21 validates code state, authorization, provenance, and poisoning.
2. [E] J-105 assigns per-entry execution-grounded ROI to drive decay/curation; auditors remain memory-free.
3. [E] J-10 propagates receipt-bound insights through branch ancestry and ideation constraints.
4. [E] J-96 mints cases only from proven fixes and derives their graph; J-97 reuses cases only after staleness/security filters, default off, never in auditor.
5. [E] J-109 applies exact/immutable-surface/duplicate/replay gates at every learned-artifact admission seam.
6. [E] J-20 gates promotion with nonstationary reward plus security invariants; J-50 enriches SFT with real recovery chains.
7. [E] J-56 maintains the failure-derived eval corpus; J-57 promotes prompt templates only on no-regression; J-110 measures when abstention happens.
8. [E] J-98 renders the proven-case graph only after mint/reuse safety order; J-46/J-47 measure and falsify doctrine efficacy rather than assume it.

### Canvas and supervision

1. [E] J-31 shows intended system bindings and checker-derived wiring; J-60 shows actual FSM/agent process views.
2. [E] J-61 turns a closed set of gestures into journaled intents; user preempts the model, and no direct canvas state exists.
3. [E] J-98 adds durable memory/case altitude; all three altitudes rederive from SSB and dispatch through normal FSM.
4. [E] J-80 supplies durable worktree leases/healing; J-81 detects silent worker wedges, survives supervisor restarts, and applies mid-loop caps.
5. [E] J-100 keeps background tools supervised and lets the model continue while notifications—not process ownership—return status.

## Dependency, sequencing, status, and deferred ledger

- [E] Protection-first: J-3 and J-4 precede J-5; J-2 is deliberately late behind live authorization, pin, taint, and destructive-gate rechecks.
- [E] Perception: J-1 precedes J-8; J-77 composes both; heavy VLM/OCR comparisons are remote/hardware/anti-slopsquat gated.
- [E] Front door: J-32 ships before J-31; J-35 sits after J-32 and before decomposition; J-111 hard-depends on J-32 and ships after J-112.
- [E] Canvas: J-31 is system base, J-60 process watch, J-61 steering, J-98 memory visibility last; J-60's full worktree tab needs J-80/I-2.
- [E] Usage/provider: J-28 feeds J-29 and all custom/free backends; J-30 and J-54 extend B-37 without parallel registry.
- [E] Serving/seats: J-48 precedes J-49; J-49 precedes J-59; J-63 depends J-48 and Tier-C plus unresolved product fit.
- [E] NASA runtime spine is J-69 integrity, then J-66 temporal verdict, then J-67 action.
- [E] GRC-v3 spine is J-83 carrier first, J-84/J-85 policy second, J-86 candidate ladder last.
- [E] Lacuna roots J-87/J-88/J-89 precede consumers; J-91 needs J-87 completeness; J-93 taint/rule legs need J-87.
- [E] VCG order is J-96 safe mint/measure, then J-97 risky default-off reuse, then J-98 visibility.
- [E] Recursive reasoning is J-107 measurement first; J-108 does not ship unless the local-lane scaffold result clears the collapse check.
- [E] J-34 is a broad enforcement dependency: decomposition sizing, context placement, J-36 tail budget, and L1/L4/L7 enforcement wait for measured per-model/lane knees.
- [E] J-7 retains a pending-user-yes marker and holds importing the full impeccable skill.
- [E] J-9 Tier-2 CLI and catalog panel are deferred together; catalog remains informational and isolated from routing.
- [E] J-13 leaves SDPA/FA2 outside determinism claim; J-36 leaves TurboQuant coexistence deferred.
- [E] J-19 is additive; removing/swapping the HF engine requires explicit user confirmation.
- [E] J-24 claims no tool-cost win before SAFEDELTA and never calls lazy schemas zero tokens.
- [E] J-33 is default-off, external-service/user-confirmed, with structured failure rather than invalid indexing.
- [E] J-37 cannot ship onto a memory store with no production consumer.
- [E] J-39, J-41, and J-42 begin with EXP-1/2/3 observe-only measurements; InftyThink-style training is deferred.
- [E] J-43..J-47 are observe-only until their falsifiers/calibration pass; a comprehensive scorer remains control arm, not production design.
- [E] J-50 never fabricates recovery data to hit target; J-51 role gate is advisory, not hard kill.
- [E] J-54 free-pool verification causes an explicit RRA/G-10 trust downgrade; it cannot masquerade as reproducible.
- [E] J-59 per-seat memory view is deferred; isolation is backend-first.
- [A] J-63 generated-app coupling is a stated design hypothesis and requires a before-build product-fit decision plus security review.
- [E] J-65 begins observe-only; J-68 formal tools require confirmed dependencies.
- [E] J-71 is a read-only spike and may conclude decline; no Cedar dependency or production gate is authorized by the plan itself.
- [E] J-72 training is offline-only; J-75 may measure but never prune; J-76 defers SproutRAG.
- [E] J-79 widens ambiguity triggers only after measured false-offer evidence.
- [E] J-82 can remain off on a marginal local delta.
- [E] J-87 is Linux-observer-first with explicit Windows asymmetry and host-trust ceiling.
- [E] J-89/J-90 mechanism/enforcement split waits on J-34; J-94 THIN items are explicitly deferred measure-first.
- [E] J-97/J-103/J-104/J-105/J-106/J-109 are default-off or observe-first with allowlist/promotion/false-positive gates.
- [E] J-112 TTS and J-111 ASR dependencies require confirmation; open microphone stays default off and high risk remains non-voice-only.

## Exact configuration, API, telemetry, and surface contract

### Flags and data/config artifacts

- [E] ELAI_SEED is the deterministic seed source for local token streams, benchmarks, graph/view rederivation, and paired runs.
- [E] ELAI_INTENT_CONTROL_ENABLE, ELAI_JOURNAL_COMPLETENESS_ENABLE, ELAI_LIVE_OVERLAY_ENABLE, ELAI_READ_LADDER_ENABLE, ELAI_CASE_REUSE_ENABLE, ELAI_ADMISSION_HARDENING_ENABLE, ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE, ELAI_RETRIEVAL_NAVIGATOR_ENABLE, and ELAI_VOICE_ENABLE are named enable paths coupled to allowlist/promotion or observe-first rules.
- [E] ELAI_TOOLCALL_ADHERENCE_GATE_DISABLE, ELAI_TOOLCALL_REPAIR_DISABLE, ELAI_ENV_CARD_GATE_DISABLE, ELAI_GATE_ACTION_DISABLE, and ELAI_ACP_TRANSPORT_DISABLE are named kill/disable paths whose effect must be loud.
- [E] ELAI_DARWINIAN_LOOP_ENABLE guards J-20 promotion changes.
- [E] ELAI_GENERATED_APP_BASE_URL points a Tier-C generated app only at the J-63 broker.
- [E] seats.toml presence, not an enable variable, activates J-49 multi-seat behavior.
- [E] custom_providers.toml declares custom compatible backends; models.toml remains canonical for static provider/model/pricing/free-pool data.
- [E] Perception DATA includes per-model sweet spots/resize and vision ladder; J-37 adds memory_state_schema.toml; J-23 adds arch_gate.toml.
- [E] Doctrine debt and gate manifests are off-agent frozen data; changed instruction Markdown requires J-102 hash readmission.

### API and CLI

- [E] J-2 MCP verbs: task_submit, task_status, fsm_get_state, session_fork, session_rewind, plan_status, rra_show, intervention_respond.
- [E] J-3 undo.py verbs: list, diff, restore, prune.
- [E] Retrieval/measurement commands: elai retrieval gold-build; elai bench retrieval-adapt; elai bench rrf-ablate; elai bench context-fill; elai bench tool-call-adherence; elai bench modality; elai bench tool-output; elai bench vlm-eval.
- [E] Reasoning/eval commands: elai bench longcot; elai bench abstention; elai eval ingest; elai eval run --scope subset|full; elai dataset export --format sft.
- [E] Runtime/product commands: elai usage; elai run --local --emit-rra; elai run --role-backend draft=free-pool; elai prompt promote --to; elai artifact run; elai voice; elai voice --status.
- [E] J-38 agent tool is hybrid_search(query, top_k, mode); J-108 permits only Grep/Read/Expand/SubQuery/Answer.

### Exact telemetry families

- [E] Perception/hardware/document: PerceptionScaffoldEvent, VlmPerceptionUpgradeEvent, HardwareModelCatalogEvent/CatalogBuilt, DocumentParsedEvent/ParseFailed.
- [E] Control/intent/canvas: IntentRouted, IntentControlEvent, SystemContractEvent, CanvasOpApplied, ProcessViewProjected or CanvasViewEvent, CanvasRedirectIntent.
- [E] Harness/reasoning: HarnessInterventionEvent, InsightPropagated, ReasoningCheckpointEvent, BeliefStateEvent, ReasoningBudgetScaled, OverthinkingGuardFired.
- [E] Determinism/RRA: LocalDeterminismEvent, DeterminismProbeRun, DeterminismDivergence, ReproPackEvent, TierEvaluated, FirstDivergenceLocated, FlowFingerprintMismatch.
- [E] Retrieval/context: RetrievalAugmentEvent, QueryTrace, RetrievalLoopEvent, RetrievalAbstained, ContextCalibrationEvent, ContextAccessSpecEvent.
- [E] Edit/review/safety: BlastRadiusEvent, CompositionalPolicyEvent, ReviewGateCatalogEvent, HookEnforcementEvent or SubagentEvidenceGated/EditDiagnosticGated/PathRuleInjected, EnvCardGated, ToolInputRepaired.
- [E] Memory/learning: MemoryValidationEvent, StalenessAdjudicationEvent, MemoryGovernanceEvent, CaseNodeMinted, CaseReuseScored, MemoryRoiAttributed, AdmissionVerdict.
- [E] Runtime/provider/cost: UsageTurnEvent, ProviderByokEvent, FreePoolEvent, ModalityRouted, RouteDecisionEvent or LocalServingDecisionEvent, SeatAdmissionEvent, GeneratedAppInference, RuntimeEvent::CliSubprocessSpawned.
- [E] Evaluation/promotion: SecurityInvariantGate, SftExhaustEvent, ToolCallAdherenceMeasured, EvalCorpusEvent, PromptPromotionEvent, SprintContractEvent, ConvergenceCurveMeasured.
- [E] Journal/gate integrity: SsbMonitorVerdict, GateActionApplied or HarnessInterventionEvent extension, JournalCompletenessEvent, ConfigAdmissionEvent.
- [E] Background/voice: AgentNotificationEmitted/agent_notification_emitted, WatchdogTimeout, VoiceUtteranceIngested.
- [E] Prefer-reuse alternatives in the summary are real implementation decisions, not permission to omit observation; lock review chooses field/kind extension versus a new action type.

### User surfaces

- [E] KnowledgeDrawer/ContextStatePanel show retrieval channels/rounds, skipped retrieval, calibrated curves, active thresholds, placement, and absolute context counts.
- [E] RuntimePanel/CloudPanel/model-cost cockpit show backend comparison, hardware eligibility, transport, BYOK inventory, free-pool trust, modality, adherence, steering, and token budget.
- [E] UsageWindow plus chat/CLI live gauge and canvas badge expose per-turn/per-model/per-task/per-session/per-lane usage without mislabeling local tokens as zero dollars.
- [E] Audit/compliance/review surfaces show blast cones, fragmentation, SARIF/a11y findings, temporal/gate actions, chain integrity, and RRA divergence.
- [E] Memory hub/governance/VCG surfaces show provenance, case proof, recurrence/reuse/promotion, and per-seat isolation without exposing memory contents in rows.
- [E] J-32 ACK/progress/RESULT and J-35 decision cards are the front-door UI; J-79 offer chip preserves consent.
- [E] Canvas has system, process, and memory altitudes; edges/status are checker/journal-derived and all steering goes through journaled intents.
- [E] NotificationManager consumes spoken-target notification rows; no utterance exists only in UI state.
- [E] Explicit N/A/deferred UI cases remain valid only where the numbered entry says so; runtime state otherwise needs a paired surface or W-6 justification.

## Deeper-source manifest

### Canonical J plan files

Base directory: docs/thefinalglance/J-plan

- [E] Index/orchestration source present but not opened: J-0_INDEX.md.
- [E] Direct source read completely: J-SUMMARY_INDEX.md.
- [E] All 112 numbered originals below exist by file enumeration; none was opened in this pass.
- J-1: J-1_perception_gateway.md
- J-2: J-2_elai_mcp_control_plane.md
- J-3: J-3_harness_undo_file_backups.md
- J-4: J-4_handoff_file_ledger.md
- J-5: J-5_litegame_harness_port.md
- J-6: J-6_incremental_eval_skill.md
- J-7: J-7_ext_steal_intake_closeout.md
- J-8: J-8_vlm_perception_research_upgrade.md
- J-9: J-9_hardware_detection_model_catalog.md
- J-10: J-10_insight_backprop_tree_memory.md
- J-11: J-11_arbor_steal_intake_closeout.md
- J-12: J-12_runtime_intervention_harness.md
- J-13: J-13_local_lane_token_determinism.md
- J-14: J-14_repro_pack_rra_verify_upgrade.md
- J-15: J-15_determinism_steal_amendments.md
- J-16: J-16_blast_radius_regression_gate.md
- J-17: J-17_plan_level_compositional_safety.md
- J-18: J-18_p1_retrieval_hardening.md
- J-19: J-19_vllm_alternative_backend.md
- J-20: J-20_selfimprove_bandit_security_gate.md
- J-21: J-21_memory_semantic_validation_poisoning.md
- J-22: J-22_journal_payload_schema_lock.md
- J-23: J-23_multilang_review_gate_catalog.md
- J-24: J-24_nonprompt_steering_context_frugal_tools.md
- J-25: J-25_mimocode_context_memory_mechanics.md
- J-26: J-26_sia_evaluator_abi_task_packs.md
- J-27: J-27_harness_protected_blocks_webcache_dispatch.md
- J-28: J-28_token_ledger_usage_unification.md
- J-29: J-29_usage_window_realtime_surfaces.md
- J-30: J-30_provider_byok_custom_endpoints.md
- J-31: J-31_canvas_system_mode.md
- J-32: J-32_elai_frontdoor_router.md
- J-33: J-33_mineru_document_parsing_lane.md
- J-34: J-34_empirical_context_calibration.md
- J-35: J-35_intent_control_layer.md
- J-36: J-36_freeze_head_local_prefix_kv_reuse.md
- J-37: J-37_stale_staleness_adjudication.md
- J-38: J-38_agentic_retrieval_loop.md
- J-39: J-39_decomposition_node_admission_gate.md
- J-40: J-40_research_modes_extractive_compression.md
- J-41: J-41_cheap_grounding_stack.md
- J-42: J-42_within_step_distill_reset_reasoning.md
- J-43: J-43_doctrine_propagation_keystone.md
- J-44: J-44_reasoning_method_artifact_shape.md
- J-45: J-45_cross_family_advisory_calibration.md
- J-46: J-46_doctrine_debt_rate_measurement.md
- J-47: J-47_program_falsifier_per_axis_discipline.md
- J-48: J-48_local_serving_tier_router.md
- J-49: J-49_multi_seat_admission_quota.md
- J-50: J-50_recovery_trajectory_sft_exhaust.md
- J-51: J-51_toolcall_adherence_floor_role_gate.md
- J-52: J-52_returntime_perEdit_enforcement_hooks.md
- J-53: J-53_openai_compat_tool_input_repair.md
- J-54: J-54_free_api_rotation_pool.md
- J-55: J-55_model_strength_modality_router.md
- J-56: J-56_living_eval_corpus_governance.md
- J-57: J-57_eval_gated_prompt_promotion.md
- J-58: J-58_sprint_contract_negotiation_loop.md
- J-59: J-59_per_seat_memory_trust_isolation.md
- J-60: J-60_layered_observability_canvas.md
- J-61: J-61_canvas_redirect_loop.md
- J-62: J-62_env_constraint_skill_cards_prewrite_gate.md
- J-63: J-63_local_lane_claudeception.md
- J-64: J-64_acp_transport_for_agent_bridges.md
- J-65: J-65_power_of_ten_generated_code_gate.md
- J-66: J-66_ssb_temporal_monitor_engine.md
- J-67: J-67_multimode_gate_actions.md
- J-68: J-68_self_code_formal_verification.md
- J-69: J-69_ssb_log_integrity_hash_chain.md
- J-70: J-70_gate_manifest_traceability_ivv_independence.md
- J-71: J-71_cedar_policy_as_code_spike.md
- J-72: J-72_indomain_retriever_adaptation.md
- J-73: J-73_retrieval_abstention_gate.md
- J-74: J-74_graph_as_retrieval_channel.md
- J-75: J-75_rrf_ablation_harness.md
- J-76: J-76_context_rot_calibration_riders.md
- J-77: J-77_perception_faithfulness_trust_spine.md
- J-78: J-78_context_access_spec.md
- J-79: J-79_grill_signal_interlace.md
- J-80: J-80_worktree_pool_lifecycle.md
- J-81: J-81_autonomous_supervision_hardening.md
- J-82: J-82_deterministic_token_efficient_emission.md
- J-83: J-83_grcv3_beliefstate_reasoning_carrier.md
- J-84: J-84_grcv3_reasoning_effort_classifier.md
- J-85: J-85_grcv3_overthinking_guard.md
- J-86: J-86_grcv3_receipt_gated_candidate_ladder.md
- J-87: J-87_journal_completeness_substrate.md
- J-88: J-88_runtime_dkg_edges_machine_audit_lane.md
- J-89: J-89_objective_and_plan_dag_prevention_gate.md
- J-90: J-90_crosslang_dynamic_blast_cone_composition_gate.md
- J-91: J-91_reexecutable_rra_track.md
- J-92: J-92_orchestrator_fsm_dispatch_ordering.md
- J-93: J-93_lacuna_tier2_composition_fills.md
- J-94: J-94_lacuna_tier3_honesty_thin_measure_first.md
- J-95: J-95_context_activation_wires.md
- J-96: J-96_verified_case_graph_core.md
- J-97: J-97_case_reuse_gated_promotion_ladder.md
- J-98: J-98_canvas_case_memory_altitude.md
- J-99: J-99_time_as_context_plane.md
- J-100: J-100_time_as_control_plane.md
- J-101: J-101_transformation_surviving_taint_binding.md
- J-102: J-102_config_ingress_admission_gate.md
- J-103: J-103_read_access_ladder_interceptor.md
- J-104: J-104_live_index_overlay_idle_checkpoint.md
- J-105: J-105_memory_roi_attribution_curation.md
- J-106: J-106_output_metacommentary_enforcement.md
- J-107: J-107_longcot_recursive_scaffold_bench.md
- J-108: J-108_retrieval_navigator_depth_operator.md
- J-109: J-109_learned_artifact_admission_hardening.md
- J-110: J-110_abstention_timeliness_eval_supply.md
- J-111: J-111_voice_ingress_spoken_confirm.md
- J-112: J-112_spoken_render_narration_dispatcher.md

### Grounding maps explicitly named or colocated

All paths below exist and were enumerated, not read:

- docs/thefinalglance/J-plan/_grounding/J-HARNESS-GROUNDING-MAP.md
- docs/thefinalglance/J-plan/_grounding/J-PERCEPTION-GROUNDING-MAP.md
- docs/thefinalglance/J-plan/_grounding/J-DOCTRINE-PROPAGATION-GROUNDING-MAP.md
- docs/thefinalglance/J-plan/_grounding/J-NASA-GATES-GROUNDING-MAP.md
- docs/thefinalglance/J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md
- docs/thefinalglance/J-plan/_grounding/J-LACUNA-FILL-GROUNDING-MAP.md
- docs/thefinalglance/J-plan/_grounding/J-SDL-CONTEXT-MEMORY-GROUNDING-MAP.md
- docs/thefinalglance/J-plan/_grounding/J-RLM-GROUNDING-MAP.md
- docs/thefinalglance/J-plan/_grounding/J-VOICE-GROUNDING-MAP.md

### Explicitly provided or referenced non-numbered design/reasoning sources

These exact paths were either provided as orientation or named by the summary and verified to exist; they were not opened:

- docs/thefinalglance/ELAI-SPEC-OVERVIEW.md
- docs/thefinalglance/NASA-GRADE-DETERMINISTIC-GATES-PLAN.md
- docs/strategy/ELAI-DETERMINISM-DOCTRINE.md
- docs/strategy/WIRING-FIX-PROPAGATION.md
- workflow/reasoning/J-plan-prereading-2026-06-10.md
- workflow/reasoning/J-11-decisions.md
- workflow/reasoning/J-15-decisions.md
- workflow/reasoning/steal-determinism-repos-2026-06-11.md
- workflow/reasoning/steal-pillar-sota-2026-06-12.md
- workflow/reasoning/steal-mimocode-sia-agentskills-2026-06-12.md
- workflow/reasoning/steal-goose-factory-2026-06-12.md
- workflow/reasoning/elai-front-door-bridge-reasoning.md
- workflow/reasoning/elai-local-reasoning-and-context-2026-06-14.md
- workflow/reasoning/retrieval-agentic-loop-grounding-2026-06-16.md
- workflow/reasoning/decomposition-grounding-reasoning-2026-06-16.md
- workflow/reasoning/determinism-doctrine-propagation-design-2026-06-16.md
- workflow/reasoning/steal-production-ai-talks-2026-06-20.md
- workflow/reasoning/context-retrieval-research-2026-06-23.md
- workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md
- workflow/reasoning/CONTEXT-MANAGEMENT-DEEP-DIVE-2026-06-25.md
- workflow/reasoning/self-improving-workflow-canvas-verified-case-graph-2026-06-25.md
- workflow/reasoning/time-awareness-design-2026-06-25.md
- workflow/reasoning/owasp-llm-agentic-gap-analysis-2026-06-25.md
- workflow/reasoning/LACUNA-FILL-PROGRAM-2026-06-26.md
- workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md
- workflow/reasoning/rlm-recursive-agents-grc-2026-06-26.md
- workflow/reasoning/steal-agenticsts-paw-abstention-2026-07-04.md
- workflow/papers/liberty-rotation-interface.md
- workflow/papers/agenticsts-summary.md
- workflow/papers/programasweights-summary.md
- workflow/papers/agentic-abstention-summary.md
- docs/.source/lazycodex-main-INDEX.md

### Explicit interface/source maps called out by the summary

The summary names these files or their exact interface family; all listed paths exist and were not opened:

- J-10/J-11 corpus: workflow/reasoning/ARBOR-COMBENCH-DENOVOSWE-STEAL-LIST-2026-06-11.md
- J-10/J-11 interfaces: workflow/papers/2606-11926-arbor-hypothesis-tree-refinement-interface.md; workflow/papers/2606-10479-combench-interface.md; workflow/papers/2606-10728-denovoswe-interface.md
- J-12..J-15 determinism/source interfaces: workflow/papers/life-harness-paper-interface.md; workflow/papers/life-harness-source-interface.md; workflow/papers/batch-invariant-ops-source-interface.md; workflow/papers/detllm-source-interface.md
- J-65..J-71 NASA interfaces: workflow/papers/power-of-ten-interface.md; efficient-monitoring-safety-interface.md; monitoring-rewriting-interface.md; enforceable-security-policies-interface.md; rv-survey-interface.md; edit-automata-interface.md; cbmc-interface.md; tla-specifying-systems-interface.md; spark-gnatprove-interface.md; raft-interface.md; do333-case-studies-interface.md; nasa-std-8739-8b-interface.md; cedar-interface.md (all under workflow/papers).
- J-72..J-76 context/retrieval interfaces: workflow/papers/evoembedding-interface.md; mcompassrag-interface.md; repoformer-interface.md; coderag-system-interface.md; repograph-interface.md; codescalebench-interface.md; cmu-repo-search-interface.md; chroma-context-rot-interface.md; luceck-out-of-context-interface.md; rlm-interface.md (all under workflow/papers).

### Filename-matched deeper-source candidates

These are [I] candidate matches for sources described but not given an exact path in the summary. Inspect J-0 and the relevant numbered original before treating the match as canonical:

- [I] 'harness-redesign doc': workflow/reasoning/harness-redesign-recommendations-2026-06-10.md, with companion harness-redesign-reading-2026-06-10.md.
- [I] 'ext-repo steal doc': workflow/reasoning/ext-repo-steal-recommendations-2026-06-10.md.
- [I] J-48 serving source: workflow/reasoning/steal-local-serving-router-2026-06-18.md.
- [I] J-50/J-51 GUAVA intake: workflow/reasoning/steal-deerflow-guava-2026-06-19.md and workflow/papers/guava-interface.md.
- [I] J-52 re-intake: workflow/reasoning/steal-lazycodex-reintake-2026-06-19.md.
- [I] J-54 free-pool intake: workflow/reasoning/steal-free-api-rotation-2026-06-20.md.
- [I] J-64 ACP intake: workflow/reasoning/steal-goose-acp-2026-06-21.md.
- [I] J-80..J-82 Kunchen intake: workflow/reasoning/steal-kunchen-agentic-tools-2026-06-24.md.
- [I] J-103..J-106 supporting gap reports: workflow/reasoning/elai-gaps-sdl-ai-setup-2026-06-26.md and steal-sdl-mcp-ai-setup-2026-06-26.md.
- [I] J-62/J-63 Fable provenance is not exact from the summary; possible later filename matches are workflow/reasoning/steal-fable5-orchestration-2026-07-10.md and fable5-orchestration-sources-2026-07-10.md, but dates do not match the 2026-06-20 J wave and therefore require original-file verification.

### Unresolved or explicitly missing deeper-source candidates

- [A] The explicitly named workflow/handoff/j-canvas-frontdoor-grounding.md is absent at enumeration time; locate a moved/renamed handoff through J-31/J-32 originals or repository history.
- [A] The '3-file VLM research corpus' behind J-8/J-9 is not named file-by-file in the summary. Inspect J-8, J-9, J-PERCEPTION-GROUNDING-MAP, and J-0 for the exact three paths.
- [A] The 'two production-AI talk summaries' behind J-56..J-59 are characterized but not individually path-named; inspect steal-production-ai-talks-2026-06-20.md.
- [A] J-77..J-79 refers to a 'vision report' by parts A/B/C; vision-and-orchestration-improvements-2026-06-23.md is a strong filename match but the numbered originals should confirm.
- [A] J-83..J-86 says nine papers were ingested but does not enumerate their interface filenames in the summary; inspect the GRC grounding map and reasoning source.
- [A] J-87..J-94 names four deep-research reports 7-10 and LACUNA-SWEEP but does not give report paths. LACUNA-SWEEP-2026-06-26.md exists; report identities require the Lacuna grounding map/program.
- [A] J-103..J-106 names Downloads/deep-research-report 11 and 12 without repository paths; resolve through the SDL grounding map.
- [A] J-107/J-108 says 12 ingested interface maps but names only the consolidated reasoning file; resolve through J-RLM-GROUNDING-MAP and originals.
- [A] J-111/J-112 names a voice-control call transcript and a same-day voice-mode spec-eval session without file paths. J-VOICE-GROUNDING-MAP is the only exact local source named.
- [E] dashboardRules.md is explicitly described as absent and is a future J-62 artifact, not a source to assume exists.
- [E] workflow/reasoning/J-71-cedar-spike.md is a planned output of the read-only spike, not a prerequisite source in this summary.
- [A] The stale line-2 note should not trigger a 'missing J-92..J-98' search: all seven numbered originals and all seven summary entries exist.

## Quantitative closeout

- Direct lines covered: 2,386 / 2,386.
- Numbered entries covered: 112 / 112, unique and exact-ID anchored.
- Numbered original files enumerated: 112 / 112 present.
- Grounding maps enumerated: 9.
- Atomic per-entry feature/behavior bullets: 831.
- Total tagged [E]/[I]/[A] coverage, feature, workflow, status, configuration, and source-ledger bullets: 942.
- No numbered original, grounding map, or supporting reasoning document was opened; this file is a direct-summary discovery layer intended to route deeper agents.
