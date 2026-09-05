GROUNDING SWEEP -- phase B-28
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-28_focus_tunnel.md
Date: 2026-05-14
FSM: PLANNING | Session: s-27436

SPEC REFERENCES
docs/specs/guide/focus-tunnel-design.txt (FULL) | GROUNDED | file exists; FIM tokens L71-73, mode selection L47-58, streaming val L183-213, pipeline L225-244, dep mgmt L279-310, token savings L318-332
docs/specs/guide/post-phase4-implementation-order.txt E-1..E-7 | GROUNDED | file exists; GROUP E at L319-507, micro-step max=2 at L504, mode selection rules at L393-399
B-20 3-tier edit matching | GROUNDED | B-20_tool_registry_unification.md L70: "exact -> whitespace-normalized -> ellipsis-elision"; phase-4B.txt L191-193 confirms
3F deprecated_patterns | MISMATCH | phase-3F.txt has ZERO hits for "deprecated_patterns". Defined in anti-staleness-enforcement.txt L112-522 and implemented in plugins/crates (crates/cli/src/dep_audit.rs, crates/plugins/src/import_extractor.rs). B-28 WIRING_CONTRACT cites "3F" as source -- wrong phase attribution.
3J sidecar FIM token support | MISMATCH | phase-3J.txt has ZERO hits for "FIM", "fim_prefix", "supports_fim". Phase 3J covers vLLM/DoLa/TurboQuant on Gemma4 (hardware rejected per memory). No FIM content defined for 3J. FIM exists only in crates/scaffold/src/fim.rs (scaffold-level, not providers/sidecar).

CLAIM CLASSIFICATION
Qwen2.5-Coder native FIM via <|fim_prefix|> tokens | GROUNDED | crates/scaffold/src/fim.rs:40-42: QWEN_FIM_PREFIX/SUFFIX/MIDDLE consts, comment "Verified against the upstream tokeniser"; focus-tunnel-design.txt L111
B-20 3-tier edit matching exposes hunk API | GROUNDED | B-20 task 6 "Wire hunks through 3-tier edit matching"; phase-4B.txt L191-193
rules deprecated_patterns from phase-3F | UNGROUNDED | phase-3F.txt has no deprecated_patterns content; actual source is anti-staleness-enforcement.txt + plugins crate; spec attribution in WIRING_CONTRACT is incorrect
sidecar FIM token support from phase-3J | UNGROUNDED | phase-3J has zero FIM content; it defines vLLM/DoLa/Gemma4 (rejected hardware); FIM in providers/sidecar is entirely NEW work with no spec backing in 3J
ModelCapabilities.supports_fim: bool field | VERIFY | context_budget.rs:239-245 ModelFeatures has no supports_fim; ModelCapabilityProfile has no supports_fim -- this is NEW work, not yet in codebase. Expected new symbol, but spec citation to 3J is wrong.
mode selection: edit_existing+single->FIM, edit_existing+multi->DIFF, create_file->STRUCTURED | GROUNDED | focus-tunnel-design.txt L47-58; post-phase4 E-3 L393-399 (adds execution_mode==CLOUD->STRUCTURED rule)
SSB action_types: generation_mode_selected, streaming_early_terminated, pipeline_pre_assembled, dependency_resolved | VERIFY | grep of ssb.rs returns ZERO hits for all 4 names -- these are net-new symbols to be created by B-28, not pre-existing. Correct as produces_symbols.
~350 tokens saved per FIM step | GROUNDED | focus-tunnel-design.txt L103-107: "~350 tokens overhead per step... FIM: 0 tokens overhead"
~90% generation time saved on early termination | GROUNDED | focus-tunnel-design.txt L213: "catches a bad generation at token 50 instead of token 500 saves ~90% of generation time"
pipeline parallelism saves 100-500ms per step (~88% pass rate) | GROUNDED | focus-tunnel-design.txt L234-240: "saves 100-500ms per step when validation passes (which is ~88% of the time)"
6,800 tokens saved / 27% of 25K budget (30-step task) | GROUNDED | focus-tunnel-design.txt L322-325: "Total: ~6,800 tokens = 27% of 25K budget"
45-65% effective budget increase (FIM+ShortCoder) | GROUNDED | focus-tunnel-design.txt L328-332: "Total effective budget increase: ~45-65% more useful context"
max 2 micro-steps per baby step | GROUNDED | post-phase4-implementation-order.txt L504: "Max 2 micro-steps per baby step (prevent infinite creation loops)"
pipeline depth 1 only, disable if pass rate <80% | GROUNDED | focus-tunnel-design.txt L241: "pipeline depth = 1"; L418-421: "If validation pass rate drops below 80%, disable pre-assembly"
cloud models do not support FIM | GROUNDED | focus-tunnel-design.txt L112-114: "Cloud models (Claude, GPT): do not support FIM natively. Use structured mode for cloud."

ACTIONS REQUIRED
- MISMATCH: WIRING_CONTRACT "consumes_symbols: rules deprecated_patterns (3F)" -- phase-3F.txt has no deprecated_patterns. Correct citation to: anti-staleness-enforcement.txt + plugins/crates (dep_audit.rs, import_extractor.rs). Update B-28 WIRING_CONTRACT before TESTING_FIRST.
- MISMATCH: WIRING_CONTRACT "consumes_symbols: sidecar with FIM token support (3J)" -- phase-3J has zero FIM content and targets rejected Gemma4 hardware. FIM for providers/sidecar is entirely new work. Remove the "(3J)" spec citation; this is new work B-28 itself must deliver. Update B-28 before TESTING_FIRST.
- UNGROUNDED: deprecated_patterns attribution to "3F" -- flag in B-28 WIRING_CONTRACT as UNGROUNDED or correct to "anti-staleness-enforcement.txt + plugins crate".
- VERIFY before TESTING_FIRST: ModelCapabilities.supports_fim does not exist in providers/orchestrator crates yet. Confirm that B-28 E-1 task (task 1) is treated as net-new symbol creation, not a modification of an existing struct. Currently ModelFeatures (context_budget.rs:239) and ModelCapabilityProfile (context_budget.rs:263) both lack supports_fim.

VALIDATION WARNINGS
B-28 Reads section cites focus-tunnel-design.txt and post-phase4-implementation-order.txt -- both confirmed present and on-topic.
phase-3J.txt was too large (38K tokens) for full read; searched via grep for all relevant terms (FIM, fim_prefix, supports_fim, GenerationMode) -- all returned zero matches. Classification is confident.
B-28 has no "Spec references:" lines in the standard format -- it uses a "## Reads" section instead. This is an acceptable variation but differs from the canonical format; spec_link_check.py may not detect it.
SSB schema_version is 23+ per current codebase (ssb.rs bump log); B-28 adds 4 new action_types which will require a schema version bump. Executor must increment schema_version in ssb.rs.
