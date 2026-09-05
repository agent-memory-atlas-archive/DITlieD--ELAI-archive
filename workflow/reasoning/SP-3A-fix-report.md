FIX REPORT — phase SP-3A (fixer, task_id=SP-3A-fix, session s-finish-0705)
Source: SP-3A audit BLOCK (C-MAINTAINABILITY-FILESIZE, dispatch prompt)
Date: 2026-07-06
Note: fixer.md destination workflow/sessions/s-finish-0705/fix-report.md rejected by EXECUTING-state path allowlist; written here (allowed path) instead.

FIXES APPLIED
C-MAINTAINABILITY-FILESIZE | crates/orchestrator/src/adversary.rs (was 1004 lines) | Extracted ModelFamily enum + impl (old lines 50-212) verbatim into new crates/orchestrator/src/model_family.rs (174 lines) with its two imports (serde Serialize/Deserialize, elai_providers::ProviderKind); replaced the block in adversary.rs with `pub use crate::model_family::ModelFamily;` re-export mirroring the cloud_adversary P0-1 precedent; registered `pub mod model_family;` in lib.rs beside cloud_adversary. adversary.rs now 849 lines. | Evidence: MR-1 boundary was 1004 > 1000 (audit BLOCK); post-fix maintainability_check on .elai_cc/-prefixed paths reports blocks=0, no MR-1 finding, exit 0.
FOLLOW-ON (same extraction, import bookkeeping) | crates/orchestrator/src/adversary.rs | cargo check flagged `use elai_providers::ProviderKind;` unused in non-test builds after the block moved (only remaining users are the mod tests from_provider_kind tests via super::*). Relocated the import into `mod tests`. | Evidence: warning in cargo check output (task bgaud4976); cargo test -p elai-orchestrator --lib adversary compiles the test target clean, 41 passed / 0 failed.

DEFERRED
NONE

NEW BLOCKS (if my fix introduced any)
NONE observed. Pre-existing warning `unexpected cfg condition value: vllm` moved with the block (now fires at model_family.rs:97; same construct previously fired inside adversary.rs). Relocated verbatim per dispatch, not introduced.

STUCK (if anti-spiral triggered)
NONE

TOOL DEFECT SURFACED (for orchestrator; gate is frozen to me, not fixed)
maintainability_check.py fail-opens on nonexistent input paths: the dispatch's verbatim unprefixed paths (crates/orchestrator/...) run from F:\.ELAI_workflow scanned vacuously (files_scanned=2, clean, exit 0), and a deliberately fake path also returns clean exit 0. Real verdict obtained with .elai_cc/-prefixed paths: blocks=0, warns=2 (MR-3 on generate_counter / generate_counter_dyn reported as "582/569 lines" — regex brace-matcher mis-measure of bodyless async-fn-in-trait signatures in pub trait Adversary; constructs untouched by this diff). This contradicts the MR rule's "exit 2 fail-closed on scan-config error" contract; auditor should re-run with the .elai_cc/ prefix and the checker owner should fix missing-path handling.

TEST COMMANDS THE AUDITOR SHOULD RE-RUN
- cargo check --workspace  (from F:\.ELAI_workflow\.elai_cc; my run: exit 0, finished 6m17s)
- cargo test -p elai-orchestrator --lib adversary  (41 passed / 0 failed)
- cargo test -p elai_cc_contracts --test adversary_contract  (14 passed / 0 failed, matches pre-fix 14/14)
- python .claude/tools/maintainability_check.py --files .elai_cc/crates/orchestrator/src/adversary.rs,.elai_cc/crates/orchestrator/src/model_family.rs --base HEAD --json  (from F:\.ELAI_workflow; the .elai_cc/ prefix is REQUIRED — unprefixed paths scan vacuously)
