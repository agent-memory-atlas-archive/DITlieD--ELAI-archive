# Conductor's ELAI development loop

Conductor coordinated work on ELAI from outside the product. ELAI's own runtime
lives in [`.elai_cc/`](../.elai_cc/); this directory preserves selected tooling
that dispatched development agents and handled their proposed changes.

**This is an unsupported source archive, not a portable automation installation.**
The snapshot includes later revisions of the tooling. It records how the machinery
was structured, not which exact version produced each historical ELAI commit.
Only isolated checks described in [TESTING.md](TESTING.md) were run for publication.

## Follow one change through the code

| Step | Implementation | Role and limits |
| --- | --- | --- |
| Select work | [plan_selector.py](opus_walk/plan_selector.py), [corpus_drain.py](opus_walk/corpus_drain.py), [quorum_triage.py](opus_walk/quorum_triage.py) | Read plan summaries/status, exclude completed or conflicting work, and optionally ask several models to choose a small candidate. Status files and model votes are inputs, not proof that the plan is correct. |
| Resume the walk | [opus_supervisor.py](opus_walk/opus_supervisor.py), [opus_walk_chain.py](opus_walk/opus_walk_chain.py), [walk_state.py](opus_walk/walk_state.py) | Maintain the cursor, dispatch budget, lock, stop conditions and restart decisions across fresh orchestrator sessions. State and account setup are installation-specific. |
| Alternate supervisors | [fallback_supervisor.py](fallback_walk/fallback_supervisor.py), [glm_supervisor.py](glm_walk/glm_supervisor.py), [controller registry](walk_controller_registry.py) | Preserve the provider-fallback and GLM variants used around the same ELAI worker/landing path. Their control wrappers and adapters are included; the surrounding interactive API is not. |
| Dispatch a worker | [opus_dispatch.py](opus_walk/opus_dispatch.py), [worker_adapter.py](worker_adapter.py) | Build the task and agent context, choose a provider lane, create or reuse a worktree, capture results, and clean up failed attempts. Worktree separation is not a complete security sandbox. |
| Account for work | [conductor_db.py](conductor_db.py), [migrations](migrations/), [leases.py](leases.py), [events.py](events.py) | Store tasks, attempts, leases, gate evidence and integration records in SQLite. Only schema migrations are included; no live database was copied. |
| Check the change | [gate_runner.py](gate_runner.py), [gate_packs.py](gate_packs.py), [ELAI checkers](../.claude/tools/) | Construct checks, capture outputs and retain verdicts. Some checks require omitted frozen manifests, project state or external tools. A gate name does not establish that it ran. |
| Review and land | [opus_preland_verify.py](opus_walk/opus_preland_verify.py), [opus_land.py](opus_walk/opus_land.py), [integrator.py](integrator.py) | Ask a fresh verifier to assess plan/diff/evidence, enforce declared scope, then serialize integration. The source contains an operator-scoped verifier-bypass sentinel; review was not mechanically impossible to bypass. |
| Recover and reclaim | [pending_lands.py](opus_walk/pending_lands.py), [pending_replay.py](opus_walk/pending_replay.py), [disk_hygiene.py](opus_walk/disk_hygiene.py) | Park unavailable-verifier cases, retry pending landings and reclaim build/worktree scratch. These routines can change files and Git state when actually invoked. |

The useful design idea is the separation of a worker's proposed change from the
software that records evidence and decides whether to integrate it. The historical
implementation has exceptions, switches and external dependencies. It should be
read with those limits in mind.

## What this snapshot includes

- The ELAI walk's local Python import dependencies, including provider command
  builders, output guards, context packets, worker slots, leases and telemetry.
- SQLite schema migrations, related fixture tests, a configuration example and
  the [historical orchestration guide](ORCHESTRATION-GUIDE.md).
- The ELAI Python gate checkers referenced by the runner, preserved in
  [`.claude/tools/`](../.claude/tools/).
- The build-offload adapter and profile validator in [tools/](tools/), plus the
  Cargo shim. Their account, repository and machine defaults were generalized.

[SNAPSHOT.json](SNAPSHOT.json) records the exact selected source/test/support files
and publication adaptations. The repository's [checksum manifest](../ARCHIVE-MANIFEST.sha256)
covers their published bytes.

## What you would have to supply

| Dependency | Boundary |
| --- | --- |
| Python and tools | Python 3.11+ is required by `tomllib`; fixture checks use pytest. Git and the chosen gate tools must be installed separately. The freeze checker also has an optional `blake3` dependency. |
| Repository paths | Absolute installation paths are example locations. The [configuration example](config/conductor.elai-walk.example.toml) must be adapted; it is deliberately not installed under the live filename expected by dispatch. |
| Agent/provider runtimes | The historical Claude/base-cc, Codex, Grok, local bridge and optional Prime Agent routes refer to separately installed programs and services. Those programs, model availability and accounts are not supplied or revalidated here. |
| Prompts and MCP configuration | The supervisor's original orchestrator-skill location, operator directive and MCP configuration belonged to the surrounding workspace. Those installation files are absent. ELAI's archived [agent definitions](../.claude/agents/) and [dispatch preamble](../.claude/rules/agent-dispatch-preamble.md) remain as references. |
| Status and state | Plan-status snapshots, pending landings, trust ledgers, checkpoints, DBs and event history are omitted. Status querying cannot reconstruct those historical measurements from plans alone. |
| Gate baselines and remote builds | Frozen gate manifests, Depot profiles/workflows, remote projects and credentials are not copied. Required checks may refuse to run without them. The offload helper is source, not access to build infrastructure. |

Provider adapters retain historical credential-loading and token-refresh code
because it is part of the dependency chain. No credential files or values were
copied. Importing source for study does not require configuring an account;
starting its live control loop is a separate undertaking.

## Publication changes and omissions

Personal names, account/repository identifiers and machine paths were replaced
with neutral examples. Two Python helper locations and the Cargo shim were
relocated to the included support directory. Historical Git attribution now uses
ELAI contributors. The original files were left intact.

Live JSON state, logs, transcripts, credentials, worktrees, account stores,
downloaded runtimes, personal knowledge/research/chat services and unrelated
project inventories are excluded. No Conductor Git history or parent-workspace
history was imported. These files enter ELAI in one archive-preparation commit.

The [historical guide](ORCHESTRATION-GUIDE.md) retains old commands and claims for
context. Those are not current instructions or a statement that the commands are
portable. Pricing labels such as “free”, model aliases, provider limits and
historical success claims may be obsolete.

For reuse, start with one behavior—scope checking, durable walk state, pending
landings, or serialized integration—and inspect its source, callers and tests.
Avoid inheriting the whole automation setup just to reuse one idea.

The selected original code and documents use the archive's [MIT license](../LICENSE).
External runtimes remain separate dependencies; see [third-party notices](../THIRD_PARTY_NOTICES.md).

[Short introduction](../CONDUCTOR.md) · [ELAI postmortem](../POSTMORTEM.md)
