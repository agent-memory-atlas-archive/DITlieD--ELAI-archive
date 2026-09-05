# DEEP-G-001 — exhaustive feature discovery

## Scope, method, and evidence contract

- Exact chunk: `DEEP-G-001`, family G, six numbered/index originals.
- Originals read completely and exclusively: `G-0_INDEX.md`, `G-1_taint_propagation_dataplane.md`, `G-3_blast_radius_gates.md`, `G-4_mcp_rugpull_repin.md`, `G-5_reviewer_cockpit_dsr.md`, and `G-7_confidence_producer_dual_llm.md` — **1,723/1,723 logical lines**.
- Orientation only: `07-G-SUMMARY_INDEX.discovery.md`. It is used solely to state additions/corrections in the summary-delta section; no summary-only detail is promoted into the source ledgers.
- No other G original, cross-plan original, implementation file, rationale paper, or grounding file was read. Paths named by a source remain source assertions, not independently verified code facts.
- Anchor notation: `G0:Lx-Ly`, `G1:Lx-Ly`, `G3:Lx-Ly`, `G4:Lx-Ly`, `G5:Lx-Ly`, and `G7:Lx-Ly` refer to the six manifested originals.
- Classification: `E` = explicit; `C` = explicit conflict/correction; `A` = unresolved ambiguity/open choice; `I` = inference isolated from the explicit inventory. `SHIPPED`, `PLANNED`, `WIRE-DARK`, `ABSENT`, `DEFERRED`, and “shipped end-state” are preserved rather than normalized.
- Atomic-record rule: every ledger ID is globally unique. A type/schema declaration is one atomic record even when its complete field list is preserved in that record; the audit therefore counts requirements/behaviors, not individual words or schema fields.

## Exact processed manifest and verification

| Chunk | Class | Manifest lines | Bytes | Manifest SHA-256 | Verification | Exact absolute path |
|---|---|---:|---:|---|---|---|
| DEEP-G-001 | numbered/index original | 203 | 16,661 | `9f4d4b682e5cb23e9f9de7693aa5983610f36b30dd82f8ef89c756c4224f7223` | exact | `docs/thefinalglance/G-plan/G-0_INDEX.md` |
| DEEP-G-001 | numbered/index original | 308 | 26,015 | `673d3ff40364f44dd1859de50a337f23b00190344740c835315878a72ebcc52d` | exact | `docs/thefinalglance/G-plan/G-1_taint_propagation_dataplane.md` |
| DEEP-G-001 | numbered/index original | 307 | 26,829 | `d2f67733390d909e731e64cd44da36a90f6be3d6d868b5740de4cf3275279453` | exact | `docs/thefinalglance/G-plan/G-3_blast_radius_gates.md` |
| DEEP-G-001 | numbered/index original | 252 | 20,236 | `65e86605f188ae78ce4971068727df0d6e40d4aefd59807e134f0dea046e9ef3` | exact | `docs/thefinalglance/G-plan/G-4_mcp_rugpull_repin.md` |
| DEEP-G-001 | numbered/index original | 293 | 26,579 | `b2bfd089120de0a6dcda991722f1e3953b772aaddf331056d4271185a4ede052` | exact | `docs/thefinalglance/G-plan/G-5_reviewer_cockpit_dsr.md` |
| DEEP-G-001 | numbered/index original | 360 | 34,726 | `9244c46c562faec04aa7f619b5fceb13c736e9fa07ff0ab450d29b0003a7139e` | exact | `docs/thefinalglance/G-plan/G-7_confidence_producer_dual_llm.md` |
| **Total** | 6 originals | **1,723** | **151,046** | — | exact aggregate | — |

Verification used logical newline counts, byte counts, and `sha256sum`; all six rows and both aggregates match `DEEP-SOURCE-MANIFEST.md` exactly.

## Per-file explicit ledger

### `G-0_INDEX.md` — governance, portfolio, and shared done gates

| Atomic ID | Class | Explicit feature / behavior | Area | Anchor |
|---|---|---|---|---|
| DG001-G0-E001 | E | G-plan owns the trust **READ / EVIDENCE / RUNTIME / MEASUREMENT** side and composes with F-plan’s WRITE-side gates. | Scope/architecture | G0:L1-L8, L17-L27 |
| DG001-G0-E002 | E | Its product surfaces are a reviewer cockpit, re-executable run artifact, and reproducible anti-gameable safety-delta measurement. | Capabilities | G0:L19-L27 |
| DG001-G0-E003 | E | The SSB SQLite-WAL append-only/no-UPDATE/no-DELETE journal plus `ELAI_SEED` is the shared credibility/replay substrate. | Data/determinism | G0:L23-L26 |
| DG001-G0-E004 | E | Existing `provenance.rs`, `capability_gate.rs`, `conformal_calibration.rs`, `refusal_sla.rs`, `mcp_schema_pin.rs`, and `sandbox/researcher_net.rs` substrate is characterized as shipped but mostly `WIRE-DARK`; G activates it rather than replacing it. | Status/dependencies | G0:L29-L36 |
| DG001-G0-E005 | E | Status vocabulary is `PLANNED`, `IN PROGRESS`, `DONE`, and `DEFERRED`; all nine original G sessions were `PLANNED` at generation. | Lifecycle/status | G0:L38-L43 |
| DG001-G0-E006 | E | `DONE` requires the full wiring gate; cargo-green is necessary but insufficient. | Done gate | G0:L41-L42 |
| DG001-G0-E007 | E | W-1: `wiring_contract_check.py --bn-file <G-N>.md --run-trigger` must PASS, produced symbols must exist, consumers resolve to CALL/REF, BFS must reach the entrypoint, stub bans must be clean, and the live observable effect must run and emit its SSB row and artifact. | Wiring/acceptance | G0:L49-L52 |
| DG001-G0-E008 | E | W-2: no default-off environment gate may be terminal; only a disable kill-switch is allowed, default is ON, and the observable trigger runs without an override. | Config/default | G0:L53-L54 |
| DG001-G0-E009 | E | W-3: reachability must start at a real CLI, FSM, PreToolUse/PostToolUse, or Tauri production entrypoint; a `#[cfg(test)]`-only caller is wire-dark. | Integration | G0:L55-L56 |
| DG001-G0-E010 | E | W-4: any dashboard-surfaceable SSB signal requires a frontend pair and `cross_lang_diff.py` PASS. | UI/schema | G0:L57 |
| DG001-G0-E011 | E | W-5: `observable_effect` is a live end-to-end trigger; RunPod work must provide a deterministic real-path fixture plus a separately gated live checkbox. | Verification/fallback | G0:L58-L59 |
| DG001-G0-E012 | E | W-6: at least one acceptance item must be verified through the dashboard using `dashboard-smoker`, or be marked N/A with one-line justification. | UI/acceptance | G0:L60 |
| DG001-G0-E013 | E | W-7: all consumed symbols, reachability anchors, and MODIFY targets must be grep-confirmed in `elai.map` at authoring and re-grepped by the validator. | Evidence/authoring | G0:L61-L62 |
| DG001-G0-E014 | E | W-8: every produced symbol needs an in-plan caller. | Dependency/invariant | G0:L63 |
| DG001-G0-E015 | E | W-9: new SSB action types pass the golden lock, use Pattern-2 single-variant-with-Kind where applicable, and name rather than hard-code the schema bump. | Schema/evolution | G0:L64-L65 |
| DG001-G0-E016 | E | W-10: network/subprocess/RunPod axes split into fail-closed in-process Tier 1 and separately gated Tier 2; a fail-open Tier 2 may not advertise fail-closed. | Tiering/claims | G0:L66-L68 |
| DG001-G0-E017 | E | GD-1 `COMPOSE-NOT-RESTATE`: each G item fills a read/evidence/runtime/measurement gap or consumes an F verdict; restating F-1..F-10 is a P0 incident recorded in `gotchas.md`. | Architecture/non-goal | G0:L70-L74 |
| DG001-G0-E018 | E | Named composable F verdicts include F-4 `stdout_hash`, F-1 `source_blake3`, F-9 `dep_name_hash`, and F-10 `delta_blake3`; each G plan names its consumed F verdict. | Evidence/dependencies | G0:L71-L74 |
| DG001-G0-E019 | E | GD-2: every emitted DSR/safety-delta/FP/confidence number must be recomputable from the SSB stream under a pinned `ELAI_SEED`; otherwise it is wire-dark. | Determinism/measurement | G0:L75-L77 |
| DG001-G0-E020 | E | GD-3: safety/performance results are same-model gate-ON versus gate-OFF marginal effects, never SWE-bench rank, model-capability claim, perfect-code promise, or proof that a reproduced run was right. | Evaluation/claim boundary | G0:L78-L80 |
| DG001-G0-E021 | E | GD-4: default-ON release is blocked until false-positive/over-block rate is measured and bounded on at least three real orchestrate runs. | Rollout/acceptance | G0:L81-L85 |
| DG001-G0-E022 | E | GD-5: fail-open, RunPod-pending, and cloud-opaque paths carry honest states (`runpod_pending`, action-stream-not-token reproducibility, advisory fail-open) and never advertise guarantees they lack. | Failure/claims | G0:L86-L88 |
| DG001-G0-E023 | E | Global completion additionally requires workspace cargo check green, sole-runner auditor PASS, wiring PASS, frontend cross-language PASS where applicable, dashboard E2E, recorded GD-4 bound, and memory-writer execution. | Done gate | G0:L90-L93 |
| DG001-G0-E024 | E | G-1 is the Wave-1, strong-grounding, large-effort harness-wide taint data-plane initiative. | Portfolio/status | G0:L96-L106 |
| DG001-G0-E025 | E | G-2 is the Wave-2, strong-grounding, medium-effort reproducible-run-artifact keystone. | Portfolio/status | G0:L96-L106 |
| DG001-G0-E026 | E | G-3 is the Wave-2, strong-grounding, medium-effort deterministic blast-radius initiative. | Portfolio/status | G0:L96-L106 |
| DG001-G0-E027 | E | G-4 is the Wave-1, strong-grounding, medium-effort live MCP re-pin initiative. | Portfolio/status | G0:L96-L106 |
| DG001-G0-E028 | E | G-5 is the Wave-2, strong-grounding, medium-effort cockpit/DSR initiative. | Portfolio/status | G0:L96-L106 |
| DG001-G0-E029 | E | G-6 is the Wave-3, strong-grounding, XL SAFEDELTA initiative. | Portfolio/status | G0:L96-L106 |
| DG001-G0-E030 | E | G-7 is the Wave-3, strong-grounding, large confidence-producer/quarantine initiative. | Portfolio/status | G0:L96-L107 |
| DG001-G0-E031 | E | G-8 is the Wave-2, strong-grounding, large signed-attestation/Article-50 initiative. | Portfolio/status | G0:L107-L108 |
| DG001-G0-E032 | E | G-9 is the Wave-2, moderate-grounding, medium journal-redaction/DB-TTL initiative. | Portfolio/status | G0:L108 |
| DG001-G0-E033 | E | G-10 is a post-G-9 Wave-3 extension for exogenous model drift, not one of R1..R9; it consumes D-7/G-2/F-4, detects by default, human-gates auto-pin, and distinguishes fingerprint-confirmed local drift from behaviorally suspected cloud drift. | Portfolio/status/config | G0:L113-L125 |
| DG001-G0-E034 | E | Planned SSB additions are `taint_flow`, `run_artifact_emitted`, `blast_radius_blocked`, `mcp_rug_pull_blocked`, `review_surface_scored`, deferred `safety_delta_measured`, `claim_confidence_scored`, `trifecta_quarantined`, `provenance_attestation_emitted`, optional `secret_redacted`, and `model_drift_suspected`; G-9 otherwise mutates the write path. | Events/schema | G0:L127-L140 |
| DG001-G0-E035 | E | `taint_flow` uses one Pattern-2 variant and ingest Kind `Web/ToolOutput/File/Sandbox`. | Schema | G0:L131 |
| DG001-G0-E036 | E | `blast_radius_blocked` uses Kind `FsEscape/EgressDenied` and stores full pre- and post-expansion command forms. | Schema/forensics | G0:L133 |
| DG001-G0-E037 | E | `claim_confidence_scored` and `trifecta_quarantined` are two G-7 action types, sharing Pattern-2 only if their payload permits. | Schema | G0:L137 |
| DG001-G0-E038 | E | Wave 1 co-ships G-4 and G-1 plus two citation fixes; G-1 unlocks G-3/G-7. | Sequencing/dependency | G0:L143-L147 |
| DG001-G0-E039 | E | Wave 2 starts with G-2, after which G-3/G-5/G-8/G-9 hang from its evidence manifest. | Sequencing/dependency | G0:L148-L149 |
| DG001-G0-E040 | E | Wave 3 contains G-7 and G-6; G-6 requires G-2 plus live F-2b, and no external safety-delta claim precedes the measured F-2b floor. | Sequencing/deferred | G0:L150-L152 |
| DG001-G0-E041 | E | The headline composition is G-2 RRA plus G-6 SAFEDELTA: independent replayability coupled to a falsifiable published measurement. | Product interaction | G0:L154-L156 |
| DG001-G0-E042 | E | Non-RFC hashing uses BLAKE3 only; BLAKE2/SHA2 are forbidden for those uses. | Security/config | G0:L159-L162 |
| DG001-G0-E043 | E | Production paths ban `.unwrap()`/`.expect()` except test cfg and main entrypoint allowances. | Reliability/constraint | G0:L161-L162 |
| DG001-G0-E044 | E | Dependencies require name/repository/version/date/license anti-slopsquat verification before addition. | Supply-chain security | G0:L163-L165 |
| DG001-G0-E045 | E | G-8 keeps cosign/sigstore opt-in with local hand-rolled BLAKE3-Merkle default; G-9 adds no Presidio/ggshield dependency; G-4 cannot add unverified `mcp-scan`. | Dependencies/defaults | G0:L163-L165 |
| DG001-G0-E046 | E | Work proceeds one change at a time; the auditor is sole test runner and the main LLM never runs tests. | Concurrency/process | G0:L166 |
| DG001-G0-E047 | E | External behavior claims must cite the trust synthesis/primary source and label unverified claims. | Evidence/compliance | G0:L167-L168 |
| DG001-G0-E048 | E | Presidio numeric claims are refuted; only qualitative high-recall/low-precision PII framing may be used and must be re-sourced. | Evidence correction | G0:L171-L174 |
| DG001-G0-E049 | E | Article-12 logging/Article-50 marking use the Article-99 EUR 15M/3% tier, not the Article-5 EUR 35M/7% tier. | Compliance correction | G0:L175-L176 |
| DG001-G0-E050 | E | CyberSecEval 1/2, not offensive-cyber-focused CyberSecEval 3, is the insecure-code-generation source. | Evidence correction | G0:L177-L178 |
| DG001-G0-E051 | E | The Anthropic-MCP claim must reference the stdio command-injection family/declined protocol fix, not LiteLLM CVE-2026-30623. | Evidence correction | G0:L179-L180 |
| DG001-G0-E052 | E | The named nondeterminism experiment used `Qwen3-235B-A22B-Instruct-2507`, not Qwen3-8B. | Evidence correction | G0:L181 |
| DG001-G0-E053 | E | BspEngine has a production caller; only `recover_from_checkpoint`/resume is a stub. | Status correction | G0:L182-L183 |
| DG001-G0-E054 | E | `mcp-scan` is Invariant Labs, not Snyk, and its distribution channel must be verified. | Attribution/supply-chain | G0:L184 |
| DG001-G0-E055 | E | Patch-N corrects CaMeL title/author while retaining arXiv 2503.18813, and re-confirms UniCR IDs before replacing wrong 2503.01345 with tentative 2509.01455. | Maintenance/evidence | G0:L185-L188 |
| DG001-G0-E056 | E | A shipped session logs the pipeline `context-compiler -> integration-explorer -> spec-grounder -> executor -> auditor -> fixer(if BLOCK) -> wiring validator -> dashboard smoker(if UI) -> memory-writer -> COMMITTING`. | Lifecycle/observability | G0:L190-L195 |
| DG001-G0-E057 | E | Immediate next action is co-shipped G-4/G-1 and citation patches, then G-2 to open Wave 2. | Status/sequence | G0:L197-L201 |

### `G-1_taint_propagation_dataplane.md` — live taint propagation

| Atomic ID | Class | Explicit feature / behavior | Area | Anchor |
|---|---|---|---|---|
| DG001-G1-E001 | E | Status is `PLANNED`, Wave 1, in-process and fail-closed; G-1 unlocks G-3 and G-7. | Status/dependency | G1:L1-L3 |
| DG001-G1-E002 | E | Reuse shipped `Provenance`, `Provenanced<T>`, `from_web`, `from_tool_output`, `from_file`, `from_sandbox`, `allow_tool_with_origin`, and `DESTRUCTIVE_TOOLS`; constructors are test-only/dead-wired at the stated baseline. | Existing capability/status | G1:L5-L16, L48-L54 |
| DG001-G1-E003 | E | Label every ingest byte, propagate only causally relevant provenance, and deterministically gate an Untrusted context before a destructive sink. | Core workflow/security | G1:L9-L16, L31-L36 |
| DG001-G1-E004 | E | G-1 is data-plane defense across tool-capable agents; F-3 remains the control-plane minimal-tool-set commitment using the Researcher role envelope. | Boundary/dependency | G1:L12-L14, L18-L21, L56-L62 |
| DG001-G1-E005 | E | E-37 `classify_injection`/`ClassifierVerdict` seeds an Untrusted label on a malicious span. | Integration/data flow | G1:L18-L21, L59-L60 |
| DG001-G1-E006 | E | Gate-time work is wholly W-10 Tier 1: no network, subprocess, or RunPod; estimated two sessions/eight-to-twelve hours. | Constraint/tiering | G1:L15-L16 |
| DG001-G1-E007 | E | G-1 is parallel-safe with G-4 and G-2, but not concurrent with edits to safety provenance/capability modules or the live dispatch fan-in. | Concurrency | G1:L22-L24 |
| DG001-G1-E008 | E | The intended information-flow label is a four-point integrity × confidentiality lattice, with P-T blocking Untrusted integrity and P-F restricting confidentiality egress. | Security model | G1:L38-L46 |
| DG001-G1-E009 | E | The dependency-screened join is mandatory; joining every context region is forbidden because unrelated content would over-taint the task. | Determinism/precision | G1:L38-L44 |
| DG001-G1-E010 | E | Existing `Provenance` surfaces also include `from_memory`, `user_prompt`, `system`, `is_trusted`, `is_untrusted`, `label_blake3_prefix`; `Provenanced<T>` exposes `new/trusted/value/into_value/provenance/is_untrusted/map`; types include `TrustLevel` and `TextSource`. | Existing API/schema | G1:L48-L53 |
| DG001-G1-C001 | C | Dated correction: `agent_pool.rs` is a sandbox subprocess launcher with no ingest/sink; real ingest is `web_research_orchestrator.rs:434`, `mcp_tool_bridge.rs:298`, `codeact_sandbox.rs:251`, and `production_agent_rpc.rs:1503`, while `production_agent_rpc.rs::translate_tool_request -> enforce_taint_sink_gate` is the live fan/sink. | Corrected integration | G1:L63-L70 |
| DG001-G1-E011 | E | `join_screened(relevant: &[&Provenance]) -> Provenance` computes the element-wise most restrictive join, with Untrusted dominating Trusted, over only dependency-relevant provenance. | API/algorithm | G1:L85-L94 |
| DG001-G1-E012 | E | `TaintedContext` carries the joined provenance keyed by task/turn ID. | State/schema | G1:L87-L92 |
| DG001-G1-E013 | E | `seed_from_verdict(&ClassifierVerdict) -> Provenance` bridges E-37 and forces Untrusted for a malicious verdict. | API/integration | G1:L91-L94 |
| DG001-G1-E014 | E | The relevant set is defined initially as bytes actually read by the in-flight tool call; it must not be all context. | Data-flow constraint | G1:L92-L94, L294-L295 |
| DG001-G1-E015 | E | The taint hot path uses existing `label_blake3_prefix` and contains no `.unwrap()`/`.expect()`. | Reliability/security | G1:L93-L94 |
| DG001-G1-E016 | E | WebFetch/WebSearch bodies use `Provenance::from_web(url)` and are Untrusted. | Ingest mapping | G1:L96-L100 |
| DG001-G1-E017 | E | MCP `tools/call` results use `from_tool_output(server_url, tool_name)` and are Untrusted. | Ingest mapping | G1:L96-L100 |
| DG001-G1-E018 | E | File reads use `from_file(path)`; committed `.elai_cc/` source is Trusted and other files are Untrusted. | Ingest mapping/security | G1:L96-L100 |
| DG001-G1-E019 | E | Sandbox/Bash stdout uses `from_sandbox(sandbox_id)` and is Untrusted. | Ingest mapping | G1:L99-L100 |
| DG001-G1-E020 | E | Every ingest wraps returned text in `Provenanced<String>`, registers/threads it into the current `TaintedContext`, and screened-joins the new label. | Workflow/state | G1:L96-L102 |
| DG001-G1-E021 | E | Emit one `taint_flow` SSB row per ingest with ingest Kind and resulting trust level. | Event/observability | G1:L101-L102 |
| DG001-G1-E022 | E | Before git commit/push, `.elai_cc/` Write, or Bash execution, invoke `allow_tool_with_origin(tool, &tainted_context.provenance())`. | Sink enforcement | G1:L104-L109 |
| DG001-G1-E023 | E | An Untrusted joined context yields Block and refuses the sink; there is no LLM override because verdict is pure set membership. | Security/failure | G1:L104-L109 |
| DG001-G1-A001 | A | The source permits either refusal or routing to an existing operator-confirm path but does not specify which sink takes which outcome or the confirmation state machine. | Ambiguous fallback | G1:L106-L109, L182-L188 |
| DG001-G1-E024 | E | `SsbActionType::TaintFlow` is a single Pattern-2 variant. | Event/schema | G1:L111-L115 |
| DG001-G1-E025 | E | `TaintFlowKind` has exactly `Web`, `ToolOutput`, `File`, and `Sandbox`. | Event/schema | G1:L111-L115 |
| DG001-G1-E026 | E | `TaintFlowPayload` fields are `kind`, BLAKE3 `origin_blake3`, `trust_level`, optional `sink_tool`, `blocked`, and `task_id`. | Event/schema/privacy | G1:L111-L115 |
| DG001-G1-E027 | E | Event integration includes SQL string `taint_flow`, parse arm, SQL CHECK, round-trip golden, action-type-lock golden, and named schema `baseline+1` (approximately 199 at authoring). | Schema lifecycle/tests | G1:L113-L115 |
| DG001-G1-E028 | E | `ELAI_TAINT_PROPAGATE_DISABLE=1` is the only kill-switch; default is ON and live evidence runs with no override; an ENABLE flag is banned. | Flag/default | G1:L117-L120, L171-L178 |
| DG001-G1-E029 | E | Release requires measured over-block rate on at least three real orchestrate runs plus a ten-benign-flow contract with no Trusted-flow blocks. | FP release gate | G1:L117-L120 |
| DG001-G1-E030 | E | `TaintFlowDrawer.svelte`, Tauri `get_taint_flows`, and drawer key `taint-flow` list rows by Kind/trust/block state; `cross_lang_diff.py` must PASS. | UI/surface | G1:L122-L124 |
| DG001-G1-E031 | E | `G-1-decisions.md` records screened-join choice, four-site mapping, FIDES P-T reuse, G-1/F-3 composition, and the measured over-block rate. | Evidence/decision log | G1:L126-L129 |
| DG001-G1-E032 | E | Produced safety identifiers are `join_screened`, `TaintedContext`, and `seed_from_verdict`. | Produced API | G1:L131-L138 |
| DG001-G1-E033 | E | Produced event identifiers are `SsbActionType::TaintFlow`, `TaintFlowKind`, and `TaintFlowPayload`. | Produced schema | G1:L131-L138 |
| DG001-G1-E034 | E | Produced UI identifiers are `commands_g1.rs::get_taint_flows` and `TaintFlowDrawer.svelte`. | Produced surface | G1:L139-L140 |
| DG001-G1-E035 | E | Consumed security identifiers are the provenance constructors/checks, `Provenanced`, `allow_tool_with_origin`, `DESTRUCTIVE_TOOLS`, `classify_injection`, and `ClassifierVerdict`. | Dependencies | G1:L141-L147 |
| DG001-G1-E036 | E | Corrected live integration additionally consumes process-global `tainted_context`, four real ingest modules, `enforce_taint_sink_gate`, `from_file_screened`, and `taint_flow_ssb_adapter.rs::emit_taint_flow`. | Dependencies/integration | G1:L148-L157 |
| DG001-G1-E037 | E | Web ingress reaches `translate_tool_request`, the edit/git/Bash arms, `allow_tool_with_origin`, rejection, `emit_taint_flow`, and live `Ssb::write_payload(TaintFlow)`; production SSB is installed through `.with_ssb`. | End-to-end workflow | G1:L158-L166 |
| DG001-G1-E038 | E | MCP ingress follows the same process-global side-table and sink chain; stdio is covered through `McpServerManager` delegation. | End-to-end workflow | G1:L167-L169 |
| DG001-G1-E039 | E | Unknown/unresolvable provenance defaults to Untrusted and blocks; silent fail-open is forbidden. | Failure/fallback | G1:L171-L178 |
| DG001-G1-E040 | E | Stub bans also prohibit “wire later” literals, global join, LLM bypass, BLAKE2/SHA2, and default-off enable flags. | Non-goals/security | G1:L171-L178 |
| DG001-G1-E041 | E | Observable fixture injects Web/MCP text requesting `git push --force` with no env override. | Acceptance trigger | G1:L182-L184 |
| DG001-G1-E042 | E | Expected evidence is a live `taint_flow` row with Web/ToolOutput Kind, Untrusted trust, `blocked=true`, destructive `sink_tool`, and populated BLAKE3 origin. | Acceptance evidence | G1:L185-L186 |
| DG001-G1-E043 | E | Expected artifact is no destructive effect, a benign Trusted flow unblocked in the same run, and drawer visibility. | Acceptance/fallback | G1:L187-L188 |
| DG001-G1-E044 | E | Deterministic checks grep every produced identifier and corrected ingest/sink wire, require the event/golden/UI/decision file, run the wiring validator and cross-language diff, and require zero ENABLE/BLAKE2/SHA2 matches. | Verification | G1:L190-L209 |
| DG001-G1-E045 | E | Acceptance requires live wiring/BFS/stub/effect PASS, default-on behavior, production `translate_tool_request`, dashboard rendering, screened-join contract, event round-trip, workspace cargo green, auditor PASS, and no new dependency. | Done gate | G1:L211-L227 |
| DG001-G1-E046 | E | The screened-join contract asserts a relevant-subset join is no more restrictive than the global join and does not taint an unrelated clean region. | Test/invariant | G1:L223-L224 |
| DG001-G1-E047 | E | The main contract covers web Untrusted→block/event, committed-file Trusted→allow, screened precision, malicious verdict seed, ten benign flows/zero blocks, and all four event Kind round trips. | Tests | G1:L254-L260 |
| DG001-G1-E048 | E | Fixtures include at least ten injected Web/MCP cases and ten benign cases. | Test data | G1:L260 |
| DG001-G1-E049 | E | Planned creates are safety taint module, G1 Tauri command, drawer, contract/fixtures, and decision doc; planned modifications include safety lib, SSB/golden, dashboard types/host, and a dispatch implementation. | Change surfaces | G1:L262-L273 |
| DG001-G1-C002 | C | The “Files touched” block still says to modify `agent_pool.rs` for all four ingests/sink, conflicting with the dated correction that `agent_pool.rs` has none of them. | Stale plan conflict | G1:L63-L70, L262-L273 |
| DG001-G1-C003 | C | The mandatory-read list, Task 2 wording, test-risk text, and stale file list still describe `agent_pool.rs` as the ingest fan, while the wiring contract/done-gate explicitly repoint to four other modules and `production_agent_rpc`. | Internal conflict | G1:L73-L82, L96-L102, L148-L170, L240-L247, L270-L271, L296-L297 |
| DG001-G1-E050 | E | No dependency is added; existing safety types and BLAKE3 are reused and any future add requires anti-slopsquat verification. | Dependency constraint | G1:L225-L227, L288 |
| DG001-G1-E051 | E | The plan does not claim RTBAS/FIDES effect sizes for ELAI; both are preprints and ELAI’s own over-block rate is unverified until measured. | Evidence/claim boundary | G1:L275-L299 |
| DG001-G1-A002 | A | The exact dependency-relevant-set algorithm remains the hard risk: too narrow misses attacks, too broad destroys utility; the decision doc must fix the chosen screen. | Ambiguity/risk | G1:L290-L295 |
| DG001-G1-C004 | C | The source simultaneously labels G-1 `PLANNED` and names “SHIPPED reality”/live concrete anchors plus correction records dated after authoring; it does not explicitly declare the overall initiative DONE. | Status conflict | G1:L3, L63-L70, L217-L250 |
| DG001-G1-E052 | E | Downstream consumers are G-3 scoring, G-7 lethal-trifecta detection/quarantine, G-2 RRA journaling, and the F-3+G-1 combined control/data-plane defense. | Interactions | G1:L301-L306 |

### `G-3_blast_radius_gates.md` — post-expansion filesystem and pinned-egress gates

| Atomic ID | Class | Explicit feature / behavior | Area | Anchor |
|---|---|---|---|---|
| DG001-G3-E001 | E | Status is `PLANNED`, Wave 2, in-process/fail-closed; it consumes G-1 live taint and composes with shipped E-30 argv gating. | Status/dependency | G3:L1-L3 |
| DG001-G3-E002 | E | Gate A validates expanded filesystem write/delete targets against the per-agent worktree before the verb executes. | Capability/security | G3:L5-L13 |
| DG001-G3-E003 | E | Gate B rejects null/control-character hosts and requires resolved IP membership in a pinned allow-set before connect. | Capability/security | G3:L5-L10 |
| DG001-G3-E004 | E | Both gates emit one `blast_radius_blocked` row with full pre- and post-expansion command forms. | Observability/forensics | G3:L9-L10 |
| DG001-G3-E005 | E | E-30 owns raw, whitespace-split, pre-expansion verb policy; G-3 owns expanded path/host policy and does not reimplement E-30. | Architecture/non-goal | G3:L11-L15, L77-L83 |
| DG001-G3-E006 | E | Decisions are pure set/prefix membership and never delegated to an LLM permission classifier. | Determinism/security | G3:L13-L15 |
| DG001-G3-E007 | E | Gate-time work is wholly Tier 1 with no subprocess or RunPod; estimated one-to-two sessions/eight-to-twelve hours. | Constraint/tiering | G3:L15-L16 |
| DG001-G3-E008 | E | Required shipped anchors are E-30 `check_argv`, `AllowlistVerdict`, `DENY_PATTERNS`, `argv_blake3`, live `dispatch_pre_tool_use`, Linux `allowed_hosts`/iptables setup, and post-G-1 provenance/capability surfaces. | Existing capabilities | G3:L18-L24 |
| DG001-G3-E009 | E | Work is parallel-safe with G-2/G-4 but not with concurrent edits to the PreToolUse dispatcher, sandbox networking modules, or SSB write path. | Concurrency | G3:L25-L27 |
| DG001-G3-E010 | E | E-30’s current raw split misses `$HOME`, glob, tilde, and symlink expansion and does not log what the shell ultimately expands. | Gap/forensics | G3:L30-L37 |
| DG001-G3-E011 | E | Linux networking currently resolves/pins IPs via veth+iptables OUTPUT while seccomp opens AF_INET/AF_INET6 STREAM/DGRAM and explicitly does not filter egress. | Existing state/security | G3:L38-L42, L54-L59 |
| DG001-G3-E012 | E | `ResearcherNetConfig.allowed_hosts` defaults to `arxiv.org` and `huggingface.co`; OUTPUT is default-deny REJECT with allowed IP ACCEPT rules. | Config/default/network | G3:L54-L57 |
| DG001-G3-E013 | E | The networking substrate is a non-Linux no-op. | Platform/fallback | G3:L54-L58 |
| DG001-G3-E014 | E | E-30 currently produces only `HookFinding` rule `e30-argv-deny` plus trace, so G-3 is the first typed SSB FS/egress-block event. | Status/observability | G3:L59-L63 |
| DG001-G3-E015 | E | Path-containment precedent is `ensure_within_project_root` canonicalize-parent-plus-rejoin; worktree root is agent `scratch_dir`. | Dependency/path handling | G3:L71-L73 |
| DG001-G3-E016 | E | Landlock and macOS Seatbelt are absent; Windows AppContainer exists but lacks the relevant path allowlist/egress pin. | Platform/status | G3:L73-L75 |
| DG001-G3-E017 | E | Authoring baseline is `SCHEMA_VERSION=205` and `TOTAL_VARIANTS=482`. | Schema status | G3:L74-L75 |
| DG001-G3-E018 | E | `check_fs_target(expanded_target: &Path, worktree_root: &Path) -> BlastRadiusVerdict` rejects path null/control characters before canonicalization. | API/input validation | G3:L108-L117 |
| DG001-G3-E019 | E | Existing targets are canonicalized; non-existent targets canonicalize the parent and rejoin the final component. | Path lifecycle | G3:L108-L113 |
| DG001-G3-E020 | E | A canonical target outside canonical worktree root yields `Block { kind: FsEscape }`. | Security/verdict | G3:L112-L113 |
| DG001-G3-E021 | E | `check_egress_host(host: &str, allowed: &[IpAddr]) -> BlastRadiusVerdict` validates characters, resolves, and allows only resolved-IP membership; otherwise it yields `EgressDenied`. | API/network | G3:L113-L115 |
| DG001-G3-E022 | E | `BlastRadiusVerdict` is `Allow` or `Block { kind: BlastRadiusKind, pre_expansion: String, post_expansion: String }`. | Schema/verdict | G3:L115-L116 |
| DG001-G3-E023 | E | Uncanonicalizable targets and unresolvable hosts block; there is no silent fail-open and no production `.unwrap()`/`.expect()`. | Failure/recovery | G3:L116-L117, L174-L181 |
| DG001-G3-E024 | E | After E-30, the live dispatcher shell-expands argv, checks every write/delete target and fetch/connect host, returns blocking `HookFinding`, and emits SSB on Block. | Runtime workflow | G3:L119-L125 |
| DG001-G3-E025 | E | `ELAI_BLAST_RADIUS_DISABLE` is the sole kill-switch; default is ON, no override is used for evidence, and an ENABLE flag is banned. | Flag/default | G3:L123-L125, L136-L139 |
| DG001-G3-E026 | E | `SsbActionType::BlastRadiusBlocked` is one Pattern-2 variant with `BlastRadiusKind::{FsEscape,EgressDenied}`. | Event/schema | G3:L127-L134 |
| DG001-G3-E027 | E | `BlastRadiusBlockedPayload` fields are `kind`, `pre_expansion_command`, `post_expansion_command`, `target_or_host`, BLAKE3 `worktree_root_blake3`, G-1 `trust_label`, and `ts_ms`. | Event schema/privacy | G3:L127-L134 |
| DG001-G3-E028 | E | Event lifecycle includes `blast_radius_blocked` string/parse/CHECK/round-trip/action lock, schema 205→206, variants 482→483, ALL_VARIANTS row, and golden regeneration. | Schema evolution | G3:L127-L134 |
| DG001-G3-E029 | E | Release requires over-block measurement on at least three real runs and ten benign in-worktree writes/allowed-host fetches with zero blocks. | FP gate/tests | G3:L136-L139 |
| DG001-G3-E030 | E | `BlastRadiusDrawer.svelte`, `get_blast_radius_blocks`, and drawer key `blast-radius` show Kind and a pre/post diff; cross-language diff must PASS. | UI/surface | G3:L141-L144 |
| DG001-G3-E031 | E | `G-3-decisions.md` records deterministic-versus-LLM rationale, E-30 delta, platform scope, E-30/G-1 composition, and measured over-block rate. | Evidence/decision log | G3:L146-L150 |
| DG001-G3-E032 | E | Produced safety identifiers are `check_fs_target`, `check_egress_host`, and `BlastRadiusVerdict`. | Produced API | G3:L152-L159 |
| DG001-G3-E033 | E | Produced event identifiers are `SsbActionType::BlastRadiusBlocked`, `BlastRadiusKind`, and `BlastRadiusBlockedPayload`. | Produced schema | G3:L152-L159 |
| DG001-G3-E034 | E | Produced UI identifiers are `commands_g3.rs::get_blast_radius_blocks` and `BlastRadiusDrawer.svelte`. | Produced surface | G3:L160-L161 |
| DG001-G3-E035 | E | Consumed identifiers include live `dispatch_pre_tool_use`, E-30 `check_argv`, network `allowed_hosts`/`apply_iptables_output_allow`, agent `scratch_dir`, G-1 `Provenanced`/`allow_tool_with_origin`, and two SSB schema precedents. | Dependencies | G3:L162-L169 |
| DG001-G3-E036 | E | Production reachability is `dispatch_pre_tool_use -> check_fs_target/check_egress_host -> HookFinding Block + SSB`, with Tauri-to-drawer dashboard reachability. | Workflow/integration | G3:L170-L173 |
| DG001-G3-E037 | E | Stub bans prohibit LLM classifier, pre-expansion-only implementation, default-off ENABLE, silent fail-open, Landlock/Seatbelt claims, BLAKE2/SHA2, nonexistent `pretool_argv.rs`, and nonexistent `destructive_verb_denied`. | Constraints/non-goals | G3:L174-L182 |
| DG001-G3-E038 | E | Observable fixture uses `rm -rf $X` expanding outside the worktree or a null-byte-spoofed fetch, with no env override. | Acceptance trigger | G3:L186-L188 |
| DG001-G3-E039 | E | Evidence row must have correct Kind, both command forms, and BLAKE3 root; the file/connect side effect must not occur, benign controls remain allowed, and UI shows the diff. | Acceptance evidence | G3:L189-L192 |
| DG001-G3-E040 | E | Deterministic checks grep APIs/wiring/event/golden/UI/decision doc, run validator/cross-language diff, require zero ENABLE/BLAKE2/SHA2 and zero Landlock/Seatbelt matches. | Verification | G3:L194-L207 |
| DG001-G3-E041 | E | Acceptance requires live wiring, default-on, production PreToolUse, dashboard E2E, populated/different command fields, pre-resolve host rejection, expanded-path escape block, FP bound, exact schema/variant bump, cargo/auditor green, and no new dependency. | Done gate | G3:L209-L226 |
| DG001-G3-E042 | E | Contract cases cover escaped `rm -rf`, in-worktree allow, null/control host pre-resolve denial, non-allowlisted IP denial, allowlisted host allow, ten benign flows, both Kind round trips, and uncanonicalizable fail-closed. | Tests | G3:L250-L255 |
| DG001-G3-E043 | E | Fixtures contain at least ten escape and ten benign cases. | Test data | G3:L256 |
| DG001-G3-E044 | E | Planned files create the gate module, G3 UI pair, contract/fixtures, and decision doc; modify safety lib, hook dispatcher, SSB/locks, and dashboard registration. | Change surfaces | G3:L258-L269 |
| DG001-G3-E045 | E | No new dependency is expected: reuse `std::fs`, BLAKE3, and existing resolver; any add requires anti-slopsquat verification. | Dependency constraint | G3:L225-L226, L285 |
| DG001-G3-C001 | C | Contrary to the summary pass, this original repeatedly says Landlock is absent and explicitly bans any claim it is used; the implemented floor is OS-agnostic `std::fs` plus host check, with Linux network backstop only from existing veth/iptables. | Source correction | G3:L73-L75, L146-L150, L174-L181, L271-L284 |
| DG001-G3-A001 | A | The source asserts `std::fs::canonicalize` resolves tilde/glob/symlink, but later says the hard open question is where to obtain the shell-expanded argv and forbids silently reimplementing expansion; the expansion producer and quoting semantics are unresolved. | Ambiguity/implementation gap | G3:L44-L48, L108-L112, L287-L294 |
| DG001-G3-A002 | A | Worktree containment may need `declared allowed_paths`, but no schema/owner for such paths is defined. | Ambiguity/config | G3:L287-L291 |
| DG001-G3-E046 | E | Non-Linux kernel confinement parity is not claimed; std::fs/host checks are the platform floor and Windows/macOS kernel-level protection remains outside scope. | Platform/claim boundary | G3:L295-L296 |
| DG001-G3-E047 | E | Current code-ground-truth warnings are: no `pretool_argv.rs`, no `destructive_verb_denied`, no central `worktree_root()` helper, and baselines are 205/482 rather than stale 191/464. | Status correction | G3:L297-L299 |
| DG001-G3-E048 | E | Downstream, G-2 records blocks, G-5 displays them, G-1 enriches trust labels, and G-3+E-30 form verb-plus-expanded-target protection. | Interactions | G3:L301-L305 |

### `G-4_mcp_rugpull_repin.md` — MCP schema/description re-verification

| Atomic ID | Class | Explicit feature / behavior | Area | Anchor |
|---|---|---|---|---|
| DG001-G4-E001 | E | Status is `PLANNED`, Wave 1, no-dependency, cheapest high-grounding extension of the shipped schema pin. | Status | G4:L1-L3 |
| DG001-G4-E002 | E | Existing `pin_or_verify`, `hash_tool_list`, `PinStatus`, and `mcp_schema_pin_violation` provide first-connect TOFU only. | Existing capability/gap | G4:L5-L11 |
| DG001-G4-E003 | E | A post-approval tool description or schema swap is a rug pull/line-jump because model-visible malicious instructions can hide in descriptions. | Threat model | G4:L7-L11, L24-L30 |
| DG001-G4-E004 | E | Recompute the existing BLAKE3 schema hash on a later tool-list observation and block a mismatch fail-closed pending explicit operator re-approval. | Core behavior/security | G4:L9-L11 |
| DG001-G4-E005 | E | Emit distinct `mcp_rug_pull_blocked` evidence with before/after description-change information. | Observability | G4:L10-L11 |
| DG001-G4-E006 | E | Gate-time work is Tier 1, pure in-process, no RunPod/new dependency, and uses only the MCP connection ELAI already holds; estimated five-to-eight hours. | Constraint/tiering | G4:L12-L13 |
| DG001-G4-E007 | E | Existing exact anchors include `PinStatus`, `SchemaPinError`, `PinRecord`, `ToolDef`, `hash_tool_list`, `pin_or_verify`, and `update_pin`; E-24 version pin and E-43 skill-import scan are shipped siblings. | Dependencies/status | G4:L15-L19 |
| DG001-G4-E008 | E | Parallel-safe with G-1/G-2 but not concurrent with edits to `mcp_schema_pin.rs` or the live MCP discovery handler. | Concurrency | G4:L20-L21 |
| DG001-G4-E009 | E | Do not add `mcp-scan` without distribution verification; attribute it to Invariant Labs, not Snyk. | Supply-chain/evidence | G4:L26-L33 |
| DG001-G4-E010 | E | Do not attribute LiteLLM CVE-2026-30623 to Anthropic; the separate claim is the stdio command-injection family and declined protocol fix. | Evidence correction | G4:L31-L33 |
| DG001-G4-E011 | E | Runtime rug-pull evidence remains distinct from first-connect `mcp_schema_pin_violation`; the existing pin machinery is reused, not reimplemented. | Event boundary/GD-1 | G4:L35-L48 |
| DG001-G4-E012 | E | MCP trust is a three-bracket sequence: E-43 install scan, first-connect schema/version pin, then G-4 runtime/reconnect re-verification. | Lifecycle/integration | G4:L49-L54 |
| DG001-G4-E013 | E | `reverify_on_list_changed(workspace_root, server_url, new_tools: &[ToolDef], prior: &PinRecord) -> RugPullVerdict` recomputes the hash and compares to the prior record. | API | G4:L65-L70 |
| DG001-G4-E014 | E | `RugPullVerdict` is `Unchanged` or `Changed { description_diffs: Vec<ToolDescDiff>, schema_blake3_before, schema_blake3_after }`. | Verdict schema | G4:L67-L71 |
| DG001-G4-E015 | E | `ToolDescDiff { tool_name, before, after }` is computed per tool so same-name description swaps surface. | Diff schema/security | G4:L70-L72 |
| DG001-G4-E016 | E | On `Changed`, changed tools remain blocked; `update_pin` executes only after explicit operator re-approval and never automatically. | State/lifecycle/recovery | G4:L72-L74 |
| DG001-G4-E017 | E | The re-verification path uses existing BLAKE3 and contains no production `.unwrap()`/`.expect()`. | Reliability | G4:L73-L74 |
| DG001-G4-E018 | E | The authored task says a `notifications/tools/list_changed` handler calls re-verification, emits the event, and blocks changed tools. | Authored integration | G4:L76-L81 |
| DG001-G4-E019 | E | `ELAI_MCP_REPIN_DISABLE` is the only kill-switch; default is ON, trigger uses no override, no ENABLE flag exists, and prompts cannot bypass a hash mismatch. | Flags/security | G4:L79-L81, L124-L131 |
| DG001-G4-E020 | E | `SsbActionType::McpRugPullBlocked` has payload `server_url`, `schema_blake3_before`, `schema_blake3_after`, `changed_tools: Vec<String>`, and `description_diff_count: u32`. | Event schema | G4:L83-L87 |
| DG001-G4-E021 | E | Event lifecycle includes string `mcp_rug_pull_blocked`, parse, CHECK, round-trip golden, action lock, and named `baseline+1` schema bump. | Schema lifecycle | G4:L83-L87 |
| DG001-G4-E022 | E | `McpRugPullDrawer.svelte`/existing MCP drawer plus `get_mcp_rug_pulls` render a before/after description diff and must pass cross-language diff. | UI/surface | G4:L89-L92 |
| DG001-G4-E023 | E | `G-4-decisions.md` records the TOFU gap, description-diff signal, human-only pin update, attribution/CVE landmines, and three-bracket model. | Evidence/decision log | G4:L94-L97 |
| DG001-G4-E024 | E | Produced pin identifiers are `reverify_on_list_changed`, `RugPullVerdict`, and `ToolDescDiff`. | Produced API | G4:L99-L105 |
| DG001-G4-E025 | E | Produced event/UI identifiers are `SsbActionType::McpRugPullBlocked`, `McpRugPullBlockedPayload`, `get_mcp_rug_pulls`, and `McpRugPullDrawer.svelte`. | Produced schema/surface | G4:L104-L107 |
| DG001-G4-E026 | E | Consumed pin identifiers are `hash_tool_list`, `pin_or_verify`, `update_pin`, `PinRecord`, `ToolDef`, and `PinStatus`. | Dependencies | G4:L108-L114 |
| DG001-G4-E027 | E | Corrected integration consumes `mcp_rugpull_gate.rs::run_stdio_repin_gate` and `mcp_rugpull_ssb_adapter.rs::OrchestratorRugPullSsbEmitter`. | Corrected dependency | G4:L115-L118 |
| DG001-G4-C001 | C | Dated correction says the HTTP/server-push `tools/list_changed` handler does not exist; production instead re-fetches at `mcp_stdio.rs::McpServerManager::discover_tools_for_server` on stdio reconnect, then runs `run_rug_pull_gate_for_server -> run_stdio_repin_gate`. | Corrected trigger | G4:L117-L123, L135-L147 |
| DG001-G4-E028 | E | Corrected observable flow is first discovery/pin, server restart/reconnect with swapped description, second discovery/reverification, emitter event, and tool block. | Workflow/state | G4:L135-L147 |
| DG001-G4-E029 | E | Expected row requires before/after hashes to differ, changed tool name, and `description_diff_count >= 1`; expected artifact is blocked tool until approval plus rendered diff. | Acceptance evidence | G4:L143-L147 |
| DG001-G4-E030 | E | Stub bans prohibit auto-pin, default-off ENABLE, LLM bypass, BLAKE2/SHA2, unverified `mcp-scan`, and “later” placeholders. | Security/non-goals | G4:L124-L131 |
| DG001-G4-E031 | E | Deterministic checks validate APIs, corrected stdio wire, event/golden/UI/decision doc, zero ENABLE/BLAKE2/SHA2, wiring PASS, and cross-language PASS. | Verification | G4:L149-L162 |
| DG001-G4-E032 | E | Acceptance requires live block/event, default-on, production entrypoint, dashboard diff, no automatic `update_pin`, same-name description detection, distinct event round-trip, cargo/auditor green, and no new dependency. | Done gate | G4:L164-L178 |
| DG001-G4-C002 | C | Acceptance line 169 still requires a live `tools/list_changed` handler even though the dated correction says no such handler exists and repoints W-3 to reconnect discovery. | Stale acceptance conflict | G4:L164-L170, L180-L197 |
| DG001-G4-C003 | C | Task 2, the mandatory-read block, files-touched block, and open risk still target `crates/plugins/src/mcp/*` push notification, conflicting with the corrected stdio discovery funnel. | Stale plan conflict | G4:L56-L63, L76-L81, L215-L225, L237-L242 |
| DG001-G4-C004 | C | `run_stdio_repin_gate` is listed under `consumes_symbols` but later W-1 calls it “produced”; it is absent from the declared `produces_symbols` list. | Contract conflict | G4:L99-L116, L180-L200 |
| DG001-G4-A001 | A | The SSB payload stores hashes, changed tool names, and a diff count, but not `ToolDescDiff.before/after`; the source nevertheless requires the dashboard and event to carry/render the actual before/after description diff, without naming its persistence/read path. | Schema/UI ambiguity | G4:L70-L72, L83-L92, L143-L147 |
| DG001-G4-A002 | A | The source says to reuse an operator-confirm path but does not identify its command/UI, approval record, actor authorization, or durable transition from blocked to updated pin. | Permission/lifecycle ambiguity | G4:L72-L80, L171-L174 |
| DG001-G4-E033 | E | W-10’s corrected production semantics are in-process hash comparison on existing stdio discovery, fail-closed on mismatch, with no new network/subprocess/RunPod at gate time. | Tiering/failure | G4:L180-L203 |
| DG001-G4-E034 | E | GD-4 risk is a legitimate schema update; mitigation is visible diff plus operator re-approval and measuring legitimate-update frequency before default-on. | FP gate/recovery | G4:L204-L206, L237-L240 |
| DG001-G4-A003 | A | Unlike inherited G0 GD-4, this file gives no explicit “at least three real runs” sample requirement or quantitative acceptable legitimate-update threshold. | Rollout ambiguity | G4:L204-L206, L237-L240 |
| DG001-G4-E035 | E | Contract cases cover unchanged/no event, same-name description change/event/block, no auto-update, explicitly approved legitimate update, and event round trip. | Tests | G4:L208-L213 |
| DG001-G4-E036 | E | Test fixtures are a first-connect list plus a post-approval swapped-description list. | Test data | G4:L213 |
| DG001-G4-E037 | E | Planned creates are G4 UI pair, contract/fixtures, and decision doc; authored modifications are schema-pin, MCP handler, SSB/golden, and dashboard types/host. | Change surfaces | G4:L215-L225 |
| DG001-G4-E038 | E | Stdio command injection is a separate, explicitly out-of-scope MCP threat and must not be conflated with schema/description swaps. | Non-goal | G4:L241-L244 |
| DG001-G4-C005 | C | Overall status remains `PLANNED`, while later corrections name concrete live modules/emitter and speak in shipped-ground-truth terms; the file never explicitly changes overall status to DONE. | Status conflict | G4:L3, L115-L123, L180-L203 |
| DG001-G4-E039 | E | Downstream, G-2 records rug-pull blocks, G-5 displays them, and G-1 treats changed-tool output as Untrusted. | Interactions | G4:L246-L250 |

### `G-5_reviewer_cockpit_dsr.md` — evidence-backed review surface and DSR

| Atomic ID | Class | Explicit feature / behavior | Area | Anchor |
|---|---|---|---|---|
| DG001-G5-E001 | E | Header status is `PLANNED`, Wave 2, with stated G-2 RRA dependency and F-4/F-1/grounding-row consumption. | Status/dependency | G5:L1-L3 |
| DG001-G5-E002 | E | The cockpit renders deterministic proof tokens per hunk so vouched hunks can be skimmed and full attention focuses on unvouched hunks. | Capability/workflow | G5:L5-L10 |
| DG001-G5-E003 | E | Vouch sources are F-4 stdout-hash-bound passing tests, F-1/F-2a gate verdicts including CWE, and grounding-sweep citations. | Evidence/integration | G5:L5-L7, L17-L20 |
| DG001-G5-E004 | E | DSR is `lines needing full attention / total changed lines`. | Metric/schema | G5:L8-L9 |
| DG001-G5-E005 | E | DSR must be recomputed deterministically/byte-identically from the SSB journal under GD-2. | Determinism | G5:L8-L14 |
| DG001-G5-E006 | E | Findings are silent by default and surface only for a deterministic gate verdict or failing test, never model-only opinion. | Signal policy/default | G5:L9-L11 |
| DG001-G5-E007 | E | The current journal has per-hunk verdict identity but no changed-line-count denominator, so G-5 must persist that denominator and may not claim preexisting auditability. | Data gap/claim boundary | G5:L11-L14, L40-L47 |
| DG001-G5-E008 | E | The within-subjects human review study is separate measurement work, not the implementation done-gate. | Deferred/non-goal | G5:L14-L15 |
| DG001-G5-E009 | E | Existing sources are `TestOutcomePayload.stdout_hash`, `VibeGateBlockedPayload`/`SecurityRuleBlockedPayload` CWE, `GroundingSweepResultPayload`, and `ChangeDecisionPayload.hunk_id`. | Dependencies/evidence | G5:L17-L20 |
| DG001-G5-E010 | E | Existing UI is `ReviewWindow.svelte -> get_file_changes -> SsbViewer::file_changes`; G-2 `rra.rs` was absent at the authoring baseline. | Existing surface/status | G5:L20-L24 |
| DG001-G5-E011 | E | G-5 is parallel-safe with G-4/G-8/G-9, not with G-2 or concurrent changes to FileWrite/SSB write path. | Concurrency | G5:L23-L24 |
| DG001-G5-E012 | E | Review-time evidence is deliberately deterministic because naive LLM review is treated as noisy/style-biased and weak on SQLi/XSS. | Product rationale/constraint | G5:L27-L38 |
| DG001-G5-E013 | E | `ChangeDecisionPayload.hunk_id` is derived from a unified-diff `@@ -N,M +P,Q @@` header hash and provides the hunk-level precedent/numerator join. | Data schema | G5:L40-L42 |
| DG001-G5-E014 | E | Existing `FileWritePayload` is only `{path,size_bytes,hash}` and `FileChange` is only `{path,action,summary}`; neither persists changed-line count. | Data status | G5:L42-L46 |
| DG001-G5-E015 | E | Existing dashboard access projects `has_stdout_hash` but lacks a dedicated `SsbViewer` TestOutcome reader. | Integration gap | G5:L49-L52 |
| DG001-G5-E016 | E | Authored CLI attach point is hand-rolled `args.rs::parse_args`/subcommand matching, mirroring the `run` parser arm. | CLI integration | G5:L60-L63 |
| DG001-G5-E017 | E | Authoring baseline is schema 205 / variants 482. | Schema status | G5:L62-L63 |
| DG001-G5-E018 | E | `HunkVouch` is `{ hunk_id: String, vouched_by: Vec<VouchKind>, lines_changed: u32, needs_attention: bool }`. | Data schema | G5:L91-L99 |
| DG001-G5-E019 | E | `VouchKind` is `TestStdoutHash`, `WriteGateVerdict { cwe: Option<String> }`, or `GroundingSweep`. | Data schema | G5:L93-L96 |
| DG001-G5-E020 | E | `score_review_surface(file_changes, test_rows, gate_rows, sweep_rows) -> ReviewSurfaceScore` joins SSB evidence by hunk and divides unvouched-line sum by all changed-line sum. | API/algorithm | G5:L93-L99 |
| DG001-G5-E021 | E | A hunk without a vouch is conservatively `needs_attention=true`; no production `.unwrap()`/`.expect()`. | Fallback/reliability | G5:L97-L99 |
| DG001-G5-E022 | E | A surfaced finding requires VibeGate/SecurityRule CWE evidence or `stdout_hash` with `passed=false`; the cockpit makes no LLM review-comment call. | Signal workflow/non-goal | G5:L101-L104 |
| DG001-G5-E023 | E | `elai review --cockpit [--session <id>] [--json]` reads session SSB, scores it, and prints vouched/unvouched bands plus DSR. | CLI/capability | G5:L106-L109 |
| DG001-G5-E024 | E | The authored CLI optionally reads the RRA evidence manifest after G-2 ships. | Optional integration | G5:L106-L109 |
| DG001-G5-E025 | E | `SsbActionType::ReviewSurfaceScored` is one Pattern-2 event with `session_id`, `dsr`, `total_changed_lines`, `attention_lines`, `vouched_hunks`, and `unvouched_hunks`. | Event schema | G5:L111-L115 |
| DG001-G5-E026 | E | Authored event lifecycle is `review_surface_scored` string/parse/CHECK/round-trip/lock with schema 205→206 and variants 482→483. | Authored schema evolution | G5:L111-L115 |
| DG001-G5-E027 | E | `ELAI_REVIEW_COCKPIT_DISABLE` is the only kill-switch; default is ON and live evidence uses no override. | Flag/default | G5:L117-L119 |
| DG001-G5-E028 | E | DSR plus escaped-defect rate is the GD-4 warning; record DSR across at least three real runs, treating failure to shrink as an over-block/ignored-band signal. | Measurement/rollout | G5:L117-L119 |
| DG001-G5-E029 | E | `CockpitWindow.svelte`/extended `ReviewWindow`, `get_review_surface`, and window type `cockpit` render per-hunk badges, vouched/unvouched bands, and headline DSR; cross-language diff must PASS. | UI/surface | G5:L121-L123, L156-L172 |
| DG001-G5-E030 | E | `G-5-decisions.md` records the missing denominator, deterministic-only signal rule, DSR/escaped-defect warning, study-as-hypothesis, and evidence composition. | Decision evidence | G5:L125-L129 |
| DG001-G5-E031 | E | Produced scoring identifiers are `score_review_surface`, `HunkVouch`, `ReviewSurfaceScore`, and corrected `build_file_signals`. | Produced API | G5:L131-L142 |
| DG001-G5-E032 | E | Produced event identifiers are `SsbActionType::ReviewSurfaceScored` and `ReviewSurfaceScoredPayload`. | Produced schema | G5:L136-L137 |
| DG001-G5-C001 | C | Dated correction says the shipped CLI function is `review_cmd.rs::run_review`, dispatched from `main.rs:489`, not a generic `run`. | Corrected identifier | G5:L138-L140 |
| DG001-G5-E033 | E | Consumed evidence identifiers include TestOutcome, VibeGate/SecurityRule, GroundingSweep, ChangeDecision, FileWrite, `file_changes`, and existing review UI/CLI parser surfaces. | Dependencies | G5:L144-L152 |
| DG001-G5-C002 | C | Dated DD5 correction removes G-2 as a load-bearing shipped dependency: G-5 scores from live SSB alone via `build_file_signals` over `session_files_by_seq`; `.rra` reading is optional future enrichment. | Dependency correction | G5:L153-L155 |
| DG001-G5-E034 | E | Corrected production flow is CLI dispatch → `run_review` → `open_score_and_emit` → `build_file_signals + score_review_surface` → `ssb.write_payload(ReviewSurfaceScored)` plus printed DSR. | End-to-end workflow | G5:L156-L161 |
| DG001-G5-E035 | E | Stub bans prohibit model-generated comments, fabricated DSR/unpersisted denominator, claims of pre-G5 recomputability, default-off ENABLE, treating absent `rra.rs` as current, and BLAKE2/SHA2. | Constraints/non-goals | G5:L162-L169 |
| DG001-G5-E036 | E | Observable run orchestrates mixed vouched/unvouched fixture changes, then invokes `elai review --cockpit --session <id>` without override. | Acceptance trigger | G5:L173-L175 |
| DG001-G5-E037 | E | Expected row contains DSR, total/attention lines, and hunk counts; expected artifact is badge/band/headline rendering in the cockpit. | Acceptance evidence | G5:L176-L179 |
| DG001-G5-C003 | C | Dated correction states `--run-trigger` is a deprecated no-op and the validator trigger runs unconditionally, so the deterministic command omits the option. | Corrected tooling | G5:L181-L184 |
| DG001-G5-E038 | E | Deterministic checks grep scoring/wiring/event/golden, require denominator field/UI/decision doc, run validator/cross-language diff, and ban an ENABLE flag. | Verification | G5:L181-L194 |
| DG001-G5-C004 | C | Acceptance still literally requires `wiring_contract_check.py --run-trigger PASS`, conflicting with the dated correction that the flag is deprecated/no-op and omitted. | Stale done-gate conflict | G5:L181-L184, L196-L198 |
| DG001-G5-E039 | E | Acceptance requires default-on CLI/dashboard entrypoints, dashboard bands/DSR, byte-identical DSR recompute, silent behavior for unsupported hunks, honest study framing, event round-trip, cargo/auditor green, and no new dependency. | Done gate | G5:L196-L211 |
| DG001-G5-C005 | C | Dated shipped schema correction replaces stale 205→206/482→483 with baseline 211→212 and variants 488→489; final event ordinal is 0346. | Schema correction | G5:L208-L209, L213-L228 |
| DG001-G5-E040 | E | Corrected W-3 anchors are `Review` variant, `ReviewAction::Cockpit`, parser/dispatch, `main.rs:489`, and dashboard CockpitWindow. | Corrected integration | G5:L213-L224 |
| DG001-G5-E041 | E | Corrected W-7 explicitly says fresh `elai.map` indexed G-5 files and that RRA is not a shipped consume. | Evidence/status | G5:L222-L224 |
| DG001-G5-E042 | E | G-5 is entirely Tier 1: read journal, join evidence per hunk, compute DSR; no network/subprocess/RunPod at gate time. | Tiering | G5:L227-L233 |
| DG001-G5-E043 | E | GD-1 composition is F-4 stdout hash, F-1 verdict/CWE, and C-1 sweep; G-2 is only optional future enrichment. | Architecture/dependency | G5:L230-L233 |
| DG001-G5-E044 | E | DSR contract cases cover hand-computed mixed-hunk ratio, journal recomputation, no-vouch attention, event round-trip, and persisted denominator. | Tests | G5:L235-L239 |
| DG001-G5-E045 | E | Silent-default contract asserts no finding without deterministic evidence, and positive findings for a VibeGate CWE or failing hashed test. | Tests | G5:L240-L241 |
| DG001-G5-E046 | E | Planned creates are scoring/CLI/UI/contracts/fixtures/decision doc; planned modifications are orchestrator lib/SSB/locks, CLI parser/main, and dashboard registration. | Change surfaces | G5:L244-L257 |
| DG001-G5-E047 | E | The human study uses within-subjects, screen recording, clustered standard errors, cockpit-versus-plain-diff, time-to-first-approve, and escaped-defect guardrail; speed-and-safety is a hypothesis, not a result. | Evaluation/claims | G5:L259-L273 |
| DG001-G5-A001 | A | The plan says changed-line counts are sourced from “the FileWrite diff,” while explicitly stating `FileWritePayload` stores no diff/count; it does not define the production diff source or persist per-hunk `lines_changed` in the event schema. | Data provenance ambiguity | G5:L40-L47, L93-L99, L111-L115, L275-L281 |
| DG001-G5-A002 | A | Acceptance says recomputation uses “FileWrite line count,” but no FileWrite schema modification is planned; storing aggregate total/attention fields in the result row permits ratio recomputation but not independent reconstruction of hunk assignment from prior rows as described. | Determinism ambiguity | G5:L196-L204, L244-L257 |
| DG001-G5-A003 | A | Cross-row hunk identity may not align among FileWrite diff, TestOutcome, and gate evidence; `hunk_id=None` falls back to file level, but exact join/fallback semantics remain to be confirmed. | Join ambiguity | G5:L275-L281 |
| DG001-G5-C006 | C | Header/objective/pre-req/reads still call G-2 a hard forward dependency, while later dated correction and done-gate explicitly say SSB-only scoring shipped and RRA is optional. | Internal dependency conflict | G5:L3, L22-L24, L78-L89, L153-L155, L222-L233 |
| DG001-G5-C007 | C | Overall status remains `PLANNED`, yet dated comments repeatedly call functions, event ordinals, schema 212/489, and SSB-only path “shipped”; the file never declares overall DONE. | Status conflict | G5:L3, L138-L155, L208-L233 |
| DG001-G5-E048 | E | The human study must not block feature DONE; cockpit plus journal-auditable DSR is the implementation gate. | Deferred/non-goal | G5:L275-L285 |
| DG001-G5-E049 | E | Downstream, G-7 supplies confidence triage, optional G-2 enriches off-machine vouching, G-8 can sign DSR, and G-5 is the reviewer-facing trust spine. | Interactions | G5:L287-L291 |

### `G-7_confidence_producer_dual_llm.md` — confidence production and lethal-trifecta containment

| Atomic ID | Class | Explicit feature / behavior | Area | Anchor |
|---|---|---|---|---|
| DG001-G7-E001 | E | Header status is `PLANNED`, Wave 3, dependent on G-1 live taint and intended to activate the shipped D-16 conformal consumer. | Status/dependency | G7:L1-L3 |
| DG001-G7-E002 | E | Existing `decide`, `tau`, and `TauUpdater` consume a precomposed `claim_score`; no live per-change producer exists at the authored baseline. | Existing capability/gap | G7:L5-L12 |
| DG001-G7-E003 | E | G-7 fuses intrinsic generation signals, self-consistency dispersion, and F-4-derived test execution agreement into the scalar passed to `decide()`. | Core capability/data flow | G7:L9-L13 |
| DG001-G7-E004 | E | Low confidence triggers the real conformal abstention path, holding/surfacing the change as an honest stop. | Workflow/state | G7:L12-L13 |
| DG001-G7-E005 | E | A context with Trusted secret, Untrusted content, and destructive/egress capability is the lethal trifecta. | Security predicate | G7:L13-L15 |
| DG001-G7-E006 | E | Authored design routes only the Untrusted span through a type-constrained quarantined sub-LLM with no tool grants. | Security/capability isolation | G7:L13-L18 |
| DG001-G7-E007 | E | Platt rescaling is a post-hoc, model-size-agnostic heuristic; it is not a proof of calibrated guarantees for a 7B target. | Calibration/claim boundary | G7:L16-L18, L135-L139 |
| DG001-G7-E008 | E | Abstention boundary is in-process/fail-closed; authored quarantine call is local inference with no RunPod at gate time; estimated ten-to-fourteen hours. | Tiering/constraint | G7:L17-L18 |
| DG001-G7-E009 | E | Required shipped dependencies are D-16 conformal surfaces, G-1 provenance/capability surfaces, provider `GGSignals`, F-4 `stdout_hash`, and the FIDES interface document. | Dependencies | G7:L20-L24 |
| DG001-G7-E010 | E | Parallel-safe with G-2/G-4/G-8, not with G-1 or concurrent edits to conformal calibration/SSB write path. | Concurrency | G7:L25-L26 |
| DG001-G7-E011 | E | `decide(claim_score,tau)` fail-closes NaN to Abstain, clamps score to `[0,1]`, computes nonconformity `1-p`, and Abstains when nonconformity exceeds tau. | Deterministic algorithm/failure | G7:L31-L35 |
| DG001-G7-E012 | E | `tau` is the `(1-epsilon)` empirical quantile with `DEFAULT_EPSILON=0.1`; `TauUpdater` uses atomic temp+rename persistence and minimum corpus size 500. | Calibration/state/lifecycle | G7:L34-L36 |
| DG001-G7-E013 | E | Existing `GGSignals` fields are `mean_logprob`, `min_logprob`, `entropy_mean`, `entropy_max`, and `generation_novelty`, all generation-time/zero-extra-inference intrinsic data. | Input schema | G7:L42-L45 |
| DG001-G7-E014 | E | `generation_novelty` is used as a dispersion proxy; F-4 `TestOutcomePayload.stdout_hash` supplies oracle-free generated-test agreement. | Signals/integration | G7:L42-L49 |
| DG001-G7-E015 | E | FIDES `query_llm` and `HIDE` are paper-only with zero code matches; G-7 builds the primitive and must not cite an existing code symbol. | Status/dependency | G7:L52-L56, L85-L86 |
| DG001-G7-E016 | E | Authored SSB baseline is 205/482; a dated correction states shipped end-state 216/493 after exactly two G-7 variants. | Schema status | G7:L66-L70 |
| DG001-G7-E017 | E | UniCR must be cited as tentative arXiv 2509.01455, not wrong 2503.01345, and its risk-control claim remains moderate/unproven. | Evidence correction/claims | G7:L70-L73 |
| DG001-G7-E018 | E | `compute_claim_score(gg, self_consistency, exec_agreement, platt) -> f64` outputs `[0,1]`. | API | G7:L100-L109 |
| DG001-G7-E019 | E | Intrinsic component normalizes mean/min log probability and entropy. | Algorithm/signal | G7:L102-L105 |
| DG001-G7-E020 | E | Consistency component combines generation novelty and a k-sample agreement ratio. | Algorithm/signal | G7:L104-L106 |
| DG001-G7-E021 | E | Execution component is the fraction of generated tests whose stdout hashes agree across runs. | Algorithm/signal | G7:L105-L106 |
| DG001-G7-E022 | E | `PlattParams { a, b }` and `platt_rescale(raw,p)` implement post-hoc scaling. | API/schema | G7:L106-L108 |
| DG001-G7-E023 | E | Missing signals lower confidence and never default to `1.0`; hot paths contain no `.unwrap()`/`.expect()`. | Failure/fallback/reliability | G7:L109-L110 |
| DG001-G7-E024 | E | At live per-change evaluation, score feeds unchanged `decide`; Abstain holds the change and emits confidence evidence. | Runtime workflow | G7:L112-L116 |
| DG001-G7-E025 | E | `ELAI_CLAIM_CONFIDENCE_DISABLE` is the sole kill-switch; default is ON, trigger uses no override, and an ENABLE flag is forbidden. | Flag/default | G7:L112-L116, L202-L210 |
| DG001-G7-E026 | E | `detect_trifecta(ctx:&TaintedContext,pending_tool:&str)` authored predicate is Trusted-secret AND any Untrusted span AND pending tool in `DESTRUCTIVE_TOOLS`. | API/security | G7:L118-L121 |
| DG001-G7-E027 | E | `query_quarantined(span,expected_type)` returns a type-constrained/relabelled `QuarantinedResult` from a no-tool sub-LLM; its result cannot carry executable tool invocation. | API/security | G7:L121-L124 |
| DG001-G7-E028 | E | `ClaimConfidenceScoredPayload` fields are `claim_score`, `logprob_component`, `self_consistency_component`, `exec_agreement_component`, `tau`, `conformal_verdict`, and `abstained`. | Event schema | G7:L126-L133 |
| DG001-G7-E029 | E | `TrifectaQuarantinedPayload` authored fields are BLAKE3 `untrusted_origin_blake3`, `sink_tool`, `secret_present`, and `quarantined`. | Event schema/privacy | G7:L126-L133 |
| DG001-G7-E030 | E | Both events are single-variant Pattern-2 types with string/parse/CHECK/round-trip/action lock; dated end-state is schema 216, variants 493, including SQL CHECK rows. | Schema lifecycle | G7:L126-L133 |
| DG001-G7-E031 | E | Decision/event copy labels UniCR moderate/unproven and Platt heuristic; no proven 7B calibration guarantee is allowed. | Compliance/claim boundary | G7:L135-L139 |
| DG001-G7-E032 | E | `ConfidenceDrawer.svelte`, `get_claim_confidence`, `get_trifecta_quarantines`, and drawer key `confidence` show score/components/abstention plus origin/sink/quarantine rows; cross-language diff must PASS. | UI/surface | G7:L141-L144 |
| DG001-G7-E033 | E | `G-7-decisions.md` records three-signal fusion, heuristic calibration, evidence strength, trifecta/no-tool design, and G-1/F-4/G-5 composition. | Evidence/decision log | G7:L146-L151 |
| DG001-G7-E034 | E | Produced confidence identifiers are `compute_claim_score`, `platt_rescale`, `PlattParams`, `logprob_confidence`, `self_consistency_agreement`, and `exec_agreement_ratio`. | Produced API | G7:L153-L162 |
| DG001-G7-E035 | E | Produced quarantine identifiers are `detect_trifecta`, `query_quarantined`, `TrifectaComponents`, `QuarantineDispatcher`, and `emit_trifecta_quarantined`. | Produced API | G7:L163-L168 |
| DG001-G7-E036 | E | Produced integration/event/UI identifiers include `task_runner.rs::emit_claim_confidence_scored`, both action types/payloads, two Tauri commands, and `ConfidenceDrawer.svelte`. | Produced surfaces | G7:L168-L175 |
| DG001-G7-C001 | C | Dated correction says the live producer replaces a fake score in `task_runner.rs::dispatch_conformal_refusal_per_claim_hook`; `passing_stdout_hashes(test_rows)` is the nonconstant F-4 input and unchanged conformal `decide` remains the consumer. | Corrected integration | G7:L176-L183 |
| DG001-G7-C002 | C | Dated DD1 correction says local GG log-prob threading is DEFERRED and cloud uses test plus self-consistency; this narrows the authored claim that all three signals feed every live score. | Signal/status correction | G7:L184-L185 |
| DG001-G7-E037 | E | Corrected quarantine consumes `is_egress_tool`, G-9-backed argv-scope `context_holds_secret`, and G-1 `tainted_context`. | Dependencies/security | G7:L186-L187 |
| DG001-G7-E038 | E | Corrected confidence flow is live per-change hook → score using real test signal → unchanged conformal decision → held change on Abstain → `emit_claim_confidence_scored`. | End-to-end workflow | G7:L188-L192 |
| DG001-G7-C003 | C | Corrected production trifecta flow runs before `git_commit`, detects secret+Untrusted+egress, emits `trifecta_quarantined { quarantined=false, sub_llm_verdict="not_run_blocked" }`, and returns `Err` for a hard block. | Corrected security behavior | G7:L193-L200 |
| DG001-G7-C004 | C | The no-tool sub-LLM primitive is built and tested but not driven in production because no live client reaches the free-function chokepoint; live routing is explicitly DEFERRED. | Deferred correction | G7:L197-L200 |
| DG001-G7-E039 | E | The load-bearing production guarantee is that hard refusal prevents the Untrusted span reaching the real tool even though no sub-LLM ran. | Security invariant/fallback | G7:L197-L200 |
| DG001-G7-E040 | E | Stub bans prohibit fabricated/constant score, no-tool leakage, default-off ENABLE, overclaimed UniCR, BLAKE2/SHA2, paper-only code citation, and wrong UniCR ID. | Constraints/non-goals | G7:L202-L210 |
| DG001-G7-E041 | E | Authored observable fixture combines low-consistency/failing-test change with secret+injected Untrusted+destructive-tool context. | Acceptance trigger | G7:L214-L217 |
| DG001-G7-E042 | E | Authored expected evidence is low-score `abstained=true` and `trifecta_quarantined` with `quarantined=true`; artifact requires held change, no tool reach, clean FP control, and drawer rendering. | Authored acceptance | G7:L218-L222 |
| DG001-G7-C005 | C | The authored expectation `quarantined=true` directly conflicts with corrected production evidence `quarantined=false`/`not_run_blocked`. | Acceptance conflict | G7:L193-L200, L214-L222 |
| DG001-G7-C006 | C | Corrected deterministic process removes nonexistent `--run-trigger` opt-in and places the combined Tier-1 contract under `crates/orchestrator/tests`, not `crates/contracts/tests`. | Tooling/test correction | G7:L224-L243 |
| DG001-G7-E043 | E | Deterministic checks grep all score/quarantine APIs, live task/sink wires, both event cascades/goldens, UI/test/decision doc, forbid ENABLE/wrong UniCR/BLAKE2/SHA2, and run cross-language diff. | Verification | G7:L224-L243 |
| DG001-G7-C007 | C | Acceptance and W-1 text still require `--run-trigger PASS` and “the untrusted span was quarantined,” despite corrected tooling and production block-only semantics. | Stale done-gate conflict | G7:L245-L248, L264-L286 |
| DG001-G7-E044 | E | Remaining acceptance requires default-on abstention, real production entrypoints, dashboard E2E, seeded byte-identical score recomputation, honest evidence labels, bidirectional FP guards, event round trips, schema 216/493, cargo/auditor green, and no new dependency. | Done gate | G7:L245-L262 |
| DG001-G7-E045 | E | FP guards require high-confidence clean changes not to abstain and Trusted-only/missing-leg flows not to trigger quarantine. | Tests/invariant | G7:L258 |
| DG001-G7-E046 | E | W-9 final golden ordinals are 0069 for `claim_confidence_scored` and 0464 for `trifecta_quarantined`. | Observability/schema evidence | G7:L278-L279 |
| DG001-G7-E047 | E | Tier 1 is in-process signal fusion/decision plus hard trifecta block; missing signals lower confidence and trifecta returns `Err`. | Failure/tiering | G7:L280-L285 |
| DG001-G7-E048 | E | A real agent run is separately Tier 2/RunPod-gated; production live sub-LLM routing remains a documented follow-up. | Deferred/tiering | G7:L280-L286 |
| DG001-G7-E049 | E | Combined contract covers low score→Abstain/event, high score→Answer, missing-signal degradation, seeded recompute, monotone Platt, score sensitivity to F-4, captured no-tool quarantine, missing-leg FP guard, and both event round trips. | Tests | G7:L291-L300 |
| DG001-G7-E050 | E | Corrected change surfaces create score/quarantine/SSB-adapter/UI/test/decision files and modify orchestrator lib, task runner, production RPC, SSB/locks, and dashboard registration. | Change surfaces | G7:L302-L318 |
| DG001-G7-C008 | C | Event schema declares no `sub_llm_verdict`, but corrected production emission explicitly names that field; the source does not state a corresponding schema correction. | Schema conflict | G7:L126-L133, L193-L200 |
| DG001-G7-A001 | A | Corrected live confidence wire explicitly names real test hashes but does not pin how local deferred log-probs or k-sample self-consistency values are populated at that call; exact per-lane feature vector remains unresolved. | Data-flow ambiguity | G7:L176-L185, L340-L348 |
| DG001-G7-A002 | A | Payload records component values and tau but not `PlattParams`, input-presence mask, model/lane, or seed; the asserted byte-identical score recomputation algorithm/evidence sufficiency is therefore not fully specified. | Determinism ambiguity | G7:L126-L133, L245-L255 |
| DG001-G7-A003 | A | The source alternates between “destructive tool,” “destructive/egress,” and corrected `is_egress_tool`; exact sink universe and whether non-egress destructive tools satisfy the third leg are not reconciled. | Predicate ambiguity | G7:L13-L15, L52-L55, L118-L124, L186-L200 |
| DG001-G7-C009 | C | The corrected `context_holds_secret` explicitly bridges to G-9, adding a dependency absent from the header/pre-requisite/parallel-safety declarations. | Dependency conflict | G7:L3, L20-L26, L186-L200 |
| DG001-G7-A004 | A | Cold-start Platt behavior is only “identity-ish” with low-confidence-by-default; calibration-data selection, persistence, and transition out of cold start are not specified. | Lifecycle ambiguity | G7:L340-L346 |
| DG001-G7-A005 | A | k-sample self-consistency cost is unresolved; the decision doc must choose between k sampling and generation novelty alone. | Config/cost ambiguity | G7:L347-L348 |
| DG001-G7-E051 | E | Before trusting calibration, measure both over- and under-abstention behavior against D-17’s 5%–50% honest-refusal SLA; ELAI’s calibration error has no baseline. | Measurement/release risk | G7:L340-L344 |
| DG001-G7-E052 | E | UniCR remains unverified single-author evidence; only general conformal-risk-control grounding may support stronger theory claims. | Evidence/claim boundary | G7:L349-L351 |
| DG001-G7-C010 | C | Overall status remains `PLANNED`, while dated corrections call the producer, schemas, tests, and end-state “SHIPPED”; no explicit overall DONE transition is present. | Status conflict | G7:L3, L66-L70, L88-L89, L126-L133, L291-L318 |
| DG001-G7-E053 | E | Downstream, G-5 reads low-confidence first; G-2 records both events; G-1 supplies taint; F-3 plus containment forms control/data-plane defense; the producer also enables D-17 refusal measurement. | Interactions | G7:L353-L358 |

## Explicit cross-source interactions and workflows

| Interaction ID | Explicit sequence | Anchors |
|---|---|---|
| DG001-XINT-001 | Shared governance: every G item inherits W-1..W-10 and GD-1..GD-5 → real entrypoint/effect/SSB/UI evidence → FP bound → auditor/wiring/dashboard/memory gates → only then completion. | G0:L45-L93 |
| DG001-XINT-002 | Taint: real Web/MCP/file/sandbox ingest → provenance register/screened join → E-37 can seed Untrusted → production sink uses shipped capability gate → reject/confirm → `taint_flow` → drawer/RRA. | G1:L85-L129, L148-L188, L301-L305 |
| DG001-XINT-003 | Blast radius: E-30 approves/rejects raw verb → G-3 obtains expanded target/host → canonical/path or resolved-IP membership check → fail-closed `HookFinding` → `blast_radius_blocked` with both command forms → drawer/RRA/cockpit. | G3:L106-L150, L170-L192, L301-L305 |
| DG001-XINT-004 | MCP: first discovery TOFU-pins → corrected stdio reconnect re-fetches tool list → hash/diff comparison → changed tool blocked and event emitted → human reviews diff → only explicit approval calls `update_pin`; evidence feeds RRA/cockpit and output taint. | G4:L65-L97, L117-L147, L246-L250 |
| DG001-XINT-005 | Review: live SSB rows → `build_file_signals`/per-hunk join → vouch/attention bands and DSR → `review_surface_scored` + CLI output → CockpitWindow; later G-7 confidence, optional RRA, and G-8 signing enrich the result. | G5:L91-L129, L153-L179, L287-L291 |
| DG001-XINT-006 | Confidence: live F-4/available lane signals → `compute_claim_score` → unchanged conformal `decide` → Answer or held Abstain → `claim_confidence_scored` → drawer/G-5 triage/RRA. | G7:L100-L116, L176-L192, L353-L358 |
| DG001-XINT-007 | Trifecta production: G-9 secret bridge + G-1 Untrusted context + egress sink → pre-dispatch detection → hard `Err` refusal → `trifecta_quarantined` with `quarantined=false`; test-only/built `query_quarantined` routing remains deferred. | G7:L118-L124, L186-L200, L280-L300 |
| DG001-XINT-008 | Sequencing: Wave 1 G-4+G-1 → G-1 unlocks G-3/G-7 → Wave 2 begins with G-2 and includes G-3/G-5/G-8/G-9 → Wave 3 G-7/G-6; G-6 remains gated by live F-2b. | G0:L143-L156 |

## Inferred interactions (not explicit requirements)

| Inference ID | Inference | Basis / caution |
|---|---|---|
| DG001-INF-001 | The dated correction blocks look like post-implementation backfills, so G-1/G-4/G-5/G-7 may be partially or fully landed despite `PLANNED` headers. | High-confidence textual inference from “shipped” identifiers/baselines, but no source changes overall status to DONE; do not promote it. |
| DG001-INF-002 | G-1’s process-global task/turn side-table needs concurrency isolation, cleanup, and task-ID collision policy. | Implied by `TaintedContext` and process-global accessor; lifecycle is not specified. |
| DG001-INF-003 | G-3 needs TOCTOU-resistant binding between canonical validation and the actual filesystem/connect operation. | Canonicalize-then-execute creates a potential race, but the source never defines descriptor-based execution or retry semantics. |
| DG001-INF-004 | G-4 either needs to persist `ToolDescDiff` elsewhere or expand its SSB payload for the dashboard’s exact before/after view. | The declared event payload lacks descriptions; no hidden storage may be assumed. |
| DG001-INF-005 | G-5’s aggregate result row can reproduce the DSR division but cannot alone re-derive which lines/hunks were vouched. | The declared payload has aggregate counts, while per-hunk line provenance remains underspecified. |
| DG001-INF-006 | G-7’s newly explicit G-9 secret-detector bridge is sequence-compatible because G-9 is Wave 2 and G-7 Wave 3, but it is still a missing declared prerequisite. | Cross-source schedule supports the ordering; implementation availability is not independently verified. |

## Ambiguity and conflict register

This register is exhaustive by reference: every source-local `A` or `C` atom appears below exactly once.

| Source | Ambiguities | Conflicts/corrections |
|---|---|---|
| G-0 | none isolated; its rules are explicit governance | none internal in this chunk ledger |
| G-1 | `DG001-G1-A001..A002` | `DG001-G1-C001..C004` |
| G-3 | `DG001-G3-A001..A002` | `DG001-G3-C001` |
| G-4 | `DG001-G4-A001..A003` | `DG001-G4-C001..C005` |
| G-5 | `DG001-G5-A001..A003` | `DG001-G5-C001..C007` |
| G-7 | `DG001-G7-A001..A005` | `DG001-G7-C001..C010` |

## Summary-pass additions and corrections

The orientation pass already captured the initiative-level intent. The following is the exhaustive grouped delta register; referenced atomic records hold the exact identifiers, fields, defaults, tests, and anchors.

| Delta ID | Type | Addition/correction to `07-G-SUMMARY_INDEX.discovery.md` | Deep atomic records |
|---|---|---|---|
| DG001-DELTA-001 | Addition | G0 supplies the complete W-1..W-10 mechanics, seven-part global DONE gate, status semantics, portfolio effort/waves, exact action-type catalog/Kinds, dependency defaults, authoring constraints, verifier landmines, and ship-session pipeline. | `DG001-G0-E003..E057` |
| DG001-DELTA-002 | Addition | G-1 defines the lattice/API surface, exact screened-join signature/state/seed, file trust policy, one-event-per-ingest behavior, full `TaintFlowPayload`, kill-switch name, exact FP fixtures, decision doc, wiring chain, stub bans, and tests/change surfaces. | `DG001-G1-E006..E051`, `DG001-G1-A001..A002` |
| DG001-DELTA-003 | Correction | Summary wording that threads through `agent_pool` is stale: corrected production ingress is split over Web/MCP/sandbox/production RPC, with `translate_tool_request -> enforce_taint_sink_gate`; stale agent-pool text remains elsewhere in the original. | `DG001-G1-C001..C003`, `DG001-G1-E036..E038` |
| DG001-DELTA-004 | Addition | G-3 pins existing allowed-host defaults/network behavior, exact APIs/verdict/event fields, schema 205→206/482→483, kill-switch, error defaults, UI, done gates, fixtures, and non-existent-symbol bans. | `DG001-G3-E008..E045`, `DG001-G3-A001..A002` |
| DG001-DELTA-005 | Correction | The summary’s Landlock-use statement is refuted by the original: Landlock is absent and claiming its use is banned; the OS-agnostic floor is `std::fs`/host checking, with existing Linux veth/iptables only. | `DG001-G3-C001`, `DG001-G3-E016`, `DG001-G3-E037`, `DG001-G3-E046..E047` |
| DG001-DELTA-006 | Addition | G-4 adds exact reverify/verdict/diff schemas, operator-only pin transition, payload fields, kill-switch, UI/decision/evidence gates, legitimate-update FP handling, and tests/change surfaces. | `DG001-G4-E006..E039`, `DG001-G4-A001..A003` |
| DG001-DELTA-007 | Correction | There is no production `notifications/tools/list_changed`/HTTP push handler; shipped/corrected trigger is stdio reconnect-time `discover_tools_for_server`. Authored tasks/acceptance/files still conflict. | `DG001-G4-C001..C003` |
| DG001-DELTA-008 | Correction | The summary says the event contains the before/after description diff, but the original’s declared payload contains only hashes, tool names, and a count; diff persistence/read path is unresolved. | `DG001-G4-A001` |
| DG001-DELTA-009 | Addition | G-5 adds exact existing-row gaps, `HunkVouch`/`VouchKind`/scorer schemas, CLI options, aggregate event fields, kill-switch, UI window, exact signal rule, tests, study protocol, and denominator/join ambiguities. | `DG001-G5-E007..E049`, `DG001-G5-A001..A003` |
| DG001-DELTA-010 | Correction | G-2/RRA is not a hard shipped G-5 dependency: corrected path scores live SSB alone; RRA is optional future enrichment. | `DG001-G5-C002`, `DG001-G5-C006`, `DG001-G5-E033..E034`, `DG001-G5-E041..E043` |
| DG001-DELTA-011 | Correction | G-5’s real CLI function/anchors are `run_review` and `main.rs:489`; `--run-trigger` is deprecated/no-op; shipped schema is 212/489 (ordinal 0346), not authored 206/483. | `DG001-G5-C001`, `DG001-G5-C003..C005`, `DG001-G5-E040` |
| DG001-DELTA-012 | Addition | G-7 adds exact conformal decision/tau semantics, GG signal fields, scoring/Platt APIs, missing-signal fallback, both payload schemas, kill-switch/UI, shipped symbols, tests, cold-start/SLA/k-sample risks, and schema ordinals. | `DG001-G7-E011..E053`, `DG001-G7-A001..A005` |
| DG001-DELTA-013 | Correction | Production confidence replaces a fake task-runner score with real F-4 hashes; local log-prob threading is deferred and cloud uses test+self-consistency rather than an always-three-signal live path. | `DG001-G7-C001..C002` |
| DG001-DELTA-014 | Correction | Production lethal-trifecta handling is hard block-only and emits `quarantined=false`; no sub-LLM runs. The built no-tool dispatcher is deferred, contradicting summary/authored `quarantined=true` routing. | `DG001-G7-C003..C005`, `DG001-G7-C007`, `DG001-G7-E037..E039`, `DG001-G7-E047..E049` |
| DG001-DELTA-015 | Correction | G-7’s corrected path adds an undeclared G-9 secret-detector dependency and names a `sub_llm_verdict` field absent from its declared event payload. | `DG001-G7-C008..C009` |
| DG001-DELTA-016 | Correction | G-1/G-4/G-5/G-7 retain `PLANNED` headers while dated blocks describe shipped end-states; summary status must remain conflicted rather than flattened to planned or done. | `DG001-G1-C004`, `DG001-G4-C005`, `DG001-G5-C007`, `DG001-G7-C010` |

## Zero-omission / zero-duplicate audit

| Source | Lines read | Explicit (`E`) | Conflict/correction (`C`) | Ambiguous (`A`) | Atomic total |
|---|---:|---:|---:|---:|---:|
| `G-0_INDEX.md` | 203 | 57 | 0 | 0 | 57 |
| `G-1_taint_propagation_dataplane.md` | 308 | 52 | 4 | 2 | 58 |
| `G-3_blast_radius_gates.md` | 307 | 48 | 1 | 2 | 51 |
| `G-4_mcp_rugpull_repin.md` | 252 | 39 | 5 | 3 | 47 |
| `G-5_reviewer_cockpit_dsr.md` | 293 | 49 | 7 | 3 | 59 |
| `G-7_confidence_producer_dual_llm.md` | 360 | 53 | 10 | 5 | 68 |
| **Total** | **1,723** | **298** | **27** | **15** | **340** |

- Omission audit: each file was read from line 1 through its terminal line in order. Every status/objective/context/existing-state/task/wiring-contract/observable-effect/deterministic-check/acceptance/done-gate/test/file/citation-risk/next block was checked; repeated statements were folded into the first atomic record and later contradictions were retained as `C`, never discarded.
- Duplicate audit: **340 IDs / 340 unique IDs / 0 duplicate IDs**; explicit + conflict + ambiguity arithmetic is **298 + 27 + 15 = 340**. Cross-source interactions, inferences, and deltas use separate namespaces and are not double-counted as source atoms.
- Original-file audit: **6 files / 1,723 logical lines / 151,046 bytes**, exactly matching the manifest; SHA mismatches: **0**; line-count mismatches: **0**; byte-count mismatches: **0**; out-of-chunk originals read: **0**.
- Output audit: `474` logical lines, `84769` bytes after finalization; normalized output SHA-256: `c607695a15943f403d88e2a55299d3d9cfb81ab1f3e347fb4124187e3995c915` (computed with this digest token replaced by literal `__NORMALIZED_OUTPUT_SHA256__`, avoiding an impossible self-referential hash).
