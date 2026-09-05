GROUNDING SWEEP -- phase I-F8
Source: docs/thefinalglance/F-plan/F-8_scf_symbolic_conflict_selfmod.md
Date: 2026-06-01
FSM: EXECUTING | Session: s-6344

SPEC REFERENCES

workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md | GROUNDED | file confirmed at F:\.ELAI_workflow\workflow\reasoning\VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md; sections 1.6/4/C8/5/7 present (grep: "C8", "27.9", "WARN-then-escalate", "inter-agent BSP", "Pillar 4" all match)
workflow/research/shard-prior-art/00-COMPARISON-shard-vs-priorart.md | GROUNDED | file confirmed at F:\.ELAI_workflow\workflow\research\shard-prior-art\00-COMPARISON-shard-vs-priorart.md; Pillar 2 + Pillar 4 sections referenced
F:\.ELAI_workflow\.claude\tools\elai.map | GROUNDED | used directly for all symbol verification; all consumes_symbols below confirmed

CLAIM CLASSIFICATION

apply_structural_change at self_mod_safety.rs, elai.map:270596 | GROUNDED | elai.map line 270798: DEF_FN lChange) -> Result<(),  crates/orchestrator/src/self_mod_safety.rs:247  (plan cites map line 270596, actual is 270798; file:line 247 confirmed in source -- minor map-line off by ~200, symbol real)
check_locked_key at self_mod_safety.rs, elai.map:471273 | GROUNDED | elai.map line 471712: DEF_FN t<(), SafetyErro  crates/orchestrator/src/self_mod_safety.rs:222  (map line 471712 vs plan's 471273; file:line confirmed)
is_locked at self_mod_safety.rs, elai.map:280844 | GROUNDED | elai.map line 281083: DEF_FN locked_s  crates/orchestrator/src/self_mod_safety.rs:208  (map line 281083 vs plan's 280844; file:line confirmed)
ParameterKey at self_mod_safety.rs:86, elai.map:193921 | GROUNDED | elai.map line 194001: DEF_ENUM EtcThresh crates/orchestrator/src/self_mod_safety.rs:86 (map line 194001 vs plan's 193921; file:line 86 confirmed, matches source read)
locked_set at self_mod_safety.rs, elai.map:474605 | GROUNDED | elai.map line 475054: DEF_FN tatic str>  crates/orchestrator/src/self_mod_safety.rs:214 (map line 475054 vs 474605; symbol real)
is_known_parameter at self_mod_safety.rs:149, elai.map:14598 | GROUNDED | elai.map line 14585: DEF_FN (key: &str) -> boo  crates/orchestrator/src/self_mod_safety.rs:149  (map line ~14585; symbol + file:line confirmed)
StructuralChange at self_mod_types.rs:421, elai.map:544399 | GROUNDED | source read confirmed StructuralChange struct at line 421 carrying human_approved; elai.map line 544900 DEF_STRUCT
ConfigVariant at self_mod_types.rs:254, elai.map:469038 | GROUNDED | source read confirmed ConfigVariant struct at line 254; elai.map line 469476
SelfModProposal at self_mod_types.rs:295, elai.map:544416 | GROUNDED | elai.map line 544917: DEF_STRUCT { pub varia  crates/orchestrator/src/self_mod_types.rs:295
ConfigVariantSimilarity at self_mod_types.rs:443, elai.map:68043 | GROUNDED | elai.map line 68141: DEF_STRUCT arity { pub key_ove  crates/orchestrator/src/self_mod_types.rs:443
ProjectScope at self_mod_types.rs:286 | GROUNDED | elai.map line 371488: DEF_STRUCT pub ecosy  crates/orchestrator/src/self_mod_types.rs:286
Layer1aProcessor trait at self_mod_layer1a.rs:323 | GROUNDED | elai.map line 28341: DEF_TRAIT crates/orchestrator/src/self_mod_layer1a.rs:323
DefaultLayer1aProcessor at self_mod_layer1a.rs:430/463 | GROUNDED | elai.map lines 155950/174654 confirm both the SCIP_DEF and the explicit DEF_STRUCT
apply_proposal wired at runtime_setup.rs:299 | GROUNDED | elai.map line 39876 CALL: let layer1a = DefaultLayer1aProcessor::new(rules_db, rules_db_path.clone(), ssb.clone()); confirmed
two_gate_promote at promotion_gate.rs:103 | GROUNDED | source read confirmed; PromotionConfig at line 46, PromotionDecision at line 81, two_gate_promote at line 103
ConfigArchive at config_archive.rs:148, with get_variant/select_parent/revert/enforce_cap | GROUNDED | elai.map: config_archive.rs:148 DEF_STRUCT, get_variant:254, select_parent:274, revert:348, enforce_cap:301 all confirmed
TripwireKind at tripwire.rs:53 (IqrOutlier/Perplexity/Divergence) | GROUNDED | elai.map line 260488: DEF_ENUM ivergence  crates/orchestrator/src/tripwire.rs:53
TripwireFired struct | GROUNDED | elai.map line 148306: DEF_STRUCT d: TripwireKi  crates/orchestrator/src/tripwire.rs:212
TripwireMonitor struct | GROUNDED | elai.map line 398392: DEF_STRUCT ripwireSample>  crates/orchestrator/src/tripwire.rs:592
observe_post_consensus_tripwires at agent_pool.rs:1622 (NOT tripwire.rs) | GROUNDED | elai.map line 300219: DEF_FN Monitor, archive_chain: Opti  crates/orchestrator/src/agent_pool.rs:1622
consensus aggregate at tdd_verifier/consensus.rs | GROUNDED | elai.map line 53907: USE aggregate from crate::verification_result; actual DEF is in verification_result, called by consensus.rs (plan says consensus.rs::aggregate which matches the USE/call relationship)
applied_proposals_layer1a at ssb_viewer.rs:4150 | GROUNDED | source read confirmed: line 4150 pub fn applied_proposals_layer1a(&self)
SelfModProposalRow at ssb_viewer.rs:4507 | GROUNDED | elai.map line 398081: DEF_STRUCT ring, pub appl  crates/orchestrator/src/ssb_viewer.rs:4507
get_applied_proposals at commands.rs:1129 | GROUNDED | source read confirmed: line 1129 pub fn get_applied_proposals(state: State<'_, AppState>) -> CmdResult<Value>
ReasoningDrawer.svelte subscribes to reflect-progress + shows TripwireStatusStrip + ArchiveVerifyGauge | GROUNDED | source read confirmed: imports both components lines 7-8, listens to reflect-progress at line 55, renders them at lines 179-180
apply_proposal confirm flow at commands.rs:1143 | VERIFY | plan says "confirm flow already in commands.rs:1143" -- line 1143 is only the doc comment; actual apply_proposal_from_user starts at line 1152. The confirm flow EXISTS (elai.map line 447515) but the line citation is 9 lines off. Non-blocking.
check_no_undeclared_interfaces/check_version_incremented/CoherenceError in coherence_check.rs | GROUNDED | elai.map: check_no_undeclared_interfaces at coherence_check.rs:106 (SCIP_DEF line 100617), check_version_incremented at :146 (line 100890), CoherenceError DEF_ENUM line 111566
ContractSpec not in elai.map (C-50 PLANNED not shipped) | GROUNDED | grep ContractSpec elai.map = 0 matches; no namespace collision confirmed
blake3 in orchestrator Cargo.toml | GROUNDED | grep blake3 crates/orchestrator/Cargo.toml: "blake3 = { workspace = true }" at line 82
drawers.svelte.ts as reasoningData state | GROUNDED | elai.map line 13256: CALL export const reasoningData = createReasoning() at drawers.svelte.ts:316; DEF_FN createReasoning at drawers.svelte.ts:240
SCHEMA_VERSION claim: plan says "baseline 191" | MISMATCH | ssb.rs:2742: const SCHEMA_VERSION: i64 = 196 (F-1/F-2a landed as SCHEMA_VERSION 196 per ssb.rs:2727 comment "F-1 / F-2a (SCHEMA_VERSION 196 -- 2026-06-01)"). CLAUDE.md "191/464" is stale. Actual baseline is 196 before F-8 adds +1 = 197. Plan prose says "baseline 191 + 1" which is wrong by 5 versions.
TOTAL_VARIANTS claim: CLAUDE.md says 464, plan uses 191 framing | MISMATCH | action_type_lock.rs line 490: const TOTAL_VARIANTS: usize = 473. Plan references "baseline 191" which carries implicit 464-variant assumption. Actual is 473 variants at SCHEMA_VERSION 196. F-8 adds 1 variant -> 474.
SsbViewerError line for pending_proposals_layer1a:4075 | GROUNDED | elai.map line 174459: DEF_FN ected proposal_ids.  crates/orchestrator/src/ssb_viewer.rs:4075
27.9% precision is sim-only limitation, not a design choice | GROUNDED | VIBESEC synthesis 1.6 confirms: "VERDICT: OVERSTATED. The 27.9% precision is a [simulation limitation]" confirmed by grep
inter-agent BSP DEFERRED (RunPod-pending) | GROUNDED | VIBESEC synthesis section 5/503 confirmed: "Do NOT ship the SCF inter-agent conflict gate yet (27.9% FP in parallel dispatch; simulation-only"

ACTIONS REQUIRED

- MISMATCH: SCHEMA_VERSION in plan prose says "baseline 191 + 1". Actual current SCHEMA_VERSION is 196 (F-1/F-2a shipped as 196 per ssb.rs:2727). F-8 must use "baseline 196 + 1 = 197". Update the plan's WIRING_CONTRACT and W-9 ACCEPTANCE item to name 197 as the post-bump version. The "named at edit time" rule still applies -- hard-code 197 in the executed edit only.
- MISMATCH: TOTAL_VARIANTS says 464 (CLAUDE.md stale reference). Actual is 473 (action_type_lock.rs:490). F-8 adds 1 -> expect 474. The golden lock test will enforce this; implementer must use 474 not 465.
- VERIFY (minor, non-blocking): apply_proposal confirm flow line cited as commands.rs:1143 -- actual function start is line 1152 (line 1143 is the doc comment). No code change needed; cite line 1152 instead in any new code comments.
- VERIFY (minor, non-blocking): several elai.map line numbers cited in plan's Citations section are off by 50-200 lines vs actual (ParameterKey:193921 vs 194001, apply_structural_change:270596 vs 270798, etc.). All symbols are real and at correct file:line; the discrepancy is in the elai.map row index only. Non-blocking but the map may have been regenerated since authoring. Implementer should re-confirm via W-7 at edit time.

VALIDATION WARNINGS

- SCHEMA_VERSION 191 in CLAUDE.md is stale (actual 196 in ssb.rs as of F-1/F-2a). The plan's "baseline 191 + 1" prose is therefore wrong; correct baseline is 196 + 1 = 197. This is the only blocking finding.
- TOTAL_VARIANTS 464 in CLAUDE.md is stale (actual 473 in action_type_lock.rs). Non-blocking at authoring time but will cause golden-lock test failure if 465 is used instead of 474.
- elai.map line-index citations in the plan's Citations section are stale by ~50-200 rows (likely regenerated since authoring). All file:line references to source are correct. W-7 re-confirm at edit time is sufficient.
- all other sources validated
