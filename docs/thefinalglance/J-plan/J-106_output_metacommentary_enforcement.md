# J-106: output-side meta-commentary enforcement (op-emission roles only) — a runtime banned-phrase reject + work-log output grammar + action-density metric, the OUTPUT-side twin of I-32, HARD-bounded to op-emission by HD-3

STATUS: PLANNED (grep-confirmed gap: I-32 scopes roles + strips narrative persona on the INPUT side, but ELAI has NO runtime OUTPUT validator / work-log grammar / action-density metric; J-106 adds them ON OP-EMISSION ROLES ONLY, NEVER reasoning roles; NEW, small; composes I-32/I-47/E-35; default-OFF observe-only until the reasoning-corpus false-positive rate is bounded at 0 — the HD-3 boundary is the whole risk)

- Track J | lever ACT-DONT-NARRATE-AT-RUNTIME (an op-emission role emits a work-log, not ceremony; a reasoning role is left free to think) | grounding STRONG (I-32 role_contracts.rs + report_fns.rs role classification / I-47 prompt_hardening.rs scream path / E-35 eval seam code-grounded this session; re-grep at edit W-7) / HOUSE-DOCTRINE (the global "act, don't narrate" / measured-Fable-signature rule operationalized product-side) | nature BUILD (new runtime validator) | effort S-M | WAVE 1
- Source: workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md §4 (J-106) + the carry-forward "META-COMMENTARY OUTPUT ENFORCEMENT" gap (grep-confirmed absent product-side). Doctrine: J-0_INDEX.md + the global output-formatting doctrine (result-first openings, ~4 tool-actions per prose block, low words/message) + .claude/rules/output-formatting.md.
- This is a HOUSE-DOCTRINE + HARNESS-CONVENTION gap, NOT a paper claim — there is no external number to cite or to fabricate. The evidence is the ELAI codebase grep (absent) + the workflow's own output-formatting doctrine. The done-gate is ELAI-measured on an ELAI op-emission/reasoning corpus.

## Context
THE GAP (grep-confirmed absent in .elai_cc): I-32 STRUCTURAL ROLE SCOPING strips narrative persona / enforces schema-first role contracts on the INPUT side (report_fns.rs role-system-prompt build + role_contracts.rs + i32_role_scoping_contract.rs — BUILT). I-47 is the dropped-row scream path (prompt_hardening.rs). But ELAI has NO runtime OUTPUT validator (a banned-phrase reject + retry), NO work-log output GRAMMAR (READ/FOUND/DECISION/ACTION/RESULT-shaped), and NO action-density EVAL metric. So the model can still emit ceremony / affirmation / hedging / narration on the OUTPUT side even after I-32 scoped the role on the input side. J-106 closes the output half.

THE HARD BOUNDARY (HD-3 — the critical constraint, DO NOT VIOLATE): "never constrained-decode reasoning roles; grammars only on pure op-emission." The output grammar + the banned-phrase reject apply ONLY to OP-EMISSION / work-log roles (executor tool-call narration, status emission), NEVER to reasoning / synthesis roles (a reasoner MUST be free to think in prose). Decoding is sacred for reasoning; this gate is op-emission-only. The role classification I-32 already computes is REUSED to scope J-106 — a reasoning role is provably untouched.

WHY OUTPUT-SIDE, NOT JUST INPUT-SIDE: an input-side persona-strip (I-32) reduces ceremony but does not GUARANTEE the output is a work-log — the model can still open with an affirmation or narrate diligence. A deterministic OUTPUT validator (a frozen banned-phrase table + a bounded retry, the I-46 frozen-fixup-table pattern applied to output) is the structural guarantee, scoped to the roles where ceremony is pure cost.

## Existing state (what J-106 composes — do NOT re-author, GD-1)
- STRUCTURAL ROLE SCOPING [PLANNED(I-32)] drop narrative personas from executor/researcher, KEEP for auditor/critic/anti-spiral, per-role A/B measured (IC-17); report_fns.rs role-system-prompt build + role_contracts.rs (grep-confirmed this session) + i32_role_scoping_contract.rs. J-106 REUSES the role classification to scope the output gate to OP-EMISSION roles (HD-3).
- VERIFIER-INPUT HARDENING / SCREAM PATH [PLANNED(I-47)] prompt_hardening.rs dropped_row_count scream path + worker discipline rules injected mechanically, byte-stable (HD-2-safe) — the SCREAM-PATH PATTERN J-106 reuses for a dropped/violating output row.
- STRUCTURED OUTPUT SUBSTRATE [PARTIAL] E-35 XGrammar + CodeAct + SEARCH/REPLACE; per-mode grammars + CI compile ceiling; NEVER force-JSON reasoning roles (HD-3) — the eval-metric SEAM + the op-emission-only grammar precedent.
- BASH INTERCEPTOR + FIXUP [PLANNED(I-46)] the FROZEN deterministic fixup table pattern J-106's banned-phrase table mirrors (a frozen table, gate-manifest-eligible).
- CODING METRICS [PARTIAL] E-49 STABLE-Pass@1 + $/CTS + 7-tuple — the eval-corpus the action-density metric joins.
- live SSB baseline ~268/577 (re-grep at edit, W-9); J-106 prefers-reuse an existing emission/violation row; +OutputCommentaryRejected ONLY if a journaled reject row is wanted (net 0..+1).

## Reads (mandatory before coding)
1. workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md §4 IN FULL (the validator, the grammar, the action-density metric, the HD-3 boundary).
2. .claude/rules/output-formatting.md + the global "act, don't narrate" / measured-Fable-signature rule (result-first openings, ~4 tool-actions per prose block, low words/message) — the doctrine this operationalizes.
3. crates/orchestrator/src/report_fns.rs (the I-32 role classification + role-system-prompt build — the scope source) + role_contracts.rs — re-grep live at edit (W-7).
4. crates/tools/src/prompt_hardening.rs (the I-47 scream-path pattern) + the E-35 eval-metric seam + the op-emission role OUTPUT path (where executor/status output is produced — the validator call-site).
5. crates/tools/src/bash_validation.rs (the I-46 frozen-fixup-table pattern the banned-phrase table mirrors).

## Tasks (WAVE 1 — op-emission only; the HD-3 reasoning-role exemption is the load-bearing assert)
1. [BANNED-PHRASE TABLE + REJECT] a FROZEN banned-phrase set (ceremony / affirmation / hedging openers — the output-formatting doctrine's ban list) that, on an OP-EMISSION role's output, triggers a REJECT + a BOUNDED retry (the I-46 frozen-fixup-table pattern; the table is gate-manifest-eligible, mutable=0). DETERMINISTIC (TD-3, a frozen table not an LLM judge). PRODUCER: crates/orchestrator/src/output_commentary_gate.rs (new).
2. [WORK-LOG GRAMMAR] a work-log output grammar (READ / FOUND / DECISION / ACTION / RESULT-shaped) for OP-EMISSION roles only — a reject-and-retry-to-shape, NOT a hard constrained-decode of the token stream (HD-3: grammars only on pure op-emission, and even here a reject+retry, never forcing reasoning). CONSUMER: the op-emission role output path calls the validator.
3. [HD-3 SCOPE] reuse the I-32 role classification to scope BOTH legs to op-emission roles. A REASONING / synthesis role's output is provably UNTOUCHED (the load-bearing exemption). A contract test feeds a reasoning role ceremony-laden prose and asserts it passes through unchanged.
4. [ACTION-DENSITY METRIC] an eval-corpus signal: tool-actions per prose block / words-per-message, recorded on the E-49 eval set (NOT a runtime gate — an eval signal that measures whether the doctrine is being met). CONSUMER: the eval harness (E-35/E-49 seam).
5. [SCREAM PATH] a dropped/violating output row uses the I-47 scream-path pattern (a violation is visible, never silently swallowed).
6. [optional +OutputCommentaryRejected] if a journaled reject row is wanted (for replay/audit of WHAT was rejected + retried), add ONE Pattern-2 SSB action_type — full 9-tuple lockstep, TOTAL_VARIANTS from LIVE at edit (W-9). Otherwise net 0 (reuse an existing emission/violation row).
7. workflow/reasoning/J-106-decisions.md (the SSB decision; the banned-phrase table seed; the reasoning-corpus FP=0 observe-only finding).

## WIRING_CONTRACT
produces_symbols:
  - the FROZEN banned-phrase table + the deterministic reject + bounded retry on op-emission output [output_commentary_gate.rs]
  - the work-log output grammar (READ/FOUND/DECISION/ACTION/RESULT) reject-to-shape for op-emission roles
  - the HD-3 scope gate (reuse I-32 role classification; reasoning roles provably untouched)
  - the action-density eval metric (tool-actions per prose block / words-per-message) on the E-49 corpus
  - the I-47 scream-path for a dropped/violating output row
  - OPTIONAL +OutputCommentaryRejected (Pattern-2) if a journaled reject row is chosen
consumes_symbols:
  - the I-32 role classification (report_fns.rs / role_contracts.rs) — the scope source (op-emission vs reasoning), cited not re-authored
  - the I-47 scream-path pattern (prompt_hardening.rs) — the dropped/violating-row idiom
  - the E-35 eval-metric seam + E-49 coding metrics — the action-density corpus
  - the I-46 frozen-fixup-table pattern (bash_validation.rs) — the banned-phrase table is the same frozen, gate-manifest-eligible shape
  - the op-emission role OUTPUT path (the executor/status output call-site) — the validator consumer
reachable_from:
  - a live `elai run` where an OP-EMISSION role emits ceremony-laden output -> the validator rejects + retries to a compliant work-log-grammar output [PRODUCTION]
  - a live REASONING / synthesis role emitting the same prose -> it passes UNTOUCHED (the HD-3 exemption) [PRODUCTION, the load-bearing boundary]
  - the eval harness records action-density on the E-49 corpus [PRODUCTION, the eval signal]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies the validator or the eval leg (R7)
stub_ban:
  - NO constraint applied to a REASONING / synthesis role's output (HD-3 — decoding is sacred for reasoning; the gate is op-emission-only; a reasoning-role false-positive is a BLOCK)
  - NO hard constrained-DECODE of the op-emission token stream (a reject+retry, the I-46 pattern; never force-JSON the stream — even op-emission)
  - NO LLM judging whether output is "ceremony" (TD-3 — a FROZEN deterministic banned-phrase table, mutable=0)
  - NO silent swallow of a violating row (the I-47 scream-path)
  - NO default-ON before the reasoning-corpus false-positive rate is bounded at ZERO (GD-4); ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE needs a wire-dark-allowlist row + promotion_plan_id J-106-promotion (W-2/R9)
frontend_pair:
  none required at runtime; the action-density metric surfaces on the E-49 eval/metrics surface. Decide a dashboard tile at edit (W-4) or W-6-justify N/A.
observable_effect:
  trigger: "an OP-EMISSION role emits a ceremony-laden output (banned-phrase opener + narration); the SAME prose from a REASONING role; an eval-corpus run"
  expect_ssb: "an existing emission/violation row on the reject (net 0 new action_type); OR if chosen, OutputCommentaryRejected rows recomputable from the journal (GD-2). bump TOTAL_VARIANTS from LIVE at edit (W-9)."
  expect_artifact: "the op-emission ceremony output is REJECTED + retried to a compliant work-log-grammar output; the reasoning-role prose is UNTOUCHED (zero false-positive on the reasoning corpus — the HD-3 done-gate); the action-density metric is recorded on the eval set. NEGATIVE: a reasoning-role output that gets constrained is a BLOCK (the boundary is load-bearing)."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-106_output_metacommentary_enforcement.md --run-trigger
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/output_commentary_gate.rs --json   # the validator has a non-test production caller (the op-emission output path)
rg -n "op_emission|OpEmission|role.*class|reasoning|untouched" .elai_cc/crates/orchestrator/src/output_commentary_gate.rs | rg -v "test"   # the HD-3 scope is enforced from the I-32 classification
rg -n "banned|ceremony|work_log|READ.*FOUND.*DECISION|reject.*retry" .elai_cc/crates/orchestrator/src/output_commentary_gate.rs   # the frozen table + grammar + bounded retry present
rg -n "action_density|tool_actions_per|words_per_message" .elai_cc/crates/orchestrator/src | rg -v "test"   # the eval metric wired into the E-49/E-35 seam
test -f workflow/reasoning/J-106-decisions.md
```

## ACCEPTANCE
- [ ] an OP-EMISSION role's ceremony-laden output is deterministically REJECTED + retried to a compliant work-log-grammar (READ/FOUND/DECISION/ACTION/RESULT) output.
- [ ] a REASONING / synthesis role's output is UNTOUCHED — ZERO false-positive on the reasoning corpus (the HD-3 done-gate, the load-bearing boundary).
- [ ] the banned-phrase table is FROZEN + deterministic (the I-46 pattern, gate-manifest-eligible); the retry is bounded; no LLM judges "ceremony" (TD-3).
- [ ] the action-density metric (tool-actions per prose block / words-per-message) is recorded on the E-49 eval corpus (an eval signal, NOT a runtime gate).
- [ ] a dropped/violating output row uses the I-47 scream-path (never silently swallowed).
- [ ] ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE default-OFF with a wire-dark-allowlist row (promotion_plan_id=J-106-promotion); observe-only until the reasoning-corpus FP rate is bounded at 0 (GD-4).
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator (+contracts); action_type_lock GREEN if +OutputCommentaryRejected. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = a live op-emission ceremony output is rejected+retried AND a live reasoning output is untouched, observed end-to-end — not "the table exists". W-2: ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE gets a wire-dark-allowlist row + promotion_plan_id J-106-promotion. W-3: composes I-32/I-47/E-35, not a port.
- W-5: trigger = a live `elai run` op-emission ceremony output + a reasoning output + an eval-corpus run. W-7: report_fns.rs role classification + prompt_hardening.rs + the op-emission output path + the E-49 seam re-grepped at edit. W-9: ~268/577 re-grep; bump only if +OutputCommentaryRejected.
- HD-3: the boundary IS the gate — reasoning roles untouched, op-emission only, reject+retry never force-decode. TD-3: frozen deterministic table, no LLM judge. GD-4: observe-only until reasoning-corpus FP=0. R7/R9. GD-2: optional reject row recomputable.

## Tests to add
- crates/contracts/tests/j106_output_commentary_contract.rs (~120 LOC): (a) an op-emission ceremony fixture (banned-phrase opener + narration) is rejected + retried to a compliant work-log-grammar output; (b) the HD-3 arm: the SAME prose from a REASONING role passes UNTOUCHED (zero false-positive — the load-bearing assert); (c) a reasoning-role output that gets constrained is a BLOCK (the boundary is enforced); (d) the banned-phrase reject is deterministic across re-runs under ELAI_SEED (TD-3, frozen table); (e) the retry is bounded; (f) a dropped/violating row uses the I-47 scream-path; (g) the action-density metric is recorded on a fixture corpus.

## Files touched
- CREATE: crates/orchestrator/src/output_commentary_gate.rs (the frozen banned-phrase table + reject + bounded retry + work-log grammar + the HD-3 scope from the I-32 classification; exact seams re-grepped W-7)
- MODIFY: crates/orchestrator/src/report_fns.rs (consume the I-32 role classification to scope the gate to op-emission roles; the op-emission output path calls the validator)
- MODIFY: the E-35/E-49 eval seam (record the action-density metric on the eval corpus)
- MODIFY (only if +OutputCommentaryRejected): crates/orchestrator/src/ssb.rs + ssb/payloads.rs + action_type.rs + tests/action_type_lock.rs + .golden (full 9-tuple lockstep, TOTAL_VARIANTS from LIVE)
- CREATE: workflow/reasoning/J-106-decisions.md
- CONFIG: .elai_cc/wire-dark-allowlist.toml row for ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE (type=experiment, owner, expiry, promotion_plan_id=J-106-promotion, smoke) + the frozen banned-phrase table added to .claude/gate-manifest.txt (R8, mutable=0)

## Citations / Grounding
- workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md §4 (the full J-106 spec) + the carry-forward META-COMMENTARY OUTPUT ENFORCEMENT gap (grep-confirmed absent product-side).
- .claude/rules/output-formatting.md + the global "act, don't narrate" / measured-Fable-signature doctrine (result-first openings, ~4 tool-actions per prose block, low words/message) — the HOUSE-DOCTRINE evidence (NOT a paper claim; nothing external to cite or fabricate).
- ELAI ground truth (code-grounded this session; re-verify at edit W-7): I-32 report_fns.rs role classification + role_contracts.rs + i32_role_scoping_contract.rs [PLANNED/BUILT]; I-47 prompt_hardening.rs scream path [PLANNED]; E-35 eval seam [PARTIAL]; E-49 coding metrics [PARTIAL]; I-46 frozen-fixup-table pattern (bash_validation.rs). live ~268/577 (W-9).
- Hard rules invoked: HD-3 (NEVER constrained-decode reasoning roles; grammars/gates only on pure op-emission — the load-bearing boundary); TD-3 (frozen deterministic banned-phrase table, no LLM judge); GD-4 (observe-only until reasoning-corpus FP=0); GD-2 (recomputable reject row); W-1/W-2/W-7/W-9; R7/R9/R8 (the table frozen in the gate-manifest).

## Open questions / risks
- HD-3 FALSE-POSITIVE (highest, the whole risk): the gate must NEVER touch a reasoning role. Mitigation = scope from the I-32 classification + the reasoning-corpus FP=0 done-gate + observe-only first; a single reasoning false-positive blocks default-ON.
- the BANNED-PHRASE TABLE seed: too broad over-blocks legitimate op-emission, too narrow misses ceremony. DECISION = seed from the output-formatting doctrine's explicit ban list, frozen, expanded only via a gate-manifest review (R8). // DECISION.
- the SSB decision (journal +OutputCommentaryRejected vs net-0): net 0 is the default; journal only if reject-audit is wanted; resolved in J-106-decisions.md. // DECISION.
- WORK-LOG GRAMMAR rigidity: a strict grammar could distort a legitimate op-emission edge case. Mitigation = reject+retry-to-shape (not force-decode), bounded retries then pass-through-with-a-scream (I-47), observe-only first.

## Next
Closes the output half of I-32: an op-emission role emits a work-log not ceremony, enforced by a frozen deterministic table + bounded retry, while reasoning roles stay free to think (HD-3). Operationalizes the global "act, don't narrate" doctrine into a product-side runtime check, measured by an action-density eval metric. Composes I-32 (the scope) / I-47 (the scream) / E-35 (the eval seam); binds HD-3 as the load-bearing boundary.

End of J-106.
