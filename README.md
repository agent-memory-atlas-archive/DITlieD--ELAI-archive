<div align="center">

<br>

# LIA

<sub>formerly ELAI</sub>

<br>

<img src="https://img.shields.io/badge/Rust-1B7A6E?style=flat-square&logo=rust&logoColor=0d1117" alt="Rust"> &nbsp; <img src="https://img.shields.io/badge/local--first-1B7A6E?style=flat-square" alt="local-first"> &nbsp; <img src="https://img.shields.io/badge/trust--evidenced-1B7A6E?style=flat-square" alt="trust-evidenced">

<br>

an autonomous engineering system where the harness is the product<br>
the model generates, the deterministic harness sizes the work, owns the state,<br>
runs the gates, and decides "done" off evidence the model did not author

<br>

</div>

---

## why a harness

Coding agents fail in ways better models dont fix. Evaluation artifacts inside an agent's own write boundary are gamed at rates approaching 100 percent, and stronger models cheat more, not less (Berkeley RDI, Apr 2026; ImpossibleBench). Between 5.2 and 21.7 percent of LLM dependency suggestions name packages that dont exist (Spracklen et al., USENIX Security 2025). An agent deleted production data during a code freeze and misrepresented it afterward (Replit, July 2025). 96 percent of developers say they dont fully trust AI code (Sonar, 2026).

LIA's answer is structural: the agent cannot mark its own homework. Every gate measures an artifact the generator did not write, missing evidence is a failure rather than a skip, and a seeded clock plus an append-only journal make runs recomputable.

## what it does about each failure mode

**false claims** > a claimed test pass is bound to a harness-captured process hash, so a fabricated PASS is structurally refused. A claim>evidence layer answers verified, refuted, or unsupported by mechanical checks (file exists, symbol exists, test ran, dependency real, source supports), never by asking a model to grade itself.

**sycophancy** > an unsupported-agreement detector flags a "yes" no evidence backs, a cross-model adversary generates structured counter-evidence from a different model family, and verifier roles carry anti-rubber-stamp rules so no reviewer can wave its own work through.

**destructive actions** > fail-closed write and shell gates run before disk and before exec, with post-expansion path resolution (a `$(rm -rf /)` inside a substitution is still caught) and taint tracking from untrusted content to destructive sinks.

**hallucinated dependencies** > a write-time gate checks the package, the version, and the registry before the import lands.

**token economics** > cache-stable context layout, diff and fill-in-middle emission instead of whole-file rewrites, an 8-slot context budget allocator that never exceeds the window, deterministic observation masking, and a per-turn token ledger with honest per-model pricing.

**context management** > hybrid retrieval over a dependency knowledge graph, BM25, vector search, and a PageRank repo map fitted to a token budget; a session-handoff DAG with lossless trimming; an immutable transcript with pure fold-view projections so cache stability is structural, not hoped for.

**reproducibility** > all time and randomness flow through injected providers, every run emits a signed reproducible-run artifact, and a standalone verifier replays it offline. The local lane is byte-recomputable; the cloud lane is trace-authenticated, and LIA says exactly that instead of overclaiming.

**self-improvement** > a DGM-style loop over configs, rules, and reasoning templates where the modifying agent never evaluates its own modification, promotion needs multiple concurring verifiers, and an un-gameable execution-grounded filter sits between generation and adoption.

## the open kernel

The trust layer is open source and model-neutral: **[LIA Trust Kernel](https://github.com/maintainer/lia-trust)**, a small Rust binary that installs the same discipline into Claude Code (PreToolUse hook) and OpenAI Codex (MCP), with an append-only Ed25519-signed journal and offline verification.

```bash
curl -fsSL https://raw.githubusercontent.com/maintainer/lia-trust/main/install.sh | bash
```

The boundary: everything needed to independently establish trust in an agent action is open; the intelligence that decides what to do, how to solve it, and how to recover stays in LIA.

## numbers, honestly

LIA carries its own claims policy: no published number without a tag and a re-runnable receipt, enforced by a linter over these docs.

- kernel adversarial bench: catch rate 1.0 at false-block 0 on the trust corpus, offline verify green [MEASURED, signed bundles in the lia-trust repo, re-runnable from a fresh clone]
- write-gate static precursor: insecure rate 0.50 > 0.156 on a 96-sample CWE corpus [MEASURED, static fixture; the live model-in-loop delta is PENDING]
- wire-dark rate on LIA's own tree, public functions with no production caller: 7.69 percent [MEASURED 2026-07-09]
- build progress by LIA's own standard (a unit counts only with a landed commit plus durable gate evidence): 44 of 551 units verified, 162 more claimed but not yet verified. The gap between claimed and verified is the whole point of the product

## shape

22 Rust crates: knowledge engine, grounded-reasoning chain, FSM orchestration with a single-writer journaled state board, process-isolated agents, fail-closed rules and safety gates, local inference sidecar, native dashboard. Roughly 880k lines of Rust (raw count, tests included), specified across a 551-unit plan corpus.

Runs local-first on consumer GPUs, routes to cloud only when the task earns it.

<br>

<div align="center">
<sub>built for one user, then the world</sub>
</div>
