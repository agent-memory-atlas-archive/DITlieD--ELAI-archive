# P1: SSB action_type codex allowed_writes incomplete

## Symptom
First E-11 ScopeCeilingAdvisory EXECUTE: `STATUS BLOCKED`, `FILES TOUCHED: NONE`, patch 0 B.  
Question: expand write scope to `ssb.rs` + `ssb/tests.rs` (and later `tests/action_type_lock.rs` for second exhaustive match).

## Evidence
- Slice evidence `opus_walk/evidence/E-11/E-11-T8-slice-plan.md` MOVE 1 eventually listed full checklist after failures:
  - action_type.rs, payloads.rs, schema.rs, **ssb.rs SSB_ACTION_TYPES**, **ssb/tests.rs**, **tests/action_type_lock.rs**, idea_framing.rs, idea_framing_unit.rs
- `opus_dispatch.dispatch_codex_executor` only enforces non-empty allowed_writes; no SSB expansion.
- Head constructs allowed_writes in the boot turn; no static helper.

## Root cause
Any new `SsbActionType` requires an **exhaustive cascade** of registry sites. Head enumerated only “obvious” producer files on first pass; compilers later named the rest. No `expand_ssb_write_set()` for prompt generation.

## Proposed fix
1. Add `opus_walk/ssb_write_set.py`:
   - `SSB_REGISTRATION_FILES = [...]` relative paths under `.elai_cc/crates/orchestrator/...`
   - `expand_allowed_writes(paths: list[str]) -> list[str]`: if any path matches ssb action_type|payloads|schema|idea_framing SSB emit, union full registration set + action_type_lock + ssb/tests.
2. Document in sequential-opus-orchestrator SKILL / boot prompt: “for any new SsbActionType, call expand (or always include the list).”
3. Unit tests for expand_allowed_writes pure function.

## Test plan
- expand(['ssb/action_type.rs']) includes ssb.rs, tests.rs, action_type_lock.rs.
- expand(['cli/main.rs']) unchanged.
- Full opus_walk suite green.

## Risk
- Slightly wider codex scope → acceptable for schema registry atomicity.
