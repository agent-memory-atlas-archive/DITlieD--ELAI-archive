<div align="center">

```
                ███████╗██╗      █████╗ ██╗
                ██╔════╝██║     ██╔══██╗██║
                █████╗  ██║     ███████║██║
                ██╔══╝  ██║     ██╔══██╗██║
                ███████╗███████╗██║  ██║██║
                ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝
```

<br>

<img src="https://img.shields.io/badge/lang-Rust-1B7A6E?style=flat-square" alt="Rust"> <img src="https://img.shields.io/badge/status-Phase_1A-1B7A6E?style=flat-square" alt="Phase 1A"> <img src="https://img.shields.io/badge/license-open--source-1B7A6E?style=flat-square" alt="License">

---

**Rust-native LLM development engine.**<br>
**Not an assistant. An autonomous workflow system.**

---

</div>

<br>

## What is ELAI

ELAI is a local-first development engine that replaces prompt-driven AI assistants with deterministic, code-driven orchestration. You give it a task. It researches, plans, writes tests, implements, audits, and learns from the outcome — all through a verifiable state machine, not prompt spaghetti.

**Core idea:** models are execution units, not decision-makers. Workflow logic lives in Rust. Every output is verified by deterministic scripts. Every reasoning step cites its grounding source. Every state transition requires evidence artifacts, not self-assessment.

<br>

## Architecture

```
    human input
         │
    ┌────▼────┐
    │ PLANNING │──── Research Agent (tree-structured, grounded)
    └────┬────┘
         │
    ┌────▼──────────────┐
    │ FEASIBILITY_PROBE  │──── hardware probe + stub sandbox
    └────┬──────────────┘
         │
    ┌────▼──────────┐
    │ TESTING_FIRST  │──── write tests that fail (TDD)
    └────┬──────────┘
         │
    ┌────▼───────┐
    │ EXECUTING   │──── implement until tests pass
    └────┬───────┘
         │
    ┌────▼────────┐
    │ AUDITING     │──── adversarial review + validation
    └────┬────────┘
         │
    ┌────▼──────────┐
    │ MEMORIZING     │──── update knowledge graph + changelog
    └────┬──────────┘
         │
    ┌────▼────────┐
    │ COMPLETE     │──── next phase or idle
    └─────────────┘
```

<br>

## Properties

- **Model-agnostic** — cloud + local (Gemma 4, DeepSeek-R1, Qwen, any GGUF)
- **Deterministic-first** — all outputs verified by scripts, not vibes
- **Entropy-quarantined** — all time/randomness through injectable traits; same seed = same output
- **Per-project** — state, indexes, graphs, RAG all live inside the project directory
- **Token-efficient** — msgpack storage, postcard wire, ~330 tokens of meta-instructions per turn
- **Interactive-first** — human-in-the-loop validated end-to-end before autonomous mode
- **Self-improving** — rules and configs evolve from pipeline outcome data

<br>

## Engine

9 crates. 61K LOC base. Forked from a Rust port, stripped and restructured.

| Crate | Role |
|---|---|
| `core` | config, paths, entropy traits, serde |
| `providers` | LLM abstraction, model lifecycle, VRAM scheduling |
| `knowledge` | DKG, BM25, dual-vector, SPLADE, PageRank, CatRAG |
| `reasoning` | grounded reasoning chains, verification, counterfactuals |
| `orchestrator` | FSM, agents, SSB, BSP supersteps, sessions |
| `tools` | file ops, git, validation, DAIRA trace, edit matching |
| `plugins` | hooks, tiered static analysis, MCP bridge |
| `rules` | predicate DSL, self-discovery, phase-aware enforcement |
| `cli` | thin REPL shell, slash routing |

<br>

## Hardware

| Phase | GPU | Model |
|---|---|---|
| Dev (Phase 0) | 1x RTX 3060 12GB | Gemma 4 E2B (2.3B effective) |
| Production | 2x RTX 3090 24GB | Gemma 4 31B + DeepSeek-R1 32B |

Sequential model loading. Embedding model on secondary GPU. 15-30s swap at phase boundaries.

<br>

## Status

Phase 1A — knowledge engine, DKG layer (tree-sitter parse into SQLite graph).

<br>

<div align="center">

---

<sub>built for one user, then the world</sub>

</div>
