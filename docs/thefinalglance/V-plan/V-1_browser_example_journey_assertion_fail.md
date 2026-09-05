# V-1: example_journey synthetic browser run fails one assertion on current build
Objective: OPEN: from /home/user/teikoku/elai, literal `browser run example_journey` (via Ougi elai_qa_shim; product requires --journey under the hood) exits nonzero with failed=1 and toolchain_missing=null. Scope-lean CLI uses synthetic-fixture snapshot; has_selector CSS form input[name='email'] is not a substring of synthetic HTML (attributes are name='email'). Fix fixture assertions to match synthetic snapshot, soft-pass synthetic path, or bind live Playwright. Evidence: u4_product_example_journey.txt, u4_rehearsal.log.
Estimated effort: 1 session
Pre-requisites: none (CLI browser_e2e surface; no product prerequisite beyond current build)
Parallel-safe with: other V-* QA units; not parallel with browser_e2e handler rewrites

## Repro
1. `source /home/user/teikoku/Ougi/tools/elai_qa_env.sh`
2. `cd /home/user/teikoku/elai && $ELAI_BIN browser run example_journey` (x3)
3. Observe rc!=0, JSON failed:1, toolchain_missing:null, runner_reached

## Evidence
- /home/user/teikoku/Ougi/plans/elai-unlimited-burn/evidence/u4_product_example_journey.txt
- /home/user/teikoku/Ougi/plans/elai-unlimited-burn/evidence/u4_rehearsal.log
- /home/user/teikoku/Ougi/plans/elai-unlimited-burn/evidence/u4_completion_pack.txt

## Expected
All assertions pass on synthetic fixture, or CLI soft-passes synthetic path.

## Note
BURN-U4 does NOT edit product .elai_cc; this unit is filed for the walk to fix.
