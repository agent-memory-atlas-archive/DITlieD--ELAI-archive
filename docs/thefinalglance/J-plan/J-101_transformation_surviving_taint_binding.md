# J-101: transformation-surviving taint binding + cross-process taint + E-37 perimeter completion — the OWASP indirect-injection hardening, ONE E-37/G-1 EXTENSION (WI-1 keystone + WI-2 + WI-3)

STATUS: PLANNED (WAVE 1; the technical injection-hardening half the K-13 OUT-OF-K block recorded NOT authored; an EXTENSION on E-37/G-1, NOT an in-place amend — new-plan-files-only; the dominant indirect-injection path the wired per-argv substring gate misses; SSB prefer-reuse TaintFlow rows, net 0..+1)

- Track J | lever CANT-LAUNDER-TRUST (paraphrased untrusted content cannot reach a destructive sink by being transformed through the LLM) | grounding STRONG (provenance.rs / capability_gate.rs / production_agent_rpc.rs / taint_flow.rs codegraph file:line-grounded 2026-06-25 in the OWASP analysis Section D; re-grep at edit W-7) | nature BUILD (strengthen a wired gate + add a window heuristic + cross-process propagation + complete the E-37 perimeter) | effort L (3 legs, WI-1 keystone) | WAVE 1
- This is the technical companion to K-13 (the OWASP control-mapping dossier + injection-posture PROVE/PACKAGE). K-13 maps + proves the EXISTING spine; J-101 HARDENS it. OWASP threats: LLM01 Prompt Injection / LLM05 Improper Output Handling / ASI01 Agent Goal Hijack / ASI06 Memory & Context Poisoning / ASI07 Insecure Inter-Agent Comms. The three legs (WI-1/WI-2/WI-3) are ONE security-hardening subsystem (the taint binding + cross-process coverage + the injection perimeter), so they ship as ONE extension file with three vertical-slice legs (per the OWASP analysis Section E "likely shape").
- Source: workflow/reasoning/owasp-llm-agentic-gap-analysis-2026-06-25.md Section D (the live-code taint trace, the GENUINE residual gap), Section E WI-1/WI-2/WI-3 (the deliverables), Section F (the do-not-re-propose ledger — full CaMeL interpreter DEFERRED), Section G (the residue to verify first). K-13 is the K-native PROVE+PACKAGE half. Doctrine: J-0_INDEX.md.

## Context
Why this matters (the GENUINE residual gap, Section D, the code says it itself at production_agent_rpc.rs ~234 GD-5 honest-scope): ELAI's G-1/G-7 taint defense is REAL, default-ON, fail-closed, four ingest legs wired (web/MCP/file-screened/sandbox), contract-tested — the architecture is correct (structural taint + capability denial, NOT prose system-prompt guards, which OWASP + ELAI agree fail). But the per-argv binding is a VERBATIM SUBSTRING MATCH (ctx.lookup_for_argv(part), >= MIN_TAINT_MATCH_LEN bytes, same-process). It does NOT catch:
  (1) TRANSFORMED/SUMMARIZED/PARAPHRASED untrusted content — the LLM reads a poisoned doc, paraphrases the malicious instruction, and the paraphrase flows to a destructive sink. The LLM is a transformation between ingest and sink; substring identity cannot survive it. THIS IS THE DOMINANT indirect-injection path (LLM01/ASI01) and the per-argv gate misses it.
  (2) CROSS-PROCESS sinks (same-process registry; ELAI's agents are OS processes). NOTE taint_flow.rs:472 process_global_slot_is_shared suggests a partial shared slot exists — VERIFY exact cross-process coverage before scoping (Section G).
  (3) non-McpToolRegistry MCP transports.
The Provenanced<T> TYPE exists (the strong CaMeL binding the doc-comment aspires to) but is NOT threaded end-to-end through the LLM-output->argv path; the wired mechanism is the substring registry MVP. The G-7 lethal-trifecta SESSION gate (enforce_trifecta_quarantine_gate, production_agent_rpc.rs:334) is the existing backstop for exactly case (1) — lean on it, do not re-invent.

WI-1 [KEYSTONE] TRANSFORMATION-SURVIVING TAINT BINDING — hybrid, NOT the full CaMeL interpreter (research-grade, HD/IC tension, DEFERRED per Section F):
  (a) PRIMARY = strengthen the already-wired G-7 lethal-trifecta SESSION gate (production_agent_rpc.rs:334) as the load-bearing backstop: if untrusted content entered context AND a destructive sink is reached AND an egress/exfil capability is present => HITL/quarantine, REGARDLESS of per-value match.
  (b) SECONDARY = a taint-WINDOW heuristic: mark argv produced by the model AFTER it consumed untrusted context in the same turn as carrying DERIVED-UNTRUSTED provenance (span/turn window), so transformation does not launder trust.
  (c) keep the substring gate as the cheap exact-match FAST PATH.
ASI06 NOTE (Section G, decide at authoring): memory provenance = TrustLevel::Recallable does NOT block destructive sinks today (capability_gate.rs:96; HITL at a higher layer). Decide whether the WI-1 window logic should treat Recallable-after-untrusted-write as Untrusted (a recalled poison could drive a sink). // DECISION at authoring, default = treat Recallable-written-after-an-untrusted-ingest-in-the-same-session as derived-untrusted under the window (fail-closed, conservative).

WI-2 CROSS-PROCESS TAINT PROPAGATION (ASI07/ASI01): tainted_context() is same-process; Tier-C agents are OS processes. Propagate Provenance across the Tier-C postcard IPC boundary so cross-process destructive sinks see upstream taint — but VERIFY the existing process_global_slot (taint_flow.rs:472) FIRST and scope to the CONFIRMED gap only (do NOT rebuild a working shared slot; Section G).

WI-3 E-37 PERIMETER COMPLETION (LLM01/LLM05) — ACTIVATION of planned E-37: build the recorded-ABSENT pieces output_sanitizer.rs + output_classifier.rs + egress allowlist + an AgentDojo CI job (feature-map 1.8 PROMPT INJECTION DEFENSE [PARTIAL]). Composes G-1 (data-plane) + G-3 (egress).

THE NUMBER (WI-4, OWNED BY K-13, not this file): the gate-ON-vs-OFF injection-defense delta from G-6 SAFEDELTA's AgentDojo arm is K-13's dossier row, [BENCH-PENDING] on F-2b RunPod. J-101 ships the MEASURABLE done-gate FIXTURE + the contract test (a paraphrased injection BLOCKED by the window/trifecta gate, ten-benign FP guard green); the full SAFEDELTA number FILLS IN via K-13/A-2/A-3. Do NOT fabricate an injection number here (GD-3: gate-ON-vs-OFF ablation, never a leaderboard rank).

## Existing state (what the legs harden / cite; do NOT re-author, GD-1)
- TAINT PROPAGATION (G-1) [WIRED] Provenance{source,trust} + Provenanced<T> (crates/safety/src/provenance.rs); Trusted user_prompt/system, Untrusted from_web/from_tool_output/from_file_screened/from_sandbox; from_file_screened + path_is_trusted_source FAIL-CLOSED (empty project_root => nothing trusted; any `..` => disqualified). 4 ingest legs wired in production.
- CAPABILITY GATE [WIRED] crates/safety/src/capability_gate.rs allow_tool_with_origin rejects any DESTRUCTIVE_TOOLS call whose argv carries TrustLevel::Untrusted; structured Reject{reason,citation,offending_arg_index}; gate-error => fail-CLOSED. Recallable does NOT block (capability_gate.rs:96, by design — the ASI06 decision point).
- TWO PRODUCTION GATE SEAMS [WIRED, default-ON, kill-switchable, emit SSB on block]: enforce_taint_sink_gate (production_agent_rpc.rs:224) = per-argv capability gate (the substring fast-path); enforce_trifecta_quarantine_gate (production_agent_rpc.rs:334) = G-7 lethal-trifecta SESSION-level gate (the WI-1 PRIMARY backstop to strengthen).
- tainted_context() — 16 callers incl all four ingest sites + both production gates + the contract test (crates/contracts/tests/g1_taint_propagation_contract.rs: untrusted_web_ingest_blocks_git_commit, untrusted_edit_content_blocks_edit_sink, ten_benign_flows_zero_false_blocks FP-guard). The WI-1 paraphrase case EXTENDS this contract.
- taint_flow.rs:472 process_global_slot_is_shared — the WI-2 VERIFY-FIRST anchor (partial shared slot may exist).
- PROMPT INJECTION DEFENSE (E-37) [PARTIAL] Spotlighting + PromptArmor; output_sanitizer.rs / output_classifier.rs / egress allowlist / AgentDojo CI all ABSENT — the WI-3 build target.
- G-3 egress IP-pin (the WI-3 egress allowlist composes it); F-3 CaMeL tool-filter; I-1 supervisor (generator cannot reach gate code).
- G-6 SAFEDELTA AgentDojo arm [BENCH-PENDING] — the WI-4 number source (K-13's row, gated F-2b RunPod).
- live SSB baseline ~268/577 (re-grep at edit, W-9); J-101 prefer-reuse TaintFlow / TaintSinkBlocked rows, net 0..+1.

## Reads (mandatory before coding)
1. workflow/reasoning/owasp-llm-agentic-gap-analysis-2026-06-25.md Section D IN FULL (the SUBSTRATE + WIRING + the ONE GENUINE RESIDUAL GAP), Section E WI-1/WI-2/WI-3, Section F (full CaMeL DEFERRED; prose-guards REJECTED), Section G (process_global_slot + Recallable-does-not-block + E-37-absent-files — VERIFY before scoping).
2. crates/safety/src/provenance.rs (Provenance + Provenanced<T> + from_file_screened) + capability_gate.rs (allow_tool_with_origin + the Recallable line :96) — re-grep live at authoring (W-7).
3. crates/orchestrator/src/production_agent_rpc.rs around :224 (per-argv gate) + :334 (trifecta SESSION gate, the WI-1 PRIMARY) + ~234 (the GD-5 honest-scope comment) + :2321 (from_file_screened production caller).
4. crates/safety/src/taint_flow.rs:472 (process_global_slot — the WI-2 VERIFY-FIRST anchor) + crates/ipc-types postcard (the Tier-C IPC boundary WI-2 threads taint across).
5. crates/contracts/tests/g1_taint_propagation_contract.rs (the contract WI-1 extends with a paraphrase case + keeps the ten-benign FP guard) + the E-37 plan file (the absent perimeter files WI-3 builds) + the G-6 SAFEDELTA AgentDojo arm shape (K-13's number source, BENCH-PENDING).

## Tasks (WAVE 1 — WI-1 keystone first; WI-2 needs the verify; WI-3 cheap activation)
1. [WI-1a PRIMARY] strengthen enforce_trifecta_quarantine_gate (production_agent_rpc.rs:334): untrusted-in-context AND destructive-sink-reached AND egress/exfil-capability-present => HITL/quarantine REGARDLESS of per-value substring match. The load-bearing transformation-surviving backstop (it does not need the value to survive paraphrase — it is session-level). Default-ON, kill-switchable, emits the existing SSB block row.
2. [WI-1b SECONDARY] taint-WINDOW heuristic: mark argv the model produced AFTER consuming untrusted context in the SAME turn (span/turn window) as DERIVED-UNTRUSTED provenance, so a paraphrase carries trust forward; the per-argv capability gate then denies the destructive sink. TD-3 deterministic (the window is span-bounded, the LLM does not adjudicate). OBSERVE-ONLY first (GD-4): emit the would-mark set, measure the FP rate on the ten-benign flows, before defaulting the denial ON.
3. [WI-1c FAST PATH] keep the substring gate (enforce_taint_sink_gate :224) as the cheap exact-match first check. ASI06 DECISION: treat Recallable-written-after-an-untrusted-ingest-in-session as derived-untrusted under the window (fail-closed; capability_gate.rs:96).
4. [WI-2] VERIFY taint_flow.rs:472 process_global_slot cross-process coverage FIRST; if a gap is confirmed, propagate Provenance across the Tier-C postcard IPC boundary so cross-process destructive sinks see upstream taint. Scope to the CONFIRMED gap only (do NOT rebuild a working shared slot). If the verify finds it already covered, the leg degrades to a contract test asserting cross-process taint + a note (recorded honestly).
5. [WI-3] E-37 perimeter completion: build output_sanitizer.rs + output_classifier.rs + the egress allowlist (composing G-3 IP-pin) + an AgentDojo CI job that runs in the suite with a recorded baseline.
6. workflow/reasoning/J-101-decisions.md (the ASI06 Recallable DECISION + the WI-2 verify finding + the WI-1 window FP-rate observe-only delta).

## WIRING_CONTRACT
produces_symbols:
  - the strengthened enforce_trifecta_quarantine_gate (untrusted-in-context AND destructive-sink AND egress-cap => HITL/quarantine regardless of per-value match) [WI-1a]
  - the taint-WINDOW derived-untrusted provenance marker over same-turn-after-untrusted-ingest argv [WI-1b]
  - cross-process Provenance propagation across the Tier-C postcard IPC boundary (only where the verify confirms a gap) [WI-2]
  - output_sanitizer.rs + output_classifier.rs + the egress allowlist + the AgentDojo CI job [WI-3]
consumes_symbols:
  - Provenance / Provenanced<T> / from_file_screened (crates/safety/src/provenance.rs; G-1) — extended, not re-built
  - enforce_taint_sink_gate :224 (the substring fast-path) + enforce_trifecta_quarantine_gate :334 (G-7, strengthened) + the GD-5 honest-scope comment ~234
  - capability_gate.rs allow_tool_with_origin + the Recallable line :96 (the ASI06 decision) — cited
  - taint_flow.rs:472 process_global_slot (WI-2 VERIFY-FIRST) + crates/ipc-types postcard (the Tier-C boundary)
  - G-3 egress IP-pin (WI-3 egress allowlist composes) + the E-37 perimeter (the absent files WI-3 builds) + G-6 SAFEDELTA AgentDojo arm (K-13's number, INVOKE-when-it-lands)
  - g1_taint_propagation_contract.rs (the contract WI-1 EXTENDS with a paraphrase case; keeps the ten-benign FP guard)
reachable_from:
  - a live `elai run` where the model reads a poisoned doc, PARAPHRASES the malicious instruction, and the paraphrase flows to a destructive sink -> the WI-1 window/trifecta gate BLOCKS (NOT a unit test calling the gate) [PRODUCTION, the keystone]
  - a live cross-process agent whose destructive sink carries upstream untrusted taint -> the WI-2 propagation BLOCKS [PRODUCTION]
  - a live model output containing a seeded exfil host -> the WI-3 egress allowlist + output_sanitizer BLOCK [PRODUCTION]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies any leg (R7); a unit test calling the gate is NOT the keystone reachability proof
stub_ban:
  - NO prose system-prompt guard as a primary defense (OWASP + ELAI agree they fail; structural taint is the posture; Section F)
  - NO full CaMeL control/data-flow interpreter (research-grade, HD/IC tension; the WI-1 hybrid is trifecta+window, Section F DEFERRED)
  - NO rebuild of a working process_global_slot (WI-2 VERIFY first, scope to the confirmed gap only; Section G)
  - NO fabricated injection-defense number (GD-3: gate-ON-vs-OFF ablation; the SAFEDELTA number is K-13's [BENCH-PENDING] row, F-2b-gated)
  - NO default-ON of the WI-1b window denial before the ten-benign FP rate is measured (GD-4 observe-first); a new kill-switch gets a wire-dark-allowlist row (W-2/R9)
  - NO LLM adjudicating the taint window (TD-3; the window is span-bounded deterministic)
frontend_pair:
  taint-block + quarantine events surface on the existing safety/SSB timeline; the AgentDojo CI baseline is a CI artifact, not a dashboard tile. W-6-justified (a security gate, no new UI surface) or decide an injection-defense tile at edit (W-4).
observable_effect:
  trigger: "an AgentDojo/SAFEDELTA fixture where a PARAPHRASED (substring-evading) injection reaches a destructive sink (WI-1); a cross-process untrusted->sink fixture (WI-2); a model output with a seeded exfil host (WI-3)"
  expect_ssb: "the existing taint-flow / TaintSinkBlocked / quarantine block row on each fire (recomputable GD-2); net 0..+1 new action_type (prefer-reuse)."
  expect_artifact: "WI-1: the paraphrased injection is BLOCKED by the window/trifecta gate AND the ten-benign-flows FP guard stays GREEN (extend g1_taint_propagation_contract.rs with a paraphrase case + the FP arm — the MEASURED done-gate). WI-2: a cross-process untrusted->sink fixture blocks; same-process behavior unchanged. WI-3: the AgentDojo CI job runs in the suite with a recorded baseline; the egress allowlist blocks a seeded exfil host. NEGATIVE (R2): the WI-1b window-OFF arm LETS the paraphrase through (proves the window is load-bearing); a benign same-turn-after-read flow is NOT blocked (the FP guard)."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-101_transformation_surviving_taint_binding.md --run-trigger
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/production_agent_rpc.rs,.elai_cc/crates/safety/src/taint_flow.rs --json   # the strengthened trifecta gate + window marker + cross-process propagation have non-test production callers
rg -n "derived.?untrusted|taint.?window" .elai_cc/crates/safety/src/ | rg -v "test|contract"   # the WI-1b window marker exists in production
rg -n "enforce_trifecta_quarantine_gate" .elai_cc/crates/orchestrator/src/production_agent_rpc.rs   # the strengthened WI-1a backstop
test -f .elai_cc/crates/orchestrator/src/output_sanitizer.rs && test -f .elai_cc/crates/orchestrator/src/output_classifier.rs   # WI-3 perimeter files exist
rg -n "paraphrase|transformation.?surviving|derived.?untrusted" .elai_cc/crates/contracts/tests/g1_taint_propagation_contract.rs   # the WI-1 paraphrase contract case
rg -n "AgentDojo" .elai_cc/.github 2>/dev/null || rg -n "agentdojo|AgentDojo" .elai_cc/ci* 2>/dev/null   # the WI-3 AgentDojo CI job (path re-grepped at edit)
test -f workflow/reasoning/J-101-decisions.md
```

## ACCEPTANCE
- [ ] WI-1a: enforce_trifecta_quarantine_gate quarantines/HITL on untrusted-in-context AND destructive-sink AND egress-cap REGARDLESS of per-value substring match (the transformation-surviving backstop).
- [ ] WI-1b: same-turn-after-untrusted-ingest argv is marked derived-untrusted (span-bounded, TD-3); OBSERVE-ONLY first, the ten-benign FP rate measured before the denial defaults ON; the ASI06 Recallable DECISION applied.
- [ ] WI-1 MEASURED done-gate: a PARAPHRASED injection reaching a destructive sink is BLOCKED; the ten-benign-flows FP guard stays GREEN (g1_taint_propagation_contract.rs extended).
- [ ] WI-2: process_global_slot verified FIRST; cross-process Provenance propagation added only where a gap is confirmed; same-process behavior unchanged.
- [ ] WI-3: output_sanitizer.rs + output_classifier.rs + the egress allowlist built; the AgentDojo CI job runs with a recorded baseline; a seeded exfil host is blocked.
- [ ] NO fabricated injection number (the SAFEDELTA delta is K-13's [BENCH-PENDING] row); no prose-guard primary; no full CaMeL interpreter (Section F). net 0..+1 SSB; cargo check --workspace GREEN; auditor PASS on -p elai-safety + -p elai-orchestrator (+contracts).

## WIRING DONE-GATE (clause by clause)
- W-1: done = a live paraphrased-injection fixture is BLOCKED at the REAL gate seam (R10 cross-seam integration smoke: a multi-turn run that reads a poisoned doc THEN reaches a sink), not a unit test calling the gate. W-2: the WI-1b window denial + any new kill-switch get wire-dark-allowlist rows (R9). W-3: WI-3 is an ACTIVATION of planned E-37, not a port.
- W-5: trigger = a live AgentDojo/SAFEDELTA paraphrase fixture + a cross-process fixture + a seeded-exfil output. W-7: provenance.rs + capability_gate.rs:96 + production_agent_rpc.rs:224/334 + taint_flow.rs:472 + the E-37 absent files re-grepped at edit. W-9: ~268/577 re-grep.
- GD-1: strengthen/extend G-1/G-7/E-37/F-3, compose I-1, never re-author. GD-3: SAFEDELTA gate-ON-vs-OFF, never a leaderboard. GD-4: WI-1b/WI-2 observe-first. GD-5: the GD-5 honest-scope comment is the residual-gap source. TD-3: the window is deterministic. R7: test callers excluded. R10: the cross-seam smoke is mandatory.

## Tests to add
- crates/contracts/tests/g1_taint_propagation_contract.rs (EXTEND, ~+60 LOC, do NOT fork): (a) a PARAPHRASED untrusted->sink case is BLOCKED by the window/trifecta gate (the keystone); (b) the WI-1b window-OFF arm LETS the paraphrase through (R2 negative — proves load-bearing); (c) the existing ten_benign_flows_zero_false_blocks FP guard stays GREEN with the window ON (no new false blocks); (d) a Recallable-written-after-untrusted-ingest fixture is treated as derived-untrusted (the ASI06 DECISION); (e) an empty/missing taint context fails-CLOSED (R1).
- crates/contracts/tests/j101_cross_process_taint_contract.rs (~50 LOC): a cross-process untrusted->sink blocks; same-process unchanged (WI-2; only if the verify confirms a gap, else an assertion-of-existing-coverage test + a note).
- crates/contracts/tests/j101_e37_perimeter_contract.rs (~50 LOC): output_sanitizer/output_classifier block a seeded payload; the egress allowlist blocks a seeded exfil host (WI-3).

## Files touched
- MODIFY: crates/orchestrator/src/production_agent_rpc.rs (strengthen enforce_trifecta_quarantine_gate :334 for WI-1a; the WI-1b window provenance at the per-argv path)
- MODIFY: crates/safety/src/taint_flow.rs (the WI-1b derived-untrusted window marker; the WI-2 cross-process propagation where confirmed) + crates/safety/src/capability_gate.rs (the ASI06 Recallable-under-window DECISION) + crates/safety/src/provenance.rs (a DerivedUntrusted provenance variant if needed)
- CREATE: crates/orchestrator/src/output_sanitizer.rs + crates/orchestrator/src/output_classifier.rs (WI-3 E-37 perimeter) + the egress allowlist (compose G-3) + the AgentDojo CI job (exact CI path re-grepped W-7)
- EXTEND: crates/contracts/tests/g1_taint_propagation_contract.rs (+ the paraphrase + FP + Recallable cases) + the two new j101_* contract files
- CREATE: workflow/reasoning/J-101-decisions.md (the ASI06 Recallable DECISION + the WI-2 verify finding + the WI-1b FP observe-only delta)

## Citations / Grounding
- workflow/reasoning/owasp-llm-agentic-gap-analysis-2026-06-25.md Section D (the live-code taint trace, codegraph file:line-grounded — the SUBSTRATE/WIRING/RESIDUAL split), Section E WI-1/WI-2/WI-3, Section F (full CaMeL DEFERRED; prose-guards REJECTED), Section G (process_global_slot / Recallable / E-37-absent verify-first).
- ELAI ground truth (codegraph-grounded 2026-06-25; re-verify at edit, W-7): provenance.rs Provenance/Provenanced<T>/from_file_screened [WIRED]; capability_gate.rs allow_tool_with_origin + Recallable :96 [WIRED]; production_agent_rpc.rs enforce_taint_sink_gate :224 + enforce_trifecta_quarantine_gate :334 + GD-5 honest-scope ~234 + from_file_screened :2321 [WIRED, default-ON]; taint_flow.rs:472 process_global_slot [PARTIAL]; g1_taint_propagation_contract.rs [WIRED]; E-37 output_sanitizer/classifier/egress/AgentDojo-CI [ABSENT]; G-6 SAFEDELTA AgentDojo [BENCH-PENDING]. live ~268/577 (W-9).
- arXiv:2503.18813 CaMeL §3.1 (the control/data-flow binding the hybrid approximates; the FULL interpreter is the ceiling, DEFERRED — [EXTERNAL], design-rationale). OWASP LLM01/LLM05 + ASI01/ASI06/ASI07 (genai.owasp.org 2025; the threat rubric).
- Hard rules invoked: GD-1 (extend G-1/G-7/E-37, never re-author); GD-3 (gate-ON-vs-OFF, never a leaderboard); GD-4 (WI-1b/WI-2 observe-first); GD-5 (the in-code honest-scope is the gap source); TD-3 (deterministic window); R1/R7/R9/R10; W-7/W-9.
- K-13 is the K-native PROVE+PACKAGE half (the OWASP control-mapping + the gate-ON-vs-OFF injection number); J-101 is the technical hardening it recorded OUT-OF-K. WI-4 (the SAFEDELTA RUN) + WI-5 (the mapping doc) are K-13's, not this file.

## Open questions / risks
- WI-1 FALSE-BLOCK risk (the utility-killing one): an over-broad taint window blocks benign same-turn-after-read flows (the global-join-collapses-utility-to-33% lesson, RTBAS). Mitigation = span-bounded window + OBSERVE-ONLY first + the ten-benign FP guard as a hard gate; measure the FP rate before defaulting ON (GD-4).
- WI-2 may find process_global_slot ALREADY covers the cross-process case (Section G) -> the leg degrades to an assertion test + a note, recorded honestly, not padded into code.
- ASI06 Recallable DECISION: treating Recallable-after-untrusted as derived-untrusted is conservative (fail-closed) but could over-block legitimate memory recall; the window scoping (same-session, after-an-untrusted-ingest) bounds it. Confirm the intended posture at authoring vs the user.
- The injection NUMBER is K-13's [BENCH-PENDING] row on G-6/F-2b RunPod — J-101 ships the fixture + contract, the SAFEDELTA delta fills in later; do NOT let the contract imply a measured AgentDojo result before the run lands (KD-1, GD-3).

## Next
Closes the dominant indirect-injection path (paraphrased untrusted->sink) the wired per-argv substring gate misses, adds cross-process taint coverage, and completes the E-37 perimeter — the technical hardening of the spine K-13 maps + proves. The MEASURED done-gate (a paraphrase BLOCKED, FP guard green) is the proof; the full gate-ON-vs-OFF SAFEDELTA number is K-13's BENCH-PENDING row. Pairs with J-102 (the config-ingress admission gate — untrusted CONFIG into the model's instruction surface; J-101 is untrusted DATA into sinks). Both are the "ingress governance" theme.

End of J-101.
