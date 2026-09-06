# SHARD Prior-Art Dossiers — Index

These are grounded prior-art dossiers on the systems that compete with or outclass the SHARD paper on a per-pillar basis, plus Springdrift, the composed persistent-agent runtime that occupies SHARD's exact niche. Each dossier maps a real, shipping (or published) system onto one of SHARD's four pillars — memory governance, multi-agent coordination, skill governance, and safe self-improvement — and documents its paper, its repository, its headline measured numbers, and the gaps that distinguish it from SHARD. Every dossier was adversarially grounding-checked: an independent verification pass re-confirmed each load-bearing claim (arXiv ids, repo metadata, source-code signatures, and headline benchmarks) against primary sources, flagged any contradicted or untraceable number, and recorded a grounding verdict plus a hallucination-risk rating. Treat the table's verdict column as the trust signal: one dossier (Darwin Godel Machine) carries a verify.grounded=false flag for a single transposed benchmark number, detailed under Verification caveats.

## Comparison table

| System | Pillar | Maturity | Headline measured number | Paper verified | Repo verified | Grounding verdict | File |
|---|---|---|---|---|---|---|---|
| Letta (formerly MemGPT) | memory governance | GitHub 23,064 stars / 2,457 forks; PyPI letta 0.16.8 (2026-05-14), Apache-2.0 | DMR accuracy: GPT-4+MemGPT 92.5%, GPT-4-Turbo 93.4%, GPT-3.5-Turbo 66.9% (vendor-reported) | yes | yes | grounded, low risk | 01-letta-memgpt.md |
| Mem0 | memory governance | GitHub 57,240 stars; PyPI mem0ai 2.0.4 / npm 3.0.5, Apache-2.0; $24M funding | Paper LOCOMO LLM-as-Judge: 66.9% vs OpenAI Memory 52.9% (+26% relative) | yes | yes | grounded, low risk | 02-mem0.md |
| Zep / Graphiti | memory governance (temporal knowledge graph) | Graphiti 26,820 stars, Apache-2.0; graphiti-core 0.29.1 (2026-05-21) | DMR 94.8% vs MemGPT 93.4% (gpt-4-turbo) | yes | yes | grounded, low risk | 03-zep-graphiti.md |
| Semantic Consensus Framework | multi-agent coordination / semantic conflict detection | repo: 0 stars/forks, ~4200 LOC Python, no license, empty tests/; arXiv:2604.16339 | 100% workflow completion vs 25.1% next-best baseline (SIMULATED, not live LLMs) | yes | yes | grounded, low risk | 04-semantic-consensus.md |
| Voyager | skill acquisition / skill library | GitHub 6,943 stars, 673 forks; last code push 2024-04-03 | 3.3x more unique items (63 absolute) | yes | yes | grounded, low risk | 05-voyager.md |
| Darwin Godel Machine (DGM) | safe self-improvement | 2,076 stars / 417 forks / 21 open issues, Apache-2.0, last push 2025-08-13 | SWE-bench 20.0% -> 50.0% (Claude 3.5 Sonnet New eval, vendor-reported) | yes | yes | grounded after fix (1 transposed number corrected), low risk | 06-darwin-godel-machine.md |
| Springdrift | composed persistent-agent runtime (SHARD-equivalent niche) | 45 stars, AGPL-3.0, v0.8.0, ~136-150 Gleam files; created 2026-02-15, pushed 2026-04-27 | CBR retrieval P@4=0.956 (95% CI [0.936,0.974]) vs RAG 0.920 (synthetic, CIs overlap) | yes | yes | grounded, low risk | 07-springdrift.md |

## Synthesis: mapping onto SHARD's pillars

For SHARD's **memory governance** pillar, three production systems already cover the ground SHARD claims — Letta/MemGPT (OS-style tiered virtual context with LLM self-editing), Mem0 (two-phase extract/consolidate with semantic ADD/UPDATE/DELETE/NOOP), and Zep/Graphiti (bi-temporal knowledge-graph memory with contradiction-driven invalidation); all three are battle-tested with real stars, packages, and published benchmarks, though none implement SHARD's specific staleness/trust-tier/rollback governance, which remains SHARD's only defensible differentiator in this pillar. For **multi-agent coordination**, the Semantic Consensus Framework is the closest academic peer (rule-based semantic intent-conflict detection), but its benchmarks are simulated rather than run on live LLMs and its repo is unproven (0 stars, no license), so it is weaker than its own headline numbers suggest. For **skill governance/acquisition**, Voyager is the canonical reference (critic-gated, embedding-retrieved executable skill library) and outclasses any greenfield skill store on maturity and measured tech-tree results. For **safe self-improvement**, Darwin Godel Machine is the empirically strongest peer (archive-evolved agents that rewrite their own code, 20->50% SWE-bench), and on the **composition** axis Springdrift is the direct SHARD-equivalent — a composed persistent-agent runtime fusing CBR memory, proven-complete normative safety gates, and git-backed audit. Practical guidance: reach for Mem0 or Zep/Graphiti instead of building SHARD's memory layer, Voyager's skill-library pattern instead of a bespoke skill store, DGM for self-improvement, and study Springdrift before claiming SHARD's composed-runtime niche is novel.

## Verification caveats

- **06-darwin-godel-machine.md** (CORRECTED post-verification): the adversarial pass caught one contradicted benchmark number — the cross-model transfer base score for Claude 3.7 Sonnet was written as "23.0% base", but the cited primary source (arXiv:2505.22954 HTML v3) reports it as **19.0%** (the dossier had transposed the o3-mini base value onto the Claude 3.7 line). This was fixed in-place on 2026-06-01: line 165 now reads "vs 19.0% base". The DGM-side 59.5% and o3-mini 33.0%/23.0% figures were already correct. All other claims (paper id/title/authors/dates, repo metadata, file tree, verbatim source-code formulas, headline benchmarks, affiliations, reward-hacking findings) re-verified clean.

All other dossiers passed with verify.grounded=true and hallucinationRisk=low.

## Files

- /workspace/ELAI/workflow/research/shard-prior-art/01-letta-memgpt.md
- /workspace/ELAI/workflow/research/shard-prior-art/02-mem0.md
- /workspace/ELAI/workflow/research/shard-prior-art/03-zep-graphiti.md
- /workspace/ELAI/workflow/research/shard-prior-art/04-semantic-consensus.md
- /workspace/ELAI/workflow/research/shard-prior-art/05-voyager.md
- /workspace/ELAI/workflow/research/shard-prior-art/06-darwin-godel-machine.md
- /workspace/ELAI/workflow/research/shard-prior-art/07-springdrift.md
