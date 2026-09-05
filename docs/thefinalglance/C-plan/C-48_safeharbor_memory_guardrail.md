# C-48: SafeHarbor Hierarchical Memory-Augmented Guardrail

Objective: harden ELAI's memory layer against prompt-injection / context-poisoning
attacks by stratifying memory into trust tiers (T0 user-direct, T1 codebase-derived,
T2 web-sourced, T3 model-emitted) and refusing to use T2/T3 content for safety-
critical decisions. Direct application of arXiv:2605.05704 SafeHarbor pattern.

Pre-requisites: memory-writer agent SHIPPED; C-42 (zero-trust fuzzer) SHIPPED.
Parallel-safe with: C-35..C-47, C-49..C-53.

Paper grounding (arXiv:2605.05704, 2026-05-09, verified abs page 2026-05-17):
"SafeHarbor: Hierarchical Memory-Augmented Guardrail for LLM Agent Safety."
Per verifier: paper proposes tiered memory access policy that blocks
adversarially-sourced content from influencing agent control-flow decisions.

## Context

ELAI's current memory layer treats all context homogeneously. Once content
enters memory-bank/ (or RAG, or session state), it's available to any
downstream agent. This means:
- A maliciously-crafted commit message captured into changelog.md could
  influence a future planning decision.
- A web-fetched paper abstract with a prompt-injection payload could be
  retrieved by RAG and influence the auditor.
- An LLM-emitted FAKE memory ("the user said X") could persist and bias
  future sessions.

SafeHarbor tiers memory:
- **T0 user-direct**: typed in CLI, dashboard form, explicit user-marked
  memory writes. Trustworthy for safety-critical (whether to run a
  destructive command, whether to merge a branch).
- **T1 codebase-derived**: extracted from local source files, git history.
  Trustworthy for technical decisions but not safety policy.
- **T2 web-sourced**: WebFetch, paper-ingestor mode=paper. Useful for
  grounding but NOT authoritative for control-flow.
- **T3 model-emitted**: anything the LLM wrote that was captured into
  memory without human review. Lowest trust.

Safety-critical decisions consult ONLY T0+T1. Information enrichment can
consult T2+T3 with provenance labels.

## Reads

- .elai_cc/crates/memory/ (current homogeneous memory layer)
- .elai_cc/crates/agent/src/memory_writer.rs
- workflow/papers/safeharbor-interface.md (paper-ingestor, Task 1)

## Tasks

1. **Source-grounding** — paper-ingestor (mode=paper) against arXiv:2605.05704.

2. **TrustTier enum (crates/memory/src/trust.rs)** — T0_UserDirect,
   T1_CodebaseDerived, T2_WebSourced, T3_ModelEmitted. Every memory write
   tagged at write-time.

3. **MemoryWriteSchema migration** — extend MemoryRow with `trust_tier:
   TrustTier` and `provenance: Provenance` (source URL or filepath).
   SCHEMA_VERSION bump. Migration script tags pre-C-48 rows as T3 (most
   pessimistic).

4. **Safety-critical query filter** — define list of safety-critical
   query types (destructive_command_intent, merge_decision, auth_decision,
   credential_fetch). Queries of these types use `MemoryView::T0_T1_only()`,
   which transparently excludes T2/T3 rows.

5. **Provenance propagation** — every memory READ surfaces provenance to
   the consumer. Prompts to safety-critical agents include the provenance
   inline so the model sees "user_direct" vs "web_sourced" on each cited
   fact.

6. **SSB action types** — memory_tier_assigned, memory_query_filtered,
   safety_critical_blocked (when a T2/T3 row would have influenced a
   safety query — log the block).

7. **Tests first (TDD)**:
   - 4D.X.3: TrustTier round-trips through MemoryWriteSchema
   - 4D.X.4: safety-critical query filter excludes T2/T3 rows from result
   - 4D.X.5: provenance propagates to prompt assembly visibly
   - 4D.X.6: migration script tags pre-C-48 rows as T3
   - 4D.X.7: synthetic prompt-injection fixture (T2-tier row with
     malicious payload) does NOT influence safety-critical decision

8. **Contract tests** — tests/contracts/memory_trust_contract.rs.

## WIRING_CONTRACT

produces_symbols:
  - TrustTier, Provenance enums (crates/memory/src/trust.rs)
  - MemoryView trait + T0_T1_only filter
  - SSB action_types: memory_tier_assigned, memory_query_filtered,
    safety_critical_blocked
consumes_symbols:
  - existing memory layer (extends, does not replace)
  - SSB event bus
reachable_from:
  - every memory write (write-time tagging)
  - every memory read (filter applied per query type)
stub_ban:
  - migration MUST default to most-pessimistic tier (T3) for unlabeled rows
  - safety_critical_blocked SSB MUST emit when a T2/T3 row was excluded;
    silent exclusion banned
  - provenance MUST be NON-OPTIONAL in MemoryRow post-migration

## DETERMINISTIC_CHECK

```bash
test -f .elai_cc/crates/memory/src/trust.rs
rg "T0_UserDirect|T1_CodebaseDerived|T2_WebSourced|T3_ModelEmitted" .elai_cc/crates/memory/src
rg "memory_tier_assigned|memory_query_filtered|safety_critical_blocked" .elai_cc/crates/orchestrator/src
cargo test -p elai_memory -- trust 2>&1 | grep "test result.*ok"
```

## ACCEPTANCE

- [ ] paper-ingestor produces workflow/papers/safeharbor-interface.md
- [ ] TrustTier enum implemented + migrated into MemoryRow
- [ ] SCHEMA_VERSION bumped; migration tested
- [ ] Pre-C-48 rows default to T3 post-migration
- [ ] Safety-critical query types declared
- [ ] T0_T1_only filter excludes T2/T3 transparently
- [ ] Provenance propagates to prompt assembly
- [ ] All 3 SSB action types emit on relevant events
- [ ] TDD tests pass
- [ ] Synthetic prompt-injection fixture is blocked from influencing safety
      decision (4D.X.7)
- [ ] Contract tests pass

## Files

- ADD: .elai_cc/crates/memory/src/trust.rs
- ADD: .elai_cc/crates/memory/src/view.rs (MemoryView filters)
- ADD: .elai_cc/crates/memory/tests/trust_tdd.rs
- ADD: .elai_cc/crates/memory/tests/prompt_injection_fixture.rs (4D.X.7)
- ADD: .elai_cc/crates/memory/tests/contracts/memory_trust_contract.rs
- ADD: .elai_cc/scripts/migrate_memory_trust.sh
- ADD: workflow/papers/safeharbor-interface.md
- MODIFY: .elai_cc/crates/memory/src/lib.rs (MemoryRow extension)
- MODIFY: .elai_cc/crates/agent/src/memory_writer.rs (tier-tag at write)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types + SCHEMA_VERSION bump)

## Next

C-48 closure means a malicious commit message in a 3rd-party repo
checked-out by ELAI cannot influence ELAI's safety-critical decisions.
This is the only C-N in the C-35..C-59 batch with an explicit ADVERSARIAL
threat model.
