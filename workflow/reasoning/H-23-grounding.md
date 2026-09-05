GROUNDING SWEEP -- phase H-23
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-23_same_seed_rerun_hash_equality.md
Date: 2026-06-05
FSM: PLANNING | Session: s-8404

SPEC REFERENCES
_grounding/H-TRUST-GROUNDING-MAP.md:TS-5,NN-7,TD-5 | GROUNDED | file exists; TS-5 at line 70 ("C-27+G-2 SHIPPED"); NN-7 at line 116 ("BUILD on RRA/replay"); TD-5 at line 149 ("HONEST-RESIDUE...cross-hardware FP"); H-23 symbol table at line 216
rra.rs:200 RunArtifact | GROUNDED | struct RunArtifact at rra.rs:200; seed/hardware_class/system_fingerprint_pin/events/manifest/root_blake3 fields confirmed in rra.rs:200-224
args.rs:2615 emit_rra | VERIFY_NEEDED | --emit-rra flag exists (args.rs:2669-2672 and struct field at args.rs:227); line number 2615 is off by ~54 lines but symbol exists and is correctly described
run_subcommand.rs:584 replay verify reading system_fingerprint_pin | GROUNDED | run_subcommand.rs:584 reads artifact.system_fingerprint_pin.as_deref(); emit_rra at :60/450; replay path at :576-590 confirmed
ssb.rs:20060 RunArtifactEmittedPayload | VERIFY_NEEDED | struct exists at ssb.rs:20461 (not 20060 as cited); line number off by ~401 but struct content confirmed at 20461-20469
ssb.rs:23502 replay_resume | GROUNDED | replay_resume appears in ssb.rs comments at lines 2084/4449/7710/13750; SsbReader::replay_resume fn at ssb.rs:23903
determinism.rs:317 DriftComparator | GROUNDED | DriftComparator struct at determinism.rs:317 exactly; compare() method; first-divergence logic at :368-408; DeterminismDriftPayload at :398
determinism.rs:175 system_fingerprint_pin from_env | GROUNDED | determinism.rs:175 reads ELAI_SYSTEM_FINGERPRINT_ENV exact match confirmed
predicate.rs:102 AstPattern | GROUNDED | predicate.rs:102 exact match; SchemaRequired at :121 exact match

CLAIM CLASSIFICATION
RunArtifact bundles ELAI_SEED + HardwareClass + system_fingerprint_pin | GROUNDED | rra.rs:200-224: seed/hardware_class/system_fingerprint_pin fields confirmed
elai replay --verify re-runs action stream reports PASS or DIVERGENCE | GROUNDED | run_subcommand.rs:576-590; ReplayVerdict::Verified/Divergence/Tampered at rra.rs:226-250
DriftComparator reports first-divergence-index | GROUNDED | determinism.rs:368-408; first_divergence_index field in DeterminismDriftPayload at :398-404
blake3 as the metric-output digest hash | GROUNDED | CLAUDE.md HARD RULE (blake3 never blake2/sha2); rra.rs:189-196 uses blake3; determinism.rs:269 uses blake3::hash; all existing hashes in codebase are blake3
verify_rerun_digest + RerunHashEquality are ABSENT (NN-7) | GROUNDED | grep over orchestrator/src finds no matches; correctly flagged as the pure-BUILD surface
AstPattern + SchemaRequired exist in predicate.rs | GROUNDED | predicate.rs:102/121 exact line match
Pattern-2 SSB variant baseline+1 from 207/484; re-read e45:294 | VERIFY | e45:294 currently asserts TOTAL_VARIANTS=504 (not 484); plan correctly instructs re-read + bump; executor must read live value before bumping
ML reproducibility crisis (motivation claim) | UNGROUNDED | no citation, no arXiv id, no paper title; widely known phenomenon but not cited in plan; motivation-only, not a mechanism claim -- does not block EXECUTING
Thinking Machines batch-invariant kernels 2025 (future cross-hardware path) | UNGROUNDED | no arXiv id, no URL; no RAG hit; plan labels it "documented, not assumed" but provides no grounding source; TD-5 correctly scopes H-23 to NOT assert this -- UNGROUNDED label on the future-path reference only, does not block current gate
PYTHONHASHSEED/CUBLAS_WORKSPACE_CONFIG as determinism env vars | VERIFY | widely documented (PyTorch reproducibility docs); plan instructs "WebSearch the determinism env-var set before the AST gate" -- executor must verify complete env-var list before authoring seed_coverage.toml
same-env/same-seed FP bitwise equality provable; cross-hardware is physics | GROUNDED | TD-5 in grounding map line 149; rra.rs GuaranteeLevel reflects this; plan scope correctly bounded
blake3 prefix-only (16-hex truncated) vs full hash in existing code | GROUNDED | rra.rs:195 truncate_hex() on finalize(); DriftComparator uses full hash (determinism.rs:269); metric-output digest will need to specify which -- executor decision

ACTIONS REQUIRED
- VERIFY claim: TOTAL_VARIANTS baseline is currently 504 in e45:294, not 484 as plan implies; executor must read action_type_lock.rs live value before bumping to baseline+1 (i.e. 505)
- VERIFY claim: confirm complete Python determinism env-var set (PYTHONHASHSEED, CUBLAS_WORKSPACE_CONFIG, and any others) via WebSearch or PyTorch docs before authoring seed_coverage.toml -- plan correctly flags this as a pre-coding WebSearch
- UNGROUNDED claim "Thinking Machines batch-invariant kernels 2025" -- no arXiv id or URL; add citation in H-23-decisions.md or label "documented-future-path (ungrounded)" and defer; does not block EXECUTING because TD-5 correctly scopes H-23 to NOT assert cross-hardware equality
- UNGROUNDED claim "ML reproducibility crisis" -- motivation-only, no citation needed to block EXECUTING; optionally add arXiv:2206.01677 or similar if the decisions doc requires grounding

VALIDATION WARNINGS
- Two spec line numbers are imprecise: args.rs:2615 (actual ~2669) and ssb.rs:20060 (actual 20461); symbols exist at those files but line offsets are stale -- low risk, both VERIFY_NEEDED on precision only
- No index file cited directly; all references are to per-file line ranges -- sources validated
- E45 contract baseline at line 294 shows TOTAL_VARIANTS=504 not 484; plan acknowledges re-read needed (correct); not a MISMATCH, a stale snapshot in plan prose
