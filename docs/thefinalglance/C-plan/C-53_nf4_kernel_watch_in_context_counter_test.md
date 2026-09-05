# C-53: NF4 Kernel Upstream Watch + In-Context Prompting Counter-Test

Objective: low-priority consolidated tracker for two adjacent watch-items —
(a) HuggingFace bitsandbytes upstream NF4 kernel improvements from
arXiv:2604.02556, (b) In-Context Prompting counter-test from arXiv:2604.27891
that claims orchestration is OBSOLETE for procedural tasks. Both items
deserve attention but neither warrants its own full C-N.

Pre-requisites: none (passive watch + ablation test).
Parallel-safe with: all other C-N.

## Context

### (a) NF4 kernel watch

Paper grounding: arXiv:2604.02556 (2026-04-02, verified via prior oracle):
"2.0-2.2× kernel speedup" + "up to 1.54× end-to-end improvement" via
NVIDIA-architectural shared-memory access patterns. RESEARCH-GRADE
integration (CUDA / triton kernel layer). Plug-and-play for HuggingFace
ecosystem per paper.

ELAI uses bitsandbytes NF4 in python/elai_inference/server.py. If
HuggingFace upstream adopts the paper's optimization, ELAI gets the
speedup as a transitive dependency upgrade. PASSIVE watch: subscribe to
huggingface/bitsandbytes release notes; on a major release, run benchmark
delta.

### (b) In-Context Prompting counter-test

Paper grounding: arXiv:2604.27891 (2026-05-02, forum-dump #10): claims
in-context prompting OBSOLETES agent orchestration for procedural tasks.
This is direct counter-evidence vs ELAI's whole multi-agent decomposition
approach (Phase 4A-4G).

The honest test is: take a procedural-task subset of SWE-bench, run
ELAI's full decomposition + run a single-prompt baseline; compare. The
paper's claim is the baseline wins on this class of tasks. If it does for
ELAI too, the decomposition cost on that task class isn't justified.

This is an ABLATION patch, similar in shape to C-43 (Phase 4H scope-freeze)
but at the ORCHESTRATION level.

## Tasks

### NF4 kernel watch

1. **Subscription** — add `huggingface/bitsandbytes` to
   .elai_cc/docs/upstream-watch.md as a tracked repo. Document the paper
   reference and the trigger ("major release, run benchmark delta").

2. **Benchmark harness scaffolding** — minimal `cargo bench --bench nf4_e2e`
   target that measures sidecar tok/s on a fixed prompt set. Two configs:
   current bitsandbytes version + upstream-latest. Runs only when triggered;
   not part of normal CI.

3. **Trigger criteria** — when bitsandbytes major version bumps, run the
   harness. If end-to-end speedup ≥1.2x (paper's 1.54x is upper bound; 1.2x
   is a defensible floor), bump the pinned version and document in
   changelog.

### In-Context counter-test

4. **Source-grounding** — paper-ingestor (mode=paper) against arXiv:2604.27891.
   Produces workflow/papers/in-context-prompting-counter.md.

5. **Procedural-task subset definition** — pick a 30-task subset of
   SWE-bench Verified where the task is procedural (e.g. "convert library
   from API V1 to V2 syntax, mechanical transformation"). Document at
   tests/fixtures/procedural_subset.toml.

6. **Single-prompt baseline harness** — `elai task --baseline-single-prompt`
   flag bypasses Phase 4A-4G decomposition, dispatches the task as a
   single-shot prompt-engineering call. Same model as default config.

7. **Pilot run (DEFERRED to RunPod)** — both configs against the 30-task
   subset. Compare solve-rate + cost.

8. **Decision matrix** — IF single-prompt > full-decomposition on solve-rate
   AND cost: document the finding, gate task classification (skill_selected
   from C-41) to route procedural tasks to single-prompt path. IF NOT:
   document the win for decomposition, archive the counter-evidence.

9. **SSB action types** — counter_test_run, procedural_routing_decision.

## WIRING_CONTRACT

produces_symbols:
  - .elai_cc/docs/upstream-watch.md (with bitsandbytes entry)
  - cargo bench nf4_e2e (CPU/GPU triggerable)
  - --baseline-single-prompt CLI flag
  - tests/fixtures/procedural_subset.toml
  - workflow/sessions/{id}/in-context-counter-pilot.md (artifact)
  - SSB action_types: counter_test_run, procedural_routing_decision
consumes_symbols:
  - bitsandbytes (transitive Python dep)
  - existing FSM dispatch
  - C-41 SkillRegistry (for routing flip if applicable)
reachable_from:
  - cargo bench nf4_e2e
  - elai task --baseline-single-prompt
  - elai_cc.toml [routing.procedural_single_prompt] (if flip happens)
stub_ban:
  - NF4 benchmark MUST report wall-clock end-to-end, not synthetic
    micro-benchmark
  - counter-test pilot artifact MUST include both arms (full + single) per task
  - skill routing flip MUST be config-gated; default does not auto-route
    on speculation

## DETERMINISTIC_CHECK

```bash
test -f .elai_cc/docs/upstream-watch.md
grep -q "bitsandbytes" .elai_cc/docs/upstream-watch.md
test -f .elai_cc/benches/nf4_e2e.rs
rg "baseline-single-prompt" .elai_cc/crates/cli/src/main.rs
test -f .elai_cc/crates/task_runner/tests/fixtures/procedural_subset.toml
rg "counter_test_run|procedural_routing_decision" .elai_cc/crates/orchestrator/src
```

## ACCEPTANCE

(NF4 watch — passive)
- [ ] bitsandbytes added to upstream-watch.md with trigger criteria
- [ ] cargo bench nf4_e2e compiles and runs locally
- [ ] Initial baseline (current bitsandbytes) recorded

(Counter-test — active)
- [ ] paper-ingestor produces workflow/papers/in-context-prompting-counter.md
- [ ] 30-task procedural subset defined in fixtures
- [ ] --baseline-single-prompt flag wired
- [ ] Pilot pending RunPod (DEFERRED with C-54 window)
- [ ] Decision matrix documented after pilot
- [ ] SSB action types emit on counter-test run
- [ ] Routing flip (if it happens) is config-gated, default off

## Files

- ADD: .elai_cc/docs/upstream-watch.md
- ADD: .elai_cc/benches/nf4_e2e.rs
- ADD: .elai_cc/crates/task_runner/tests/fixtures/procedural_subset.toml
- ADD: workflow/papers/in-context-prompting-counter.md
- ADD: workflow/sessions/{id}/in-context-counter-pilot.md
- MODIFY: .elai_cc/crates/cli/src/main.rs (--baseline-single-prompt flag)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (2 new action types)
- MODIFY: .elai_cc/Cargo.toml (bench target declaration)

## Next

C-53 closure produces the EVIDENCE that lets ELAI defend decomposition for
procedural tasks (or pivot for them). Both are valid outcomes. Pairs with
C-46 (Uno-Orchestra) — if procedural tasks should route single-prompt,
SelectiveDelegator's tier-0 threshold can include that path.
