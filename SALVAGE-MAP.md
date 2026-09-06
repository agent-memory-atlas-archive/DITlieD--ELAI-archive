# Salvage map

Reading suggestions for an abandoned experiment. **Source present** means the
file contains implementation, not that it builds, works end to end, or is safe to
extract. None of the product paths below was executed during archive preparation.
Reuse suggestions are judgments about where to investigate, not endorsements.

| Area | Concrete starting points | What is present / limitation | Possible reuse |
| --- | --- | --- | --- |
| Hybrid code retrieval | [Query pipeline](.elai_cc/crates/knowledge/src/pipeline.rs), [graph](.elai_cc/crates/knowledge/src/dkg/), [PageRank maps](.elai_cc/crates/knowledge/src/pagerank/) | QueryPipeline combines retrieval stages and accepts vector, graph and reranker components. Availability depends on configuration and external model/index setup. | Study or extract one retrieval stage and measure it on your own repository. |
| Evidence and completion gates | [Evidence gates](.elai_cc/crates/orchestrator/src/evidence.rs), [completion gate](.elai_cc/crates/orchestrator/src/completion_gate.rs), [task runner](.elai_cc/crates/orchestrator/src/task_runner.rs) | Typed evidence checks exist. The runner can downgrade an agent's Passed result when registered tasks remain open; a kill switch bypasses that gate. Caller-supplied evidence is not automatically independent evidence. | Small completion invariants and rejection tests, with a trusted evidence producer. |
| Shared journal | [SSB interface](.elai_cc/crates/orchestrator/src/ssb.rs), [store](.elai_cc/crates/orchestrator/src/ssb/store.rs) | Session-bound SQLite state board with serialized writes and an append-only design. Payloads and schema are heavily coupled to ELAI. | Journal architecture and event-schema lessons; start smaller than this event catalog. |
| Replay requests | [Replay validator](.elai_cc/crates/orchestrator/src/observability/replay.rs), [Tauri command](.elai_cc/crates/dashboard-v2/src-tauri/src/commands_e29.rs) | Validation and event emission exist. The source explicitly leaves model KV capture, clock/RNG interception and cross-runtime translation outside that slice. | Reference for validating checkpoint patches; it is not a complete deterministic replay engine. |
| Worker control | [Worker boot registry](.elai_cc/crates/orchestrator/src/worker_boot.rs), [task runner](.elai_cc/crates/orchestrator/src/task_runner.rs) | Startup states, trust handshakes and prompt-delivery recovery are implemented in a registry. A registry alone does not prove OS isolation or reliable transport. | A bounded worker state machine and explicit startup/failure states. |
| Context budgets | [Budget allocation](.elai_cc/crates/orchestrator/src/context_budget.rs), [projection](.elai_cc/crates/orchestrator/src/context_projection.rs) | Slot floors, output reserves and context projections have source implementations; historical allocation choices are not universal defaults. | Explicit budget accounting and protected output space, measured against actual model usage. |
| GRC reasoning | [Reasoning crate](.elai_cc/crates/reasoning/src/), [PipelineGrc adapter](.elai_cc/crates/orchestrator/src/grc.rs), [runner](.elai_cc/crates/orchestrator/src/task_runner.rs) | More than a plan: a reasoning engine and runner integration exist. Optional configuration and a disable switch affect use; this does not establish reasoning quality. | Typed claims, budgets and experimental protocols, alongside the [benchmark caveats](BENCHMARKS.md). |
| Permissions and provenance | [Capability gate](.elai_cc/crates/safety/src/capability_gate.rs), [sandbox policy](.elai_cc/crates/orchestrator/src/sandbox_policy.rs) | Provenance checks and policy code exist. The capability module describes a limited check, not full information-flow enforcement. No security certification. | Study explicit trust boundaries and refusal cases before designing your own threat model. |
| Desktop UI | [Svelte/Tauri dashboard](.elai_cc/crates/dashboard-v2/) | Experimental UI code and backend commands exist; the author worked headlessly and did not prioritize polish. | UI and command-boundary references if relevant; extraction cost is unknown. |
| Planning and failed directions | [Original plans](docs/thefinalglance/), [research decisions](workflow/reasoning/), [catalog](docs/FEATURE-CATALOG.md) | Historical plans, disagreements, audit notes and abandoned approaches. Status labels are claims made at the time. | Design references and negative lessons, without inheriting the whole workflow. |
| Development automation | [Conductor guide](CONDUCTOR.md), [selected source](conductor/) | External tooling that coordinated work on ELAI: plan selection, workers, evidence, fresh verification and serialized integration. It has omitted installation dependencies and retained historical test failures; see its [verification record](conductor/TESTING.md). | Study one control behavior, such as scope checking or durable pending landings, with its tests. |

## A small extraction process

1. Pick one behavior you need, rather than an entire plan family.
2. Read the source and its callers, then the relevant tests and recorded results.
3. Identify dependencies, placeholders, feature flags and omitted data before running it.
4. Define your own acceptance check and validate in an isolated project.
5. Preserve the applicable [licenses and notices](THIRD_PARTY_NOTICES.md).

[Short introduction](README.md) · [Postmortem](POSTMORTEM.md) · [Benchmarks](BENCHMARKS.md)
