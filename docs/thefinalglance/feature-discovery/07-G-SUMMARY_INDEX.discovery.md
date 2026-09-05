# ELAI G-plan feature discovery — summary-index pass

## Coverage and extraction contract

- Primary source read in full: `docs/thefinalglance/G-plan/G-SUMMARY_INDEX.md`, 225/225 lines (100%).
- Orientation source: `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md`; the G synopsis at lines 212-224 was used only to confirm the layer-level purpose, not to add detail absent from the G summary.
- Scope found: G-1 through G-9 are the nine original roadmap sessions; G-10 is a separately added post-G-9 extension. The title's “All 9 Sessions” therefore excludes G-10 rather than implying that G-10 is absent.
- Extraction rule: every item in the atomic inventories below is **explicit** (`E`) in the summary. Interpretive joins are isolated under **Inferred interactions** (`I`). Unresolved choices and missing specificity are marked **OPEN**; no behavior from an original initiative file is silently promoted into this summary-only pass.
- Anchor notation: `GSI:Lx-Ly` means line(s) in `G-SUMMARY_INDEX.md`; `OV:Lx-Ly` means line(s) in `ELAI-SPEC-OVERVIEW.md`.
- Status language is preserved: “SHIPS/SHIPPED,” “partly EXISTS,” “WIRE-DARK,” “STUB,” “DEFERRED,” “PLANNED,” “implementation pending,” and “unresearched” describe distinct states and must not be collapsed into “implemented.”

## Initiative/status ledger

| ID and exact name | Roadmap / lever | Grounding | Effort / wave | Summary-level state and hard dependency | Anchor |
|---|---|---|---|---|---|
| **G-1 — Harness-wide taint propagation (data-plane defense, lit up from the shipped substrate)** | R1 / `cant-wreck` | STRONG | L / Wave 1 | Label and gate substrate ships, but production constructors/tags are dead; live wiring is proposed. | GSI:L18-L36 |
| **G-2 — Reproducible-Run Artifact (RRA) [THE KEYSTONE]** | R2 / `auditable-reproducible` | STRONG | M / Wave 2 | New artifact/CLI/viewer work; consumes F-1/F-4/F-9/F-10; anchors G-5/G-6/G-8/G-9. | GSI:L38-L57 |
| **G-3 — Deterministic runtime blast-radius gates (post-expansion FS gate + egress IP-pin)** | R3 / `cant-wreck` | STRONG | M / Wave 2 | Network substrate partly exists; host-side enforcement, pinning, validation, and real dispatch wiring remain. | GSI:L59-L77 |
| **G-4 — Live MCP rug-pull / line-jump re-pin gate** | R4 / `cant-wreck` | STRONG | M / Wave 1 | Extends shipped TOFU schema pin; described as no-dependency, cheapest high-grounding work. | GSI:L79-L92 |
| **G-5 — Reviewer Cockpit + Diff-Surface-to-Review (DSR) metric** | R5 / `verified-before-seen` | STRONG | M / Wave 2 | Planned cockpit/metric; hard dependency G-2. | GSI:L94-L109 |
| **G-6 — SAFEDELTA competitive safety-delta benchmark** | R6 / `auditable-reproducible` | STRONG | XL / Wave 3 | Hard dependencies G-2 and F-2b LIVE; external claims explicitly deferred until the dynamic floor exists. | GSI:L111-L127 |
| **G-7 — Per-change confidence PRODUCER + Dual-LLM lethal-trifecta quarantine** | R7 / `honest-about-limits + cant-wreck` | STRONG | L / Wave 3 | Conformal consumer ships; producer is a deferred slot; G-7 depends on G-1. | GSI:L129-L143 |
| **G-8 — Signed in-toto provenance attestation + Article-50 verifier CLI** | R8 / `auditable-reproducible` | STRONG | L / Wave 2 | Planned attestation/verifier; hard dependency G-2. | GSI:L145-L159 |
| **G-9 — SSB/log write-time secret+PII redaction gate + DB-file-granularity TTL** | R9 / `auditable-reproducible + honest-about-limits` | MODERATE | M / Wave 2 | Planned local redaction/retention; hard dependency G-2 and stated precondition for safely exporting RRA material. | GSI:L161-L175 |
| **G-10 — Model-Drift Regression Canary (exogenous model-version behavioral-regression gate)** | Post-G-9 extension / HYBRID / GENUINELY_NEW | STRONG | EASY-M feasibility; M / Wave 3 | Explicitly PLANNED and implementation pending; depends G-2, reuses D-7. Auto-pin/rollback is outside its done-gate. | GSI:L177-L212 |

## Cross-cutting doctrine — exhaustive atomic inventory

All rows in this and the per-initiative inventories are explicit (`E`).

| Atom | Atomic requirement / behavior | Area | Anchor |
|---|---|---|---|
| G-X-001 | Each G initiative must turn a trust READ, EVIDENCE, RUNTIME, or MEASUREMENT initiative into wired reality. | Scope | GSI:L9-L12 |
| G-X-002 | Each resulting capability must be dashboard-visible. | UI/observability | GSI:L9-L12 |
| G-X-003 | Each resulting capability must be reproducible. | Reproducibility | GSI:L9-L12 |
| G-X-004 | GD-1 is COMPOSE-NOT-RESTATE: G consumes F-gate verdicts and does not reimplement them. | Architecture/invariant | GSI:L10-L12, L214-L216 |
| G-X-005 | Restating any F-1 through F-10 behavior is a P0 defect. | Quality gate | GSI:L214-L216 |
| G-X-006 | An initiative may instead satisfy GD-1 by filling a read, runtime, or measurement gap. | Architecture | GSI:L214-L216 |
| G-X-007 | GD-2 requires every emitted numeric value to be recomputable from the SSB journal. | Data/measurement | GSI:L11-L13 |
| G-X-008 | Numeric recomputation is under `ELAI_SEED`. | Determinism/config | GSI:L217-L218 |
| G-X-009 | The explicitly named recomputable metrics include DSR, safety delta, confidence, and false-positive rate. | Measurement | GSI:L217-L218 |
| G-X-010 | The metric itself, not only the gate, must be auditable. | Auditability | GSI:L217-L218 |
| G-X-011 | W-1 uses the stated `observable_effect` as the production wiring done-gate. | Lifecycle/acceptance | GSI:L2-L7 |
| G-X-012 | A session is not DONE until `wiring_contract_check.py --run-trigger` passes. | Test/ops | GSI:L12-L14 |
| G-X-013 | A session is not DONE until its false-positive rate is bounded using at least three real runs. | Test/measurement | GSI:L12-L14 |
| G-X-014 | A gate cannot flip default-on until false-positive/over-block rate is bounded on at least three real runs. | Rollout/config | GSI:L221-L222 |
| G-X-015 | A default-off environment gate is forbidden as a terminal implementation state. | Config/rollout | GSI:L13-L14 |
| G-X-016 | GD-3 requires gate-on versus gate-off ablation framing for every safety/performance number. | Evaluation | GSI:L219-L220 |
| G-X-017 | Safety/performance reporting must not be framed as a SWE-bench rank. | Claims | GSI:L219-L220 |
| G-X-018 | Safety/performance reporting must not promise perfect code. | Claims | GSI:L219-L220 |
| G-X-019 | GD-5 forbids advertising a guarantee on a fail-open path. | Claims/security | GSI:L223 |
| G-X-020 | GD-5 forbids advertising a guarantee on a RunPod-pending path. | Claims/status | GSI:L223 |
| G-X-021 | GD-5 forbids advertising a guarantee on a cloud-opaque path. | Claims/cloud | GSI:L223 |
| G-X-022 | G inherits F-0 wiring doctrine W-1 through W-10 and adds GD-1 through GD-5. | Dependency/governance | GSI:L5-L7 |
| G-X-023 | Canonical rationale and per-initiative detail live in `workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md`. | Source authority | GSI:L5-L7 |

## G-1 — Harness-wide taint propagation

| Atom | Atomic requirement / behavior | Area | Anchor |
|---|---|---|---|
| G-1-001 | Reuse the shipped RTBAS-style `Provenance` label type rather than create a second label system. | Architecture/status | GSI:L20-L22 |
| G-1-002 | Reuse the shipped `Provenanced<T>` wrapper. | Data model/status | GSI:L20-L22 |
| G-1-003 | Reuse the shipped constructors `from_web`, `from_tool_output`, `from_file`, and `from_sandbox`. | API/status | GSI:L20-L22 |
| G-1-004 | Reuse `capability_gate.rs::allow_tool_with_origin`. | Integration/status | GSI:L20-L22 |
| G-1-005 | Reuse the shipped `DESTRUCTIVE_TOOLS` classification. | Security/status | GSI:L20-L22 |
| G-1-006 | Treat the existing constructors as dead wiring because they are instantiated only in tests. | Gap/status | GSI:L20-L22 |
| G-1-007 | Treat the current live gate as untagged until production provenance reaches it. | Gap/status | GSI:L20-L22 |
| G-1-008 | Construct provenance at the real WebFetch ingest site. | Ingest/data | GSI:L23-L24, L31-L32 |
| G-1-009 | Construct provenance at the real MCP tool-call ingest site. | Ingest/data | GSI:L23-L24, L31-L32 |
| G-1-010 | Construct provenance at the real file-read ingest site. | Ingest/data | GSI:L23-L24, L31-L32 |
| G-1-011 | Construct provenance at the real sandbox-stdout ingest site. | Ingest/data | GSI:L23-L24, L31-L32 |
| G-1-012 | Thread `Provenanced<String>` through agent-pool dispatch. | Data flow/integration | GSI:L23-L24, L31-L32 |
| G-1-013 | Ensure `allow_tool_with_origin` receives the live origin/taint. | Security integration | GSI:L23-L24, L31-L32 |
| G-1-014 | Enforce the capability gate before a destructive sink executes. | Security/lifecycle | GSI:L31-L32 |
| G-1-015 | Destructive sinks explicitly include git operations. | Security/scope | GSI:L31-L32 |
| G-1-016 | Destructive sinks explicitly include Write. | Security/scope | GSI:L31-L32 |
| G-1-017 | Destructive sinks explicitly include Bash. | Security/scope | GSI:L31-L32 |
| G-1-018 | Deterministically block an untrusted-content-to-destructive-sink flow. | Security/action | GSI:L23-L25 |
| G-1-019 | A blocked flow may instead be routed to confirmation; the summary does not choose between unconditional block and confirmation routing. | Error/fallback, OPEN | GSI:L34-L36 |
| G-1-020 | Use a dependency-screened taint join. | Security/precision | GSI:L25-L28 |
| G-1-021 | Do not use a global taint join because its cited Slack failure mode over-taints and collapses utility. | Security/invariant | GSI:L27-L28 |
| G-1-022 | Consume F-3 least-privilege/tool-set commitments as the control-plane complement. | Dependency | GSI:L29-L30 |
| G-1-023 | Let E-37 `classify_injection` seed the `Untrusted` label. | Dependency/data flow | GSI:L29-L30 |
| G-1-024 | Show taint-flow rows in a dashboard drawer. | UI/observability | GSI:L31-L33 |
| G-1-025 | The done fixture includes injected web or MCP content that asks for `git push` or `fs.write`. | Test | GSI:L34-L35 |
| G-1-026 | The fixture must yield an `Untrusted`-labelled context. | Test/data | GSI:L34-L35 |
| G-1-027 | The fixture's destructive sink must be blocked or routed to confirmation. | Test/security | GSI:L34-L36 |
| G-1-028 | Emit a `taint_flow` SSB row for the blocked/confirmed flow. | Observability/data | GSI:L34-L36 |
| G-1-029 | The `taint_flow` row must name the ingest origin. | Observability/schema | GSI:L34-L36 |
| G-1-030 | A clean `Trusted` flow must remain unaffected as the false-positive guard. | Test/invariant | GSI:L35-L36 |
| G-1-031 | The finished gate is default-on with no environment override. | Config/rollout | GSI:L35-L36 |

## G-2 — Reproducible-Run Artifact (RRA) [THE KEYSTONE]

| Atom | Atomic requirement / behavior | Area | Anchor |
|---|---|---|---|
| G-2-001 | Emit one portable RRA file. | Artifact | GSI:L40-L43 |
| G-2-002 | Encode the file with msgpack. | Format | GSI:L40-L43 |
| G-2-003 | Compress the file with zstd. | Format | GSI:L40-L43 |
| G-2-004 | Bundle the SSB action stream. | Artifact data | GSI:L40-L42 |
| G-2-005 | Bundle `ELAI_SEED`. | Artifact data/config | GSI:L40-L42 |
| G-2-006 | Bundle `HardwareClass`. | Artifact data | GSI:L40-L42 |
| G-2-007 | Bundle a `system_fingerprint` pin. | Artifact data | GSI:L40-L42 |
| G-2-008 | Bundle a model pin. | Artifact data | GSI:L40-L42 |
| G-2-009 | Bundle an evidence manifest containing every consumed F-gate's BLAKE3 verdict hash. | Artifact/evidence | GSI:L41-L42 |
| G-2-010 | Include F-1 `source_blake3` in that manifest. | Evidence field | GSI:L41-L42 |
| G-2-011 | Include F-4 `stdout_hash` in that manifest. | Evidence field | GSI:L41-L42 |
| G-2-012 | Include F-9 `dep_name_hash` in that manifest. | Evidence field | GSI:L41-L42 |
| G-2-013 | Include F-10 `delta_blake3` in that manifest. | Evidence field | GSI:L41-L42 |
| G-2-014 | Compute a BLAKE3 root over the bundle. | Integrity | GSI:L42-L43 |
| G-2-015 | Add `elai run --emit-rra`. | CLI | GSI:L43-L44 |
| G-2-016 | Add `elai rra show`. | CLI | GSI:L43-L44 |
| G-2-017 | Add `elai replay --verify`. | CLI | GSI:L43-L44 |
| G-2-018 | Scope cloud-run reproducibility to the action stream, not tokens. | Cloud/claims | GSI:L46-L48 |
| G-2-019 | Do not claim a cloud token-reproducibility guarantee when the provider fingerprint is opaque. | Cloud/claims | GSI:L46-L48 |
| G-2-020 | Account for SSB tool-I/O bodies being hashed rather than stored. | Data/privacy constraint | GSI:L47-L48 |
| G-2-021 | For body-reading steps, explicitly state whether the guarantee is divergence verification or full re-execution. | Claims/OPEN | GSI:L47-L48 |
| G-2-022 | Consume F-1/F-4/F-9/F-10 verdict hashes without reimplementing those gates. | Dependency/invariant | GSI:L49 |
| G-2-023 | Serve as an anchor for G-5, G-6, G-8, and G-9. | Dependency | GSI:L49 |
| G-2-024 | Use the append-only SQLite WAL SSB action stream; no UPDATE or DELETE is permitted. | State/lifecycle | GSI:L50-L52 |
| G-2-025 | Integrate existing determinism seed, `HardwareClass`, and `system_fingerprint` substrate. | Integration/status | GSI:L50-L52 |
| G-2-026 | Confirm the canonical module path for determinism/fingerprint components at authoring time under W-7. | Authoring/ops, OPEN | GSI:L50-L52 |
| G-2-027 | Integrate at the production run entrypoint identified by `build_production_hook_dispatcher` triple-bootstrap. | Integration | GSI:L52 |
| G-2-028 | Treat `recover_from_checkpoint` as a stub, not as evidence that BSP lacks a caller. | Status/correction | GSI:L53-L54 |
| G-2-029 | Treat resume-from-journal as the actual missing resume capability. | Gap/lifecycle | GSI:L53-L54 |
| G-2-030 | Keep resume-from-journal as an optional second half rather than part of the core stated artifact gate. | Scope/deferred | GSI:L53-L54 |
| G-2-031 | Provide an RRA viewer panel. | UI | GSI:L54 |
| G-2-032 | The done fixture must cause `elai run --emit-rra` to write a `.rra` file. | Test | GSI:L55-L57 |
| G-2-033 | Verification re-runs the action stream. | Replay/action | GSI:L55-L57 |
| G-2-034 | PASS requires action-stream reproduction. | Verification | GSI:L55-L57 |
| G-2-035 | PASS requires every F-gate verdict hash to be rechecked. | Verification | GSI:L55-L57 |
| G-2-036 | On mismatch, report DIVERGENCE. | Error/reporting | GSI:L55-L57 |
| G-2-037 | A DIVERGENCE report identifies the first mismatching event. | Error diagnostics | GSI:L55-L57 |
| G-2-038 | Emit `run_artifact_emitted` into SSB. | Observability | GSI:L55-L57 |
| G-2-039 | The `run_artifact_emitted` row carries the bundle's BLAKE3 root. | Event schema | GSI:L55-L57 |

## G-3 — Deterministic runtime blast-radius gates

| Atom | Atomic requirement / behavior | Area | Anchor |
|---|---|---|---|
| G-3-001 | Gate filesystem writes after shell expansion. | Security/runtime | GSI:L61-L63 |
| G-3-002 | Gate filesystem deletes after shell expansion. | Security/runtime | GSI:L61-L63 |
| G-3-003 | Canonicalize tilde expansion before the verb runs. | Path handling | GSI:L61-L63 |
| G-3-004 | Canonicalize glob expansion before the verb runs. | Path handling | GSI:L61-L63 |
| G-3-005 | Canonicalize symlink resolution before the verb runs. | Path handling | GSI:L61-L63 |
| G-3-006 | Fail closed when the resolved target is outside the worktree root. | Security/error | GSI:L61-L63 |
| G-3-007 | Enforce a host-side egress allowlist. | Network/security | GSI:L62-L63 |
| G-3-008 | Resolve an allowed hostname to an IP. | Network | GSI:L62-L63 |
| G-3-009 | Pin the resolved allowed IP. | Network/security | GSI:L62-L63 |
| G-3-010 | Reject a hostname containing a null byte. | Input validation/security | GSI:L62-L64 |
| G-3-011 | Reject a hostname containing control characters. | Input validation/security | GSI:L62-L64 |
| G-3-012 | Emit a block event for either blast-radius gate. | Observability | GSI:L63-L65 |
| G-3-013 | Record the complete pre-expansion command in the block event. | Forensics/schema | GSI:L63-L65 |
| G-3-014 | Record the complete post-expansion command in the block event. | Forensics/schema | GSI:L63-L65 |
| G-3-015 | Use Landlock on Linux kernel 6.7+ as an OS backstop. | OS security | GSI:L65-L68 |
| G-3-016 | Compose with E-30: E-30 gates the destructive verb; G-3 gates the expanded path/host. | Dependency/boundary | GSI:L69-L70 |
| G-3-017 | Consume G-1 live taint tags as richer scoring input for filesystem/egress gates. | Dependency | GSI:L69-L70 |
| G-3-018 | Reuse the partly existing `allowed_hosts` substrate in `sandbox/researcher_net.rs`. | Status/integration | GSI:L70-L73 |
| G-3-019 | Reuse the partly existing veth+iptables OUTPUT allowlist. | Status/integration | GSI:L70-L73 |
| G-3-020 | Treat `compile_seccomp_network_extra` opening AF_INET wholesale as a gap, not sufficient egress isolation. | Status/security gap | GSI:L71-L73 |
| G-3-021 | Add the missing host-side enforcement, IP pin, and null-byte rejection. | Gap/implementation | GSI:L72-L73 |
| G-3-022 | Wire enforcement into a real dispatch path rather than the deployment-note stub. | Integration/done | GSI:L72-L73 |
| G-3-023 | Test `rm -rf $X` where `$X` expands to `~/`. | Test | GSI:L74-L75 |
| G-3-024 | Test a fetch to a null-byte-spoofed host. | Test | GSI:L74-L75 |
| G-3-025 | Block the filesystem fixture before the destructive verb runs. | Test/security | GSI:L74-L75 |
| G-3-026 | Block the hostname fixture before connect. | Test/security | GSI:L74-L75 |
| G-3-027 | Emit `blast_radius_blocked` for the fixture. | Event/observability | GSI:L74-L75 |
| G-3-028 | The `blast_radius_blocked` row carries both pre- and post-expansion command forms. | Event schema | GSI:L74-L75 |
| G-3-029 | Scope seccomp+Landlock filesystem/worktree protection as Linux-only. | Platform limitation | GSI:L76-L77 |
| G-3-030 | Treat Windows AppContainer parity as unresearched and scope it separately. | Future/OPEN | GSI:L76-L77 |
| G-3-031 | Treat macOS Seatbelt parity as unresearched and scope it separately. | Future/OPEN | GSI:L76-L77 |
| G-3-032 | Treat the egress proxy and credential broker as OS-agnostic. | Platform scope | GSI:L76-L77 |

## G-4 — Live MCP rug-pull / line-jump re-pin gate

| Atom | Atomic requirement / behavior | Area | Anchor |
|---|---|---|---|
| G-4-001 | Reuse shipped `mcp_schema_pin.rs`, including `pin_or_verify`, `hash_tool_list`, and `PinStatus`. | Architecture/status | GSI:L81-L82 |
| G-4-002 | Recognize the current schema pin as TOFU-only, operating at first connect. | Lifecycle/status | GSI:L81-L82 |
| G-4-003 | Handle every `notifications/tools/list_changed` notification. | Integration/event handling | GSI:L82-L83 |
| G-4-004 | Recompute the BLAKE3 schema pin after each tools-list change. | Integrity/action | GSI:L82-L84 |
| G-4-005 | Fail closed when the recomputed pin mismatches. | Security/error | GSI:L82-L84 |
| G-4-006 | Require explicit operator re-approval after a mismatch. | Human gate/lifecycle | GSI:L82-L84 |
| G-4-007 | Keep the changed tool blocked until re-approval. | State/lifecycle | GSI:L91-L92 |
| G-4-008 | Emit a new `mcp_rug_pull_blocked` SSB row on mismatch. | Observability | GSI:L83-L84 |
| G-4-009 | Include the before/after tool-description diff in that row. | Event schema/forensics | GSI:L83-L84 |
| G-4-010 | Treat description changes as security-significant because line-jumping can hide there. | Security invariant | GSI:L83-L86 |
| G-4-011 | Preserve existing `mcp_schema_pin_violation` behavior while extending the shipped pin. | Composition/status | GSI:L81-L82, L88 |
| G-4-012 | Compose with E-24 MCP version pin. | Dependency | GSI:L88 |
| G-4-013 | Compose with E-43 whole-directory skill-import scanning. | Dependency | GSI:L88 |
| G-4-014 | Wire the MCP plugin's `tools/list_changed` handler through recomputation and `PinStatus::Violation` to blocking/event emission. | Workflow/integration | GSI:L89-L90 |
| G-4-015 | Show the before/after description diff in a dashboard drawer. | UI | GSI:L89-L90 |
| G-4-016 | Test with a fixture MCP server that swaps a tool description after first-connect approval. | Test | GSI:L91-L92 |
| G-4-017 | The fixture must emit `mcp_rug_pull_blocked` containing the diff. | Test/observability | GSI:L91-L92 |
| G-4-018 | Verify any `mcp-scan` distribution dependency before adding it; its distribution channel is unverified. | Supply chain/ops | GSI:L85-L87 |
| G-4-019 | Attribute `mcp-scan` to Invariant Labs, not Snyk. | Source correction | GSI:L87 |

## G-5 — Reviewer Cockpit + Diff-Surface-to-Review (DSR)

| Atom | Atomic requirement / behavior | Area | Anchor |
|---|---|---|---|
| G-5-001 | Render review evidence per hunk. | UI | GSI:L96-L99 |
| G-5-002 | Render the F-4 stdout-hash-bound passing contract-test proof token. | Evidence/UI | GSI:L96-L97 |
| G-5-003 | Render grounding-sweep citations. | Evidence/UI | GSI:L96-L97 |
| G-5-004 | Render F-1 write-gate verdicts. | Evidence/UI | GSI:L96-L97 |
| G-5-005 | Render F-2 write-gate verdicts. | Evidence/UI | GSI:L96-L97 |
| G-5-006 | Mark hunks as vouched or un-vouched so reviewers can skim vouched hunks. | UX/workflow | GSI:L96-L99, L104-L106 |
| G-5-007 | Define DSR as lines needing full attention divided by total changed lines. | Metric | GSI:L97-L99 |
| G-5-008 | Compute DSR deterministically from SSB events. | Measurement/data | GSI:L97-L99 |
| G-5-009 | Use DSR as the headline review-time number. | Product metric | GSI:L98-L100 |
| G-5-010 | Be silent by default. | UX/default | GSI:L99-L100 |
| G-5-011 | Surface a finding only when a deterministic gate verdict backs it. | Signal policy | GSI:L99-L100 |
| G-5-012 | Also surface a finding when a failing test backs it. | Signal policy | GSI:L99-L100 |
| G-5-013 | Do not surface LLM-opinion-only review noise. | Signal policy/invariant | GSI:L99-L100 |
| G-5-014 | Consume F-4 `stdout_hash`. | Dependency | GSI:L103 |
| G-5-015 | Consume the G-2 RRA evidence manifest. | Dependency | GSI:L103 |
| G-5-016 | Consume grounding-sweep evidence. | Dependency | GSI:L103 |
| G-5-017 | Add a cockpit view to dashboard-v2 that reads RRA and SSB evidence. | UI/integration | GSI:L104-L105 |
| G-5-018 | Add `elai review --cockpit`. | CLI | GSI:L104 |
| G-5-019 | Emit `review_surface_scored` for the fixture diff. | Observability | GSI:L104-L106 |
| G-5-020 | Make journal recomputation of DSR byte-identical. | Determinism/test | GSI:L105-L106 |
| G-5-021 | Use DSR plus escaped-defect rate as the GD-4 false-positive early warning. | Measurement/safety | GSI:L105-L106 |
| G-5-022 | Measure time-to-first-approve in a within-subjects study. | Evaluation | GSI:L107-L109 |
| G-5-023 | Screen-record the study. | Evaluation/ops | GSI:L107-L108 |
| G-5-024 | Use clustered standard errors. | Evaluation/statistics | GSI:L107-L108 |
| G-5-025 | Compare cockpit review against a plain diff. | Evaluation/control | GSI:L107-L108 |
| G-5-026 | Use escaped-defect rate as the safety guardrail. | Evaluation/guardrail | GSI:L108-L109 |
| G-5-027 | Treat simultaneous speed and safety as a hypothesis to test, not a pre-claimed result. | Claims/invariant | GSI:L108-L109 |

## G-6 — SAFEDELTA competitive safety-delta benchmark

| Atom | Atomic requirement / behavior | Area | Anchor |
|---|---|---|---|
| G-6-001 | Run a gate-on versus gate-off ablation. | Benchmark design | GSI:L113-L114 |
| G-6-002 | Measure the gate's marginal residual-insecure rate that reaches a human. | Metric | GSI:L113-L114 |
| G-6-003 | Use git-history-stripped held-out tasks. | Anti-gaming/dataset | GSI:L114-L119 |
| G-6-004 | Treat head-to-head competitor-harness comparison as secondary to the ablation. | Benchmark priority | GSI:L114-L117 |
| G-6-005 | Include SUSVIBES-200. | Benchmark suite | GSI:L115-L116 |
| G-6-006 | Include BaxBench-392 with dynamic exploit execution. | Benchmark suite | GSI:L115-L116 |
| G-6-007 | Include SeCodePLT through the shipped `seccodeplt_runner`. | Benchmark suite/integration | GSI:L116 |
| G-6-008 | Include AgentDojo. | Benchmark suite | GSI:L116 |
| G-6-009 | Treat MSB as an optional suite. | Benchmark suite/optional | GSI:L116-L117 |
| G-6-010 | Include ELAI gate-on as a benchmark arm. | Benchmark arm | GSI:L117-L118 |
| G-6-011 | Include ELAI gate-off as a benchmark arm. | Benchmark arm | GSI:L117-L118 |
| G-6-012 | Include Cursor with a frozen model as a benchmark arm. | Benchmark arm/config | GSI:L117-L118 |
| G-6-013 | Include Claude Code as a benchmark arm. | Benchmark arm | GSI:L117-L118 |
| G-6-014 | Include Copilot as a benchmark arm. | Benchmark arm | GSI:L117-L118 |
| G-6-015 | Include Cline as a benchmark arm. | Benchmark arm | GSI:L117-L118 |
| G-6-016 | Add a cheat detector for git-history exploitation. | Anti-gaming | GSI:L118-L119 |
| G-6-017 | Execute exploits dynamically rather than rely only on static labels. | Verification | GSI:L119-L120 |
| G-6-018 | Hold out by CWE split. | Dataset/evaluation | GSI:L119-L120 |
| G-6-019 | Preserve the ablation arm as an anti-gaming control. | Evaluation invariant | GSI:L119-L120 |
| G-6-020 | Bind SAFEDELTA's own PASS to an F-4 `stdout_hash`. | Evidence integrity | GSI:L119-L120 |
| G-6-021 | Report the headline as fewer dynamically exploitable diffs per 100 tasks reaching a reviewer. | Reporting/claims | GSI:L120-L121 |
| G-6-022 | Report a 95% bootstrap confidence interval. | Statistics/reporting | GSI:L120-L121 |
| G-6-023 | Report a per-CWE breakdown. | Reporting | GSI:L120-L121 |
| G-6-024 | Never position SAFEDELTA as a SWE-bench rank. | Claims/invariant | GSI:L120-L121 |
| G-6-025 | Require the F-2b LIVE dynamic single-harness extension before G-6. | Dependency | GSI:L122-L123 |
| G-6-026 | Do not redefine F-2 inside G-6. | Architecture/invariant | GSI:L122-L123 |
| G-6-027 | Wrap runs with G-2 reproducibility. | Dependency | GSI:L123 |
| G-6-028 | Use F-4 to bind SAFEDELTA's result. | Dependency | GSI:L123 |
| G-6-029 | Add `elai bench safedelta`. | CLI | GSI:L124-L125 |
| G-6-030 | Run the full benchmark matrix under `ELAI_SEED`. | Determinism/config | GSI:L124-L125 |
| G-6-031 | Emit `safety_delta_measured`. | Observability | GSI:L124-L125 |
| G-6-032 | Emit a signed run manifest. | Artifact/integrity | GSI:L124-L125 |
| G-6-033 | Make that manifest independently replayable. | Artifact/reproducibility | GSI:L124-L125 |
| G-6-034 | Defer every external safety claim until the F-2b dynamic floor lands. | Lifecycle/claims | GSI:L124-L125 |
| G-6-035 | Treat confidence-interval tightness with 200-392 tasks and small per-CWE cells as unresolved. | Statistics/OPEN | GSI:L126-L127 |
| G-6-036 | Consider pooling suites if needed for power, but treat cross-harness methodological defensibility as unverified. | Future/OPEN | GSI:L126-L127 |

## G-7 — Per-change confidence producer + Dual-LLM quarantine

| Atom | Atomic requirement / behavior | Area | Anchor |
|---|---|---|---|
| G-7-001 | Reuse the shipped conformal-refusal consumer in `conformal_calibration.rs`. | Architecture/status | GSI:L131-L134 |
| G-7-002 | Reuse its `decide` interface. | API/status | GSI:L131-L134 |
| G-7-003 | Reuse its scalar `claim_score`. | Data/API | GSI:L131-L134 |
| G-7-004 | Reuse its `tau` threshold. | Calibration/data | GSI:L131-L134 |
| G-7-005 | Reuse `TauUpdater`. | Calibration/API | GSI:L131-L134 |
| G-7-006 | Fill the currently deferred composite `claim_score` producer slot. | Gap/status | GSI:L131-L134 |
| G-7-007 | Use token log probability as one producer signal. | Confidence signal | GSI:L132-L134 |
| G-7-008 | Use self-consistency dispersion as one producer signal. | Confidence signal | GSI:L132-L134 |
| G-7-009 | Use oracle-free generated-test execution agreement as one producer signal. | Confidence signal | GSI:L132-L134 |
| G-7-010 | Fuse those signals into the scalar consumed by `decide()`. | Data flow | GSI:L132-L134 |
| G-7-011 | Make conformal abstention fire on real changes. | Runtime behavior | GSI:L133-L134 |
| G-7-012 | Treat abstention as an honest stop. | UX/claims | GSI:L133-L134 |
| G-7-013 | Detect the lethal trifecta when one context contains a trusted secret, untrusted content, and egress capability. | Security/detection | GSI:L134-L136 |
| G-7-014 | Route only the untrusted span through a quarantined sub-LLM. | Security/data flow | GSI:L134-L136 |
| G-7-015 | Grant the quarantined sub-LLM no tools. | Capability isolation | GSI:L134-L136 |
| G-7-016 | Allow post-hoc Platt rescaling for the calibration target, including a 7B model. | Calibration/config | GSI:L136-L138 |
| G-7-017 | Do not cite the UniCR risk-control guarantee as proven; it remains unverified/moderate preprint evidence. | Evidence/claims | GSI:L139 |
| G-7-018 | Compose with D-16's shipped conformal layer. | Dependency | GSI:L140 |
| G-7-019 | Depend on G-1 live taint to detect the trifecta. | Dependency | GSI:L140 |
| G-7-020 | Consume F-4 execution signal. | Dependency | GSI:L140 |
| G-7-021 | A low-self-consistency fixture must emit `claim_confidence_scored`. | Test/observability | GSI:L141-L142 |
| G-7-022 | The same fixture must produce conformal ABSTENTION. | Test/runtime | GSI:L141-L142 |
| G-7-023 | Hold the abstained change and surface it as low-confidence. | Lifecycle/UX | GSI:L141-L143 |
| G-7-024 | A lethal-trifecta fixture must emit `trifecta_quarantined`. | Test/observability | GSI:L142-L143 |
| G-7-025 | The fixture's untrusted span must never reach the tool caller. | Security invariant/test | GSI:L142-L143 |
| G-7-026 | Feed G-5's confidence-triage band. | Dependency/UI | GSI:L143 |
| G-7-027 | Order the confidence-triage workflow so low-confidence material is read first. | Review workflow | GSI:L143 |

## G-8 — Signed in-toto provenance attestation + Article-50 verifier CLI

| Atom | Atomic requirement / behavior | Area | Anchor |
|---|---|---|---|
| G-8-001 | Emit a custom in-toto Statement v1 predicate. | Artifact/standard | GSI:L147-L150 |
| G-8-002 | Bind its subject by BLAKE3 digest. | Integrity/schema | GSI:L147-L148 |
| G-8-003 | Bind its subject by SHA-256 digest. | Integrity/schema | GSI:L147-L148 |
| G-8-004 | Carry the model ID. | Predicate field | GSI:L147-L149 |
| G-8-005 | Carry `ELAI_SEED`. | Predicate field | GSI:L147-L149 |
| G-8-006 | Carry an SLSA-VSA-style `PASSED`/`verifiedLevels` rollup of every F-gate. | Predicate field/evidence | GSI:L147-L149 |
| G-8-007 | Carry evidence hashes. | Predicate field | GSI:L147-L149 |
| G-8-008 | Carry the SSB event ID. | Predicate field/provenance | GSI:L147-L149 |
| G-8-009 | Put only the attestation digest in a git commit trailer. | Git/integration/privacy | GSI:L148-L150 |
| G-8-010 | Emit the full attestation as a sidecar file. | Artifact | GSI:L148-L150 |
| G-8-011 | Add a new SSB action type for the attestation. | State/schema | GSI:L148-L150 |
| G-8-012 | Sign using a local BLAKE3-Merkle proof by default. | Signing/default | GSI:L149-L150 |
| G-8-013 | Keep the default signing path offline. | Privacy/ops | GSI:L149-L150 |
| G-8-014 | Offer cosign keyless signing only as opt-in. | Signing/config | GSI:L149-L150 |
| G-8-015 | Add `elai verify-attestation`. | CLI | GSI:L150 |
| G-8-016 | Recheck attestations offline. | Verification/privacy | GSI:L150-L151 |
| G-8-017 | Position the feature only as “designed to support Article-50 marking.” | Compliance claims | GSI:L151-L154 |
| G-8-018 | Do not claim “certified compliant.” | Compliance invariant | GSI:L152-L154 |
| G-8-019 | Use EUR 15M/3% as the Article-99 penalty tier, not EUR 35M/7%. | Compliance correction | GSI:L153-L154 |
| G-8-020 | Consume F-4 to extend anti-fabrication off-machine. | Dependency | GSI:L155-L156 |
| G-8-021 | Share G-2's evidence manifest and SSB chaining. | Dependency/data | GSI:L155-L156 |
| G-8-022 | Verify cosign/sigstore dependencies before adding them. | Supply chain/ops | GSI:L156-L157 |
| G-8-023 | Hand-roll the local-Merkle default rather than make cosign/sigstore mandatory. | Dependency/default | GSI:L156-L157 |
| G-8-024 | `elai run` emits an attestation. | Runtime/CLI | GSI:L158-L159 |
| G-8-025 | `elai verify-attestation <commit>` verifies the commit's attestation without re-running ELAI. | CLI/workflow | GSI:L158-L159 |
| G-8-026 | The verifier prints `verified` or `not-verified`. | CLI/output | GSI:L158-L159 |
| G-8-027 | Emit `provenance_attestation_emitted` into SSB. | Observability | GSI:L158-L159 |

## G-9 — SSB/log write-time secret+PII redaction + DB TTL

| Atom | Atomic requirement / behavior | Area | Anchor |
|---|---|---|---|
| G-9-001 | Keep redaction local-only. | Privacy/default | GSI:L163-L164 |
| G-9-002 | Send no redaction payload to a cloud service. | Privacy/security | GSI:L163-L164 |
| G-9-003 | Fail closed when redaction cannot safely complete. | Security/error | GSI:L163-L165 |
| G-9-004 | Apply redaction on the single-writer SSB commit path. | Integration/lifecycle | GSI:L163-L165 |
| G-9-005 | Apply redaction on the hook-log sink. | Integration/lifecycle | GSI:L163-L165 |
| G-9-006 | Detect secrets before persistence. | Privacy/security | GSI:L163-L165 |
| G-9-007 | Detect PII before persistence. | Privacy/security | GSI:L163-L165 |
| G-9-008 | Replace detected values with stable BLAKE3-keyed tokens. | Data transformation | GSI:L164-L165 |
| G-9-009 | Preserve append-only row validity after replacement. | State invariant | GSI:L164-L166 |
| G-9-010 | Never rely on post-write scrubbing because SSB permits no UPDATE/DELETE. | Lifecycle/invariant | GSI:L164-L166 |
| G-9-011 | Implement retention by purging an expired session database as one whole file. | Retention/lifecycle | GSI:L165-L166 |
| G-9-012 | Do not delete individual expired rows. | Retention/invariant | GSI:L165-L166 |
| G-9-013 | Account for the current SSB payload carrying the raw prompt. | Threat model/status | GSI:L167-L168 |
| G-9-014 | Reuse F-1's secret-regex set. | Dependency | GSI:L171-L172 |
| G-9-015 | Do not add Presidio as a dependency. | Dependency/privacy | GSI:L169-L170 |
| G-9-016 | Do not add ggshield as a dependency. | Dependency/privacy | GSI:L169-L170 |
| G-9-017 | Implement the redactor locally/hand-rolled. | Architecture | GSI:L169-L170 |
| G-9-018 | Re-source Presidio claims from Presidio documentation because the cited PRvL numbers were refuted. | Evidence correction | GSI:L169-L170 |
| G-9-019 | Preserve the corrected EU penalty of EUR 15M/3%, not EUR 35M/7%. | Compliance correction | GSI:L169-L170 |
| G-9-020 | Make G-9 a precondition for an RRA that cannot export unredacted secrets. | Dependency/security | GSI:L171-L172 |
| G-9-021 | Treat PII detection as inherently high-recall/low-precision. | Limitation | GSI:L172 |
| G-9-022 | Bound the PII detector's false-positive rate under GD-4. | Measurement/rollout | GSI:L172 |
| G-9-023 | Test an event containing an AWS key. | Test | GSI:L173-L174 |
| G-9-024 | Test an event containing an email address. | Test | GSI:L173-L174 |
| G-9-025 | Verify both are converted to BLAKE3 tokens at SSB commit time. | Test/data | GSI:L173-L174 |
| G-9-026 | Verify raw secret material never reaches `ssb.db`. | Privacy invariant/test | GSI:L173-L174 |
| G-9-027 | Verify raw secret material never reaches the hook log. | Privacy invariant/test | GSI:L173-L174 |
| G-9-028 | Test whole-file purge of an expired session database. | Test/retention | GSI:L173-L175 |
| G-9-029 | Make RRA and attestation artifacts safe enough to ship to a client without creating a secrets-exposure liability. | Product/security outcome | GSI:L174-L175 |

## G-10 — Model-Drift Regression Canary

| Atom | Atomic requirement / behavior | Area | Anchor |
|---|---|---|---|
| G-10-001 | Treat G-10 as a post-G-9 extension, not an R1-R9 trust-max roadmap item. | Scope/status | GSI:L177-L181 |
| G-10-002 | Treat implementation as planned and pending. | Status | GSI:L177-L181 |
| G-10-003 | Compose shipped substrate rather than reopen a trust-max roadmap item. | Architecture | GSI:L178-L181 |
| G-10-004 | Watch exogenous model-version behavioral regression, unlike existing endogenous-keyed canaries. | Capability/novelty | GSI:L183-L186 |
| G-10-005 | Pin the `(model, system_fingerprint, lane)` identity tuple. | State/config | GSI:L187-L188 |
| G-10-006 | Maintain a small fixed canary set. | Test data | GSI:L187-L188 |
| G-10-007 | Key baseline scores to the pinned identity tuple. | State/data | GSI:L187-L188 |
| G-10-008 | Trigger a canary rerun when identity changes. | Lifecycle/trigger | GSI:L188-L190 |
| G-10-009 | Also permit a scheduled canary rerun. | Lifecycle/scheduling | GSI:L188-L190 |
| G-10-010 | Compare against the last-good baseline. | Evaluation/state | GSI:L188-L190 |
| G-10-011 | Compute a regression delta per canary family. | Metric | GSI:L188-L190 |
| G-10-012 | Flag `model_drift_suspected` on regression. | Observability/action | GSI:L189-L190 |
| G-10-013 | Route affected `ReasoningTemplate`s to review. | Workflow | GSI:L189-L190 |
| G-10-014 | Surface suspected drift rather than auto-swap the model by default. | Safety/default | GSI:L190-L191 |
| G-10-015 | Preserve the current ground-truth finding that no model-version pin type exists. | Status/gap | GSI:L195-L197 |
| G-10-016 | Treat security-monotonicity, staged-eval, optimizer-admit, `canary.rs`, and goal-drift canaries as endogenous-keyed only. | Status/boundary | GSI:L195-L197 |
| G-10-017 | Treat `autonomous_gate::substitute_model` as restart/VRAM-triggered, not model-drift monitoring. | Status/boundary | GSI:L196-L197 |
| G-10-018 | Treat SSB `ModelSwap` and `TierModelSwap` as wire-dark because they have no production emitter. | Status | GSI:L197 |
| G-10-019 | Reuse shipped `canary.rs::CanaryFamily`. | Dependency/status | GSI:L198-L199 |
| G-10-020 | Reuse shipped `FamilyEvalResult`. | Dependency/status | GSI:L198-L199 |
| G-10-021 | Reuse the shipped non-determinism band. | Dependency/status | GSI:L198-L199 |
| G-10-022 | Reuse shipped `held_out::TaskId`. | Dependency/status | GSI:L198-L199 |
| G-10-023 | Reuse shipped `reasoning_template::Domain`. | Dependency/status | GSI:L198-L199 |
| G-10-024 | Reuse D-7 families, delta, and band without reimplementation. | Dependency/invariant | GSI:L200-L201 |
| G-10-025 | Use the G-2 `RunArtifact` system fingerprint and model pin as identity inputs. | Dependency/data | GSI:L200-L201 |
| G-10-026 | Consume F-4 `stdout_hash` as the malformed-output-spike signal. | Dependency/signal | GSI:L201 |
| G-10-027 | Use D-1 `ReasoningTemplate` as the review/revert target. | Dependency/workflow | GSI:L201-L202 |
| G-10-028 | On the local lane, label drift fingerprint-confirmed. | Claims/lane | GSI:L202-L203 |
| G-10-029 | On an opaque cloud lane, label drift behavioral-suspected. | Claims/lane | GSI:L202-L203 |
| G-10-030 | Never fabricate cloud fingerprint confirmation. | Claims/invariant | GSI:L202-L203 |
| G-10-031 | Add `crates/orchestrator/src/model_drift.rs`. | Code integration | GSI:L204-L205 |
| G-10-032 | Add a run-end check in `runtime_setup.rs`. | Runtime integration | GSI:L204-L205 |
| G-10-033 | Add `elai model-drift baseline`. | CLI | GSI:L204-L205 |
| G-10-034 | Add `elai model-drift check`. | CLI | GSI:L204-L205 |
| G-10-035 | Add `elai model-drift show`. | CLI | GSI:L204-L205 |
| G-10-036 | Add `model_drift_suspected` as a Pattern-2 SSB event. | Event/schema | GSI:L204-L205 |
| G-10-037 | Add `ModelDriftPanel` as the frontend pair. | UI | GSI:L205-L206 |
| G-10-038 | Keep auto-pin-to-last-good as a separate, wire-dark-allowlisted, human-gated follow-on. | Future/security | GSI:L205-L207 |
| G-10-039 | Exclude auto-pin-to-last-good from the G-10 done-gate. | Scope/invariant | GSI:L205-L207 |
| G-10-040 | If later built, auto-pin-to-last-good would create the first `ModelSwap` emitter. | Future/status | GSI:L205-L207 |
| G-10-041 | Baseline the done fixture's canary set under fixture model A. | Test/workflow | GSI:L208-L210 |
| G-10-042 | Run `model-drift check` under a mock-provider model B with regressed output. | Test/workflow | GSI:L208-L210 |
| G-10-043 | Emit `model_drift_suspected` for the regressed model B. | Test/observability | GSI:L208-L210 |
| G-10-044 | Include signals in that row. | Event field | GSI:L209-L210 |
| G-10-045 | Include `per_family_delta` in that row. | Event field | GSI:L209-L210 |
| G-10-046 | Include `affected_template_ids` in that row. | Event field | GSI:L209-L210 |
| G-10-047 | Include `confirmed_by_fingerprint` in that row. | Event field | GSI:L209-L210 |
| G-10-048 | Cleanly recheck model A within the non-determinism band. | Test/FP guard | GSI:L210-L211 |
| G-10-049 | Return `NoDrift` for the clean recheck. | Test/output | GSI:L210-L211 |
| G-10-050 | Emit no drift row for the clean recheck. | Test/observability | GSI:L210-L211 |
| G-10-051 | Recompute the delta byte-identically from the SSB journal under `ELAI_SEED`. | Determinism/test | GSI:L211-L212 |
| G-10-052 | Show before/after scores in `ModelDriftPanel`. | UI | GSI:L211-L212 |
| G-10-053 | Show a fingerprint-confirmed versus behavioral badge in `ModelDriftPanel`. | UI/claims | GSI:L211-L212 |

## Explicit end-to-end interactions and workflows

These sequences only reorder actions that the summary itself states; they do not add new mechanics.

1. **G-1 taint flow:** Web/MCP/file/sandbox ingress constructs `Provenance` -> `Provenanced<String>` travels through agent-pool dispatch -> E-37 can seed `Untrusted` -> `allow_tool_with_origin` evaluates the live taint before git/Write/Bash -> the destructive action is blocked or confirmation-routed -> `taint_flow` records the origin -> the dashboard drawer exposes it. (GSI:L20-L36)
2. **G-2 artifact/replay:** production run collects the append-only SSB stream, determinism/model/hardware pins, and consumed F-verdict hashes -> msgpack+zstd bundle receives a BLAKE3 root -> `.rra` and `run_artifact_emitted` are produced -> viewer/`rra show` inspect it -> `replay --verify` reproduces the action stream and rechecks verdict hashes -> PASS or first-event DIVERGENCE is reported. (GSI:L38-L57)
3. **G-3 filesystem gate:** shell input is captured -> tilde/glob/symlink expansion is canonicalized -> resolved target is checked against worktree -> operation fails closed before the verb if outside -> both command forms land in `blast_radius_blocked`. (GSI:L59-L75)
4. **G-3 egress gate:** hostname is checked for null/control bytes -> allowed host is resolved -> its IP is pinned host-side -> connect is blocked before dispatch on violation -> `blast_radius_blocked` records the forensic command context. (GSI:L61-L75)
5. **G-4 MCP re-pin:** first connection is TOFU-approved -> `notifications/tools/list_changed` arrives -> tool-list hash is recomputed -> mismatch becomes `PinStatus::Violation` -> tool is blocked fail-closed -> description diff is journaled/displayed -> only explicit operator re-approval unblocks it. (GSI:L79-L92)
6. **G-5 review:** dashboard/CLI reads RRA plus SSB proof tokens -> diff is divided into vouched/un-vouched hunk bands -> only deterministic verdicts/failing tests surface -> DSR is computed and journaled -> reviewer skims vouched material and focuses on un-vouched/low-confidence material -> controlled study measures time-to-first-approve with escaped defects as guardrail. (GSI:L94-L109, L143)
7. **G-6 SAFEDELTA:** under a fixed seed, git-stripped held-out tasks execute across gate-on/gate-off and secondary competitor arms -> dynamic exploit checks and cheat detector guard evaluation -> F-4 binds the result -> `safety_delta_measured` plus signed replayable manifest are emitted -> headline, bootstrap CI, and CWE breakdown are derived. (GSI:L111-L127)
8. **G-7 confidence:** three confidence signals are fused -> existing conformal `decide()` consumes the scalar -> low confidence produces a journaled abstention/held change -> G-5 reads that band first. (GSI:L129-L143)
9. **G-7 trifecta:** G-1 detects trusted-secret + untrusted + egress in one context -> only the untrusted span enters a tool-less quarantined sub-LLM -> it never reaches the tool caller -> `trifecta_quarantined` proves the isolation. (GSI:L134-L143)
10. **G-8 attestation:** `elai run` builds the in-toto predicate from run identity/evidence -> signs locally by default -> writes digest-only commit trailer, sidecar, and SSB event -> offline verifier resolves a commit and prints verified/not-verified without rerunning ELAI. (GSI:L145-L159)
11. **G-9 persistence:** raw event/hook payload reaches the write boundary -> local fail-closed detector replaces secrets/PII with stable keyed tokens -> only redacted rows/logs persist -> retention removes the entire expired session DB. (GSI:L161-L175)
12. **G-10 drift:** establish model-A identity-keyed baseline -> identity change or schedule triggers canary -> model-B results are compared per family to last-good -> regression emits detailed `model_drift_suspected`, routes templates to review, and surfaces a lane-honest badge -> clean A result within the non-determinism band produces `NoDrift` and no event. (GSI:L183-L212)

## Invariants, error/fallback behavior, and claim boundaries

| ID | Explicit invariant / boundary | Anchor |
|---|---|---|
| INV-G-01 | Security and trust evidence is composed from F-gate verdicts; duplicate F implementations are forbidden. | GSI:L11-L12, L214-L216 |
| INV-G-02 | Missing/unsafe runtime evidence fails closed where specified: blast radius, MCP re-pin, and redaction. | GSI:L61-L64, L82-L84, L163-L165 |
| INV-G-03 | Append-only SSB rows are not updated or individually deleted; redaction happens before write and TTL is file-granular. | GSI:L50-L50, L164-L166 |
| INV-G-04 | Every emitted number is journal-recomputable and seeded. | GSI:L217-L218 |
| INV-G-05 | Default-on rollout requires empirical false-positive/over-block bounds from at least three real runs. | GSI:L12-L14, L221-L222 |
| INV-G-06 | Dependency-screened taint join is mandatory; global join is forbidden. | GSI:L25-L28 |
| INV-G-07 | Cloud RRA claims stop at action-stream reproduction where tokens/fingerprints are opaque. | GSI:L46-L48 |
| INV-G-08 | G-3 blocks outside-worktree effects before verb/connect and preserves both pre/post-expansion forensic forms. | GSI:L61-L75 |
| INV-G-09 | MCP schema mismatch blocks until explicit human re-approval. | GSI:L82-L92 |
| INV-G-10 | Reviewer findings require deterministic evidence or a failing test; model-only opinion is silent. | GSI:L96-L100 |
| INV-G-11 | SAFEDELTA reports marginal safety under ablation, never a SWE-bench rank. | GSI:L113-L125 |
| INV-G-12 | Quarantined untrusted text receives no tools and cannot reach the tool caller. | GSI:L134-L143 |
| INV-G-13 | Attestation verifies offline without rerunning ELAI and is not marketed as certification. | GSI:L150-L159 |
| INV-G-14 | Raw detected secrets must never reach either SSB DB or hook log. | GSI:L163-L175 |
| INV-G-15 | G-10 surfaces drift and routes review; it does not auto-swap by default. | GSI:L187-L207 |
| INV-G-16 | Local drift may be fingerprint-confirmed; opaque-cloud drift is only behavioral-suspected. | GSI:L200-L203 |

## Dependency/composition map

| Consumer | Explicit input / prerequisite | How it composes | Anchor |
|---|---|---|---|
| G-1 | F-3 | Adds data-plane taint to F-3's control-plane tool-set commitment. | GSI:L29-L30 |
| G-1 | E-37 | `classify_injection` seeds `Untrusted`. | GSI:L29-L30 |
| G-2 | F-1/F-4/F-9/F-10 | Packages their verdict hashes without reimplementation. | GSI:L41-L49 |
| G-3 | E-30 | E-30 gates verb; G-3 gates expanded path/host. | GSI:L69-L70 |
| G-3 | G-1 | Uses live taint as richer scoring context. | GSI:L69-L70 |
| G-4 | E-24/E-43 | Extends MCP version pin and composes with skill import scanning. | GSI:L88-L90 |
| G-5 | G-2/F-4/grounding sweep | Reads evidence manifest, stdout hash, and citations. | GSI:L95-L106 |
| G-6 | F-2b LIVE | Dynamic security floor must land first; G-6 does not redefine it. | GSI:L112-L125 |
| G-6 | G-2/F-4 | Reproducibility wrapper and result binding. | GSI:L122-L125 |
| G-7 | D-16/G-1/F-4 | Existing conformal consumer, taint detector, and execution signal. | GSI:L130-L143 |
| G-5 | G-7 | Reads low-confidence changes first. | GSI:L143 |
| G-8 | G-2/F-4 | Shares evidence manifest/SSB chain and extends anti-fabrication off-machine. | GSI:L145-L159 |
| G-9 | G-2/F-1 | Protects exported RRA; reuses F-1 secret regexes. | GSI:L161-L175 |
| G-10 | G-2/D-7/F-4/D-1 | Identity pin; canary family/delta/band; malformed-output signal; review target. | GSI:L183-L212 |

## Inferred interactions (not independently specified behavior)

| Inference | Basis | Confidence / caution |
|---|---|---|
| I-G-01 | G-2 is called the keystone and explicitly anchors G-5/G-6/G-8/G-9. | High: an implementation ordering G-2 -> those consumers is directly implied, but exact delivery phases are not specified beyond waves. |
| I-G-02 | G-9 depends on G-2 while also being called G-2's export-safety precondition. | High: building the RRA can precede redaction, but distributing an RRA/attestation containing content should wait for G-9. Exact release gating needs original-file confirmation. |
| I-G-03 | G-1 feeds both G-3 and G-7; G-7 feeds G-5. | High: this yields a taint -> blast-radius/quarantine -> reviewer-triage chain. The exact event joins and IDs are not in the summary. |
| I-G-04 | G-8 can verify evidence off-machine because G-2 supplies a portable manifest and SSB chain. | High, but the summary does not specify sidecar lookup, trailer parsing, or key storage details. |
| I-G-05 | G-10's future auto-pin would be a separate human-gated remediation workflow after detection. | High: explicitly excluded from the done-gate; no automatic rollback algorithm may be assumed. |

## Ambiguities and details requiring original-file inspection

| Initiative | Summary ambiguity / detail not fixed here |
|---|---|
| G-0 | Exact W-1..W-10 and GD-1..GD-5 checker mechanics, schemas, authoring rules, evidence format, and failure codes. |
| G-1 | Exact provenance enum/lattice, Trusted-secret representation, dependency-screened join algorithm, dispatch ownership, which flows block versus confirm, `taint_flow` schema, and definition/sample method for false positives. |
| G-2 | RRA envelope/versioning, zstd settings, ordering/canonical serialization, root coverage, manifest schema, event replay semantics, tool-body availability, system-fingerprint abstraction, CLI exit codes, divergence format, viewer behavior, and optional journal-resume boundary. |
| G-3 | Shell-expansion mechanism and quoting semantics, race/TOCTOU protection, worktree-root derivation, symlink/canonicalization failure handling, DNS rebinding lifetime, IP/port/protocol policy, credential-broker role, event schema, and non-Linux roadmap. |
| G-4 | Tool-list canonicalization, before/after diff format, approval storage/expiry, notification race handling, schema-vs-description hash coverage, multi-server identity, event fields, and UI action semantics. |
| G-5 | Exact hunk-vouching rule, line-count treatment, renamed/binary/generated files, DSR event fold, proof-token rendering, navigation/actions, evidence-staleness behavior, study sample size/randomization, and escaped-defect definition. |
| G-6 | Matrix orchestration, frozen-model identity, exact task split/deduplication, competitor configuration fairness, exploit sandboxing, cheat-detector behavior, manifest signature, bootstrap unit, power threshold, suite pooling, and external-claim gate. |
| G-7 | Signal normalization/weights, Platt training/evaluation data, tau update lifecycle, missing-signal behavior, event fields, exact lethal-trifecta predicate, span extraction, quarantined-model selection, return-data sanitization, and review hold/release action. |
| G-8 | Predicate type URI/schema/version, digest canonicalization, key generation/storage/rotation, Merkle proof format, cosign/Rekor behavior, sidecar naming/lookup, trailer format, offline trust roots, CLI exit codes, SSB action type, and Article-50 wording placement. |
| G-9 | Secret/PII detector patterns, key source/rotation for stable tokens, collision/domain separation, fail-closed failure mode, redaction of nested/binary data, debug access, session TTL configuration, purge scheduling/locking, crash consistency, and false-positive study. |
| G-10 | Identity canonicalization, baseline persistence/versioning, canary-set composition, schedule configuration, regression threshold/aggregation, last-good promotion, Pattern-2 schema, rerun cost controls, template review/revert workflow, lane badge derivation, and handling of missing/changed fingerprints. |

## Deeper-source manifest

### Canonical G-plan sources (all discovered; only the summary was mined in this pass)

| Source | Why the next/deep agent must inspect it |
|---|---|
| `docs/thefinalglance/G-plan/G-0_INDEX.md` | Enforcement doctrine, inherited W rules, GD rules, source map, and likely cross-initiative acceptance contracts. |
| `docs/thefinalglance/G-plan/G-1_taint_propagation_dataplane.md` | Exact taint data model, joins, live ingest wiring, gate decision semantics, event/UI schema, tests. |
| `docs/thefinalglance/G-plan/G-2_reproducible_run_artifact.md` | RRA format, canonicalization, replay algorithm, limitations, CLI/viewer contracts, resume scope. |
| `docs/thefinalglance/G-plan/G-3_blast_radius_gates.md` | FS and egress enforcement mechanics, platform boundaries, expansion/DNS edge cases, event contract. |
| `docs/thefinalglance/G-plan/G-4_mcp_rugpull_repin.md` | MCP re-pin lifecycle, hashing/diff/approval behavior, plugin integration, event/UI details. |
| `docs/thefinalglance/G-plan/G-5_reviewer_cockpit_dsr.md` | DSR fold and denominator, proof/hunk rules, cockpit UX, measurement protocol. |
| `docs/thefinalglance/G-plan/G-6_safedelta_benchmark.md` | Benchmark matrix, dataset/competitor controls, dynamic verifier, signing/replay, statistics/claim gates. |
| `docs/thefinalglance/G-plan/G-7_confidence_producer_dual_llm.md` | Confidence fusion/calibration, abstention lifecycle, quarantine architecture, security boundaries. |
| `docs/thefinalglance/G-plan/G-8_intoto_attestation_article50.md` | Predicate/signature/trailer/sidecar formats, offline trust, verifier CLI, compliance copy. |
| `docs/thefinalglance/G-plan/G-9_journal_redaction_ttl.md` | Write-boundary redaction algorithm, keyed-token/key lifecycle, TTL/purge mechanics, tests/privacy. |
| `docs/thefinalglance/G-plan/G-10_model_drift_regression_canary.md` | Baseline identity/state, canary evaluation and thresholds, events/panel/CLI, follow-on boundary. |
| `docs/thefinalglance/G-plan/G-SUMMARY_INDEX.md` | This pass's complete primary source and anchor authority. |

### Directly referenced rationale and governance sources

| Source | Reference reason |
|---|---|
| `workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md` | Named by GSI:L5-L7 as canonical “why” and per-initiative detail. |
| F-plan `F-0` doctrine (exact path must be resolved by the deep agent) | G-0 inherits F-0 W-1..W-10; precise inherited requirements are not restated here. |
| `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md` | Layer-level orientation only; G synopsis is OV:L212-L224. |

### Summary-named implementation surfaces to verify against originals/code

- `provenance.rs`: `Provenance`, `Provenanced<T>`, four constructors (GSI:L20-L22).
- `capability_gate.rs`: `allow_tool_with_origin`, `DESTRUCTIVE_TOOLS` (GSI:L20-L22, L31-L32).
- Agent-pool dispatch ingest sites: WebFetch, MCP tool call, file read, sandbox stdout (GSI:L31-L32).
- `ssb.rs`: append-only/WAL/no UPDATE/DELETE, raw prompt payload (GSI:L50-L50, L163-L168).
- `seccodeplt_runner.rs`, `l3_lock.rs`, and `build_production_hook_dispatcher` run bootstrap (GSI:L50-L53).
- `recover_from_checkpoint` and resume-from-journal gap (GSI:L53-L54).
- `sandbox/researcher_net.rs`, `allowed_hosts`, veth+iptables OUTPUT allowlist, `sandbox/mod.rs:1269`, `compile_seccomp_network_extra` (GSI:L70-L73).
- `mcp_schema_pin.rs` and `plugins/src/mcp/*` notification handler (GSI:L81-L90).
- dashboard-v2 and its taint, RRA, MCP-diff, cockpit, and ModelDrift frontend pairs (GSI:L31-L33, L54, L89-L90, L104-L106, L204-L212).
- `conformal_calibration.rs`: `decide`, `claim_score`, `tau`, `TauUpdater` (GSI:L131-L134).
- `canary.rs`, `held_out::TaskId`, `reasoning_template::Domain`, `autonomous_gate::substitute_model`, `ModelSwap`, `TierModelSwap`, `runtime_setup.rs` (GSI:L195-L205).

### External/reference claims that originals should pin precisely

- G-1: RTBAS arXiv:2502.08966; FIDES arXiv:2505.23643; CaMeL arXiv:2503.18813; especially dependency-screened versus global-join results (GSI:L25-L28).
- G-2: ESAA arXiv:2602.23193; ActiveGraph arXiv:2605.21997; claude-replay comparison; Anthropic fingerprint opacity (GSI:L44-L48).
- G-3: rm-rf issues #10077/#12637; SOCKS5 null-byte bypass; arXiv:2604.04978; Landlock kernel support (GSI:L64-L68).
- G-4: OWASP MCP03:2025; Trail of Bits line-jumping; Invariant Labs WhatsApp/mcp-scan attribution and distribution uncertainty (GSI:L85-L87).
- G-5: METR arXiv:2507.09089; review-agent signal arXiv:2604.03196; naive-review weakness arXiv:2509.13650 (GSI:L100-L109).
- G-6: SUSVIBES arXiv:2512.03262; BaxBench arXiv:2502.11844; SeCodePLT arXiv:2410.11096; AgentDojo arXiv:2406.13352; optional MSB arXiv:2510.15994; Endor git-history finding (GSI:L115-L127).
- G-7: FIDES/HIDE; Dual-LLM arXiv:2506.08837; Spiess arXiv:2402.02047; CodeHalu arXiv:2405.00253; Task-Abstention arXiv:2605.17029; unverified UniCR 2509.01455 (GSI:L134-L139).
- G-8: in-toto Statement v1; SLSA VSA; Sigstore Rekor v2; EU AI Act Articles 50 and 99; June 2026 Code of Practice status (GSI:L147-L157).
- G-9: OWASP AI Agent Cheat Sheet; GitGuardian MCP/Claude-Code figures; ggshield behavior; corrected Presidio sourcing; EU penalty correction (GSI:L167-L172).
- G-10: arXiv:2311.11123; RETAIN arXiv:2409.03928; AgentAssay arXiv:2603.02601, used as motivation rather than an ELAI target (GSI:L191-L194).

## Quantitative summary

- Source coverage: **225/225 primary-summary lines (100%)**.
- Initiatives covered: **10/10 present in the file** (G-1..G-9 plus planned G-10).
- Original canonical G-plan files manifested: **11 initiative/governance originals plus the summary**.
- Explicit atomic inventory: **343 atoms** (`G-X`: 23; G-1: 31; G-2: 39; G-3: 32; G-4: 19; G-5: 27; G-6: 36; G-7: 27; G-8: 27; G-9: 29; G-10: 53).
- Explicit end-to-end workflows: **12**.
- Explicit invariants/claim boundaries: **16**.
- Explicit dependency/composition edges: **13 rows**.
- Separated inferred interactions: **5**.
- Initiative-level ambiguity sets for deeper inspection: **11** (G-0 plus G-1..G-10).
