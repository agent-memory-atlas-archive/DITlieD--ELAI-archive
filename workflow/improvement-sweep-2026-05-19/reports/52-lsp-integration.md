# LSP integration for ELAI (2024-2026)

## TL;DR
- Add LSP as an OPT-IN precision oracle, not a default layer. Tree-sitter + SCIP (already planned) covers ~80% of structural needs; LSP earns its keep on type-narrowing, diagnostics, and "would this compile" probes where tree-sitter is provably blind.
- Highest-ROI hook: rust-analyzer/pyright/ts-server `getDiagnostics` as a pre-AUDITING gate after every Edit/Write. Anthropic shipped exactly this in Claude Code 2.0.74 (Dec 2025) and reports it cuts the propagate-bad-edit failure mode.
- Use rust-analyzer in CLI/headless mode (`analysis-stats`, `scip`, `lsif`, `ssr`) for batch indexing rather than running a persistent LSP daemon — same semantic info, no resident RAM cost.
- Wrap the multi-language story with microsoft/multilspy (Python LSP client, 12 servers, hand-tuned configs) instead of writing our own JSON-RPC wrapper. Production-tested at MSR for Monitor-Guided Decoding.
- The Preprints 202510.0924 caveat is real but narrower than the report-40 framing suggested: LSP underperforms when used as a *retrieval* mechanism competing with RAG/lexical for in-context examples; it dominates when used as a *verifier* (diagnostics, type-check probe) on candidate edits.

## Key sources
| URL | What | Takeaway |
|---|---|---|
| https://www.preprints.org/manuscript/202510.0924 | Code Retrieval in Coding Agents study (Oct 2025) | LSP as one of several retrieval techniques; structured nav (def/refs/symbol) competes with lexical+semantic RAG; verdict context-dependent, not a blanket "LSP loses" |
| https://arxiv.org/pdf/2510.22907 | Lanser-CLI / RL from Compiler+LSP feedback (Oct 2025) | LSP signals as *process rewards* for agent loops — diagnostics give step-wise machine-checked reward, deterministic+replayable; positions LSP as verifier, not retriever |
| https://arxiv.org/pdf/2306.10763 | Monitor-Guided Decoding (Agrawal et al, MSR) | LSP completion list constrains decoding; SantaCoder-1.1B+MGD beats text-davinci-003 on next-identifier match and compilation rate. Foundational paper for LSP-as-oracle |
| https://www.amazingcto.com/lsp-in-claude/ | Claude Code LSP integration writeup | Claims diagnostics-after-edit is the single highest-value LSP signal; warns pyright/rust-analyzer/jdtls are RAM-heavy on large projects |
| https://news.ycombinator.com/item?id=46355165 | HN thread on native Claude Code LSP support | Anthropic added LSP in 2.0.74 (Dec 2025), 11 languages; goto-def, find-refs, diagnostics |
| https://github.com/microsoft/multilspy | MSR multilspy library | Python LSP client, 12 servers, used for MGD; handles binary download, JSON-RPC, hand-tuned configs |
| https://deepwiki.com/rust-lang/rust-analyzer/7.2-command-line-interface | rust-analyzer CLI reference | `analysis-stats`, `lsif`, `scip`, `ssr` subcommands work without a running LSP daemon |
| https://deepwiki.com/rust-lang/rust-analyzer/9.2-scip-and-lsif-indexing | rust-analyzer SCIP/LSIF | Batch-emit SCIP/LSIF protobuf; same semantic data as live LSP, no resident process |
| https://sourcegraph.com/blog/announcing-scip | SCIP announcement | SCIP replaces LSIF: protobuf vs JSON, ~4x smaller compressed, ~10x faster CI indexing (scip-typescript replaces lsif-node) |
| https://aider.chat/2023/10/22/repomap.html | Aider repo map design | Tree-sitter chosen over LSP explicitly: "LSP more cumbersome to deploy for a broad array of languages" |
| https://deepwiki.com/continuedev/continue/6.6-lsp-context-integration | Continue.dev LSP context | LSP plugin into completion/next-edit; queries goto-def + AST crawl recursively to assemble context |
| https://arxiv.org/pdf/2508.03830 | If-T benchmark (Aug 2025) | Type narrowing benchmark across TS/Flow/Typed Racket/mypy/Pyright; shows narrowing systems differ on logical-implication tracking — a class of inference tree-sitter cannot do |
| https://github.com/jonrad/lsp-mcp | LSP-MCP server | MCP wrapper exposing LSP requests as MCP tools for AI agents; pattern for daemon mode |
| https://engineering.fb.com/2024/12/19/developer-tools/glean-open-source-code-indexing/ | Meta Glean open-source post (Dec 2024) | Glean = precomputed facts DB, Angle query lang; alternative to live LSP for scale; language-agnostic, used internally for code browsing/search |
| https://github.com/Instagram/LibCST | LibCST | Lossless Python CST; complements pyright (LibCST = rewrite, pyright = type infer); python 3.0–3.14 support |
| https://ts-morph.com/ | ts-morph docs | TS compiler API wrapper; in-process — no LSP daemon needed for TS analysis; same type info as ts-server |
| https://github.com/scip-code/scip | SCIP spec | Symbol intelligence as cross-tool protocol; protobuf payload |
| https://markaicode.com/rust-lsp-servers-2025-performance-benchmarks-feature-comparison/ | Rust LSP benchmark 2025 | Servo (500K LOC): time-to-first-completion rust-analyzer 23.4s, Hana 14.2s; cold-start 10+s on large projects |

## LSP-vs-tree-sitter decision matrix
| Signal needed | Tree-sitter sufficient? | LSP wins? | Cost overhead | Citation |
|---|---|---|---|---|
| Function/class/struct boundaries | YES | no | tree-sitter ~1ms/file | aider repomap |
| Symbol DEF sites | YES (with SCIP-enriched map, already in elai.map) | marginal | tree-sitter+SCIP precomputed | sourcegraph SCIP, ELAI elai.map |
| Find-references repo-wide | partial (text-grep false positives on overloads/methods) | YES | rust-analyzer 23s cold, ~50ms warm; ~500 tok output vs 2000+ for grep | amazingcto.com, markaicode |
| Goto-definition cross-module | partial | YES | LSP daemon resident RAM (pyright ~500MB, ra ~1-3GB on 500K LOC) | DeepWiki ra, multilspy |
| Type of an arbitrary expression | NO (no type info) | YES (essential) | hover request <100ms; needs running daemon | rust-analyzer features doc |
| "Does this snippet compile/type-check?" | NO | YES (only path) | full diagnostic pass: rust-analyzer 1-5s incremental, pyright tens of ms | amirteymoori, lanser-cli paper |
| Type narrowing (Pyright reportXxx) | NO | YES | pyright multi-pass inference | If-T benchmark |
| Borrow-checker / lifetime errors | NO (impossible) | YES (only rust-analyzer/cargo) | full cargo check on large workspace = minutes | rust-analyzer features |
| Identifier auto-complete for symbols not in training | NO | YES | LSP completion request <50ms | Monitor-Guided Decoding |
| Find-callers of TODO comment | YES (grep) | no | grep is fine | amirteymoori |
| Cross-repo navigation at scale | NO | partial; SCIP/Glean precomputed wins | precompute is the right model | Glean blog, SCIP announce |

Pattern: tree-sitter+SCIP is the fast static layer; LSP is the *precision oracle* invoked only when the question is type-aware or compile-aware.

## rust-analyzer headless usage
rust-analyzer ships CLI subcommands that produce LSP-equivalent semantic data WITHOUT keeping a daemon resident (per DeepWiki rust-analyzer 7.2 and 9.2):

- `rust-analyzer analysis-stats <path>`: batch type-checking pass; reports inference success rate, types resolved, errors. Useful as a one-shot oracle: "does this workspace fully type-check?"
- `rust-analyzer scip <path>`: emits SCIP protobuf with DEF/REF/symbol metadata + hover docs. Drop-in for ELAI's `.claude/tools/scip_indices` enrichment of elai.map.
- `rust-analyzer lsif <path>`: same data as SCIP but in LSIF JSON-lines (legacy; prefer SCIP).
- `rust-analyzer ssr '$a.foo($b) ==>> bar($a, $b)' <path>`: structured search-and-replace at the AST level — semantic, type-aware refactor. Candidate for ELAI's auto-refactor tier.
- `rust-analyzer diagnostics <path>`: emits all diagnostics (type errors, unused, borrow issues) as one batch; pre-AUDITING gate candidate.

What ELAI can extract programmatically without running a daemon:
1. After every Rust edit, shell out to `rust-analyzer diagnostics --json` on the affected crate. Empty diagnostics → green light for auditor; non-empty → BLOCK with structured error list pre-routed to fixer.
2. Refresh SCIP enrichment of elai.map incrementally per-crate after large edits (`rust-analyzer scip` is what `.claude/tools/scip_indices` already wants; just automate the refresh).
3. Use `ssr` patterns for safe mechanical refactors (constant renames, API migrations) without LLM intervention.

## Multi-language LSP wrapper cost
multilspy (Microsoft Research, github.com/microsoft/multilspy) is the canonical multi-language LSP wrapper. Supports Python (jedi-language-server), Rust (rust-analyzer), Java (Eclipse JDTLS), C# (OmniSharp), TS/JS (typescript-language-server), Go (gopls), Dart, Ruby, Kotlin, PHP, C++ (clangd). Handles binary download, JSON-RPC, server-specific config.

Per-language daemon RAM/startup cost (from markaicode 2025 bench + amazingcto warnings + multilspy issue tracker):

| Server | Cold-start (small) | Cold-start (500K LOC) | Steady RAM (small) | Steady RAM (large) | Notes |
|---|---|---|---|---|---|
| rust-analyzer | 1-3s | 23.4s (Servo) | ~300-500MB | 1.5-3GB | RAM is the bottleneck on monorepos |
| pyright | <1s | 2-5s | 150-300MB | 500MB-1GB | Multi-pass inference; can cap via PYRIGHT_MEMORY_LIMIT |
| typescript-language-server | 1-2s | 5-15s | 200-400MB | 1-2GB | tsserver under the hood |
| gopls | <1s | 2-8s | 100-300MB | 500MB-1GB | Generally well-behaved |
| jdtls | 5-10s | 30-60s | 500MB-1GB | 2-4GB | Heaviest; eclipse JDT runtime |
| clangd | 1-3s | 10-30s | 200-500MB | 1-3GB | Depends on compile_commands.json size |
| jedi-language-server | <1s | 1-3s | 100-200MB | 300-500MB | Fast but no type-check; pyright preferred for verification |

Headless CLI alternative (no resident daemon): rust-analyzer scip/lsif/diagnostics, scip-typescript, scip-python, scip-java — same semantic payload, run-once cost only. This is the model SCIP-the-protocol was designed for and what report-40's SCIP plan already targets.

Practical advice for B-30 (multi-language): for the daemon path, run servers on demand per-language with idle-timeout shutdown (multilspy supports this); for the batch path, precompute SCIP per language and only spawn LSP when ELAI hits a question SCIP cannot answer (live type inference, narrowing, "does this compile").

## Best additions for ELAI
Ranked by ROI / implementation cost:

1. **Diagnostic probe pre-AUDITING gate** | grounding: amazingcto, lanser-cli paper, Claude Code 2.0.74 release | hook: after each Write/Edit, PostToolUse hook shells out to the appropriate one-shot (`rust-analyzer diagnostics`, `pyright --outputjson`, `tsc --noEmit`, etc.) on the affected files. Empty → continue. Non-empty → inject diagnostic JSON into auditor context BEFORE LLM judgment. Cheap (one-shot CLI), language-agnostic via dispatch table.
2. **rust-analyzer SCIP refresh in elai.map regen** | grounding: rust-analyzer DeepWiki 9.2, SCIP announce | hook: extend `.claude/tools/codebase_mapper.py` to call `rust-analyzer scip` on `.elai_cc/` and merge symbols. Already half-done per CLAUDE.md (`.claude/tools/scip_indices` exists). Closes the type-resolution gap in elai.map for Rust without a daemon.
3. **LSP completion list as decoder constraint (Monitor-Guided Decoding)** | grounding: Agrawal et al. arXiv:2306.10763 | hook: at edit time, before main-model emits an identifier following `.` or `::`, run an LSP completion request and constrain the candidate set. High implementation cost (decoder integration), high payoff (eliminates "hallucinated symbol" class of errors).
4. **multilspy daemon for cross-language find-refs / goto-def** | grounding: multilspy README, lsp-mcp project | hook: Python sidecar service exposing `find_references(symbol, lang)` and `definition(symbol, lang)` over MCP. Cached per-session. Used by context-compiler/impact-mapper agents instead of grep when the symbol is overloaded or method-resolution-sensitive.
5. **ts-morph for TypeScript analysis (dashboard-v2 / TS bindings)** | grounding: ts-morph docs | hook: in-process TS analysis from a Node.js helper. Reads `ts-rs` generated bindings, validates Tauri command signatures match Rust side. Cheaper than spinning up tsserver; gives full type info programmatically.
6. **rust-analyzer ssr for mechanical refactors** | grounding: rust-analyzer CLI manual | hook: when impact-mapper identifies a pure rename/signature-shape refactor across >10 sites, dispatch to `rust-analyzer ssr` instead of LLM edits. Deterministic, fast, no audit cycle needed.
7. **LibCST for Python codemods** | grounding: LibCST docs | hook: same as ssr for Rust — pure formatting-preserving Python refactors. Counterpart for the sidecar codebase.
8. **Glean-style precomputed facts DB (LONG-TERM)** | grounding: Meta engineering blog Dec 2024 | hook: if SCIP+elai.map ever caps out, Glean's Angle query language is the next tier. Not urgent.

## Anti-patterns
- **LSP as retrieval-in-context** (Preprints 202510.0924): dumping `find_references` output into the LLM context as the *primary* retrieval signal underperforms RAG+lexical hybrid on coding-agent benchmarks; the cost is token spend without recall gain. LSP wins when used as a verifier on candidate edits, not as the way to find candidates. Match ELAI's existing pattern: use elai.map+SCIP for context, reserve LSP for the precision step.
- **Persistent LSP daemons for every language always-on** (amazingcto.com, markaicode bench): pyright+rust-analyzer+jdtls together on a 500K-LOC repo eat 4-8GB resident — competes with the local sidecar for RAM (CLAUDE.md notes 25K context = ~2.85GB total with NF4 Qwen 3B). On-demand spawn with idle shutdown is required; persistent-daemon-for-all is a non-starter on the current hardware envelope.
- **Tree-sitter for type questions** (If-T benchmark, MGD paper): tree-sitter has no symbol resolution beyond syntactic scope. Trying to answer "is this expression a `Result<T, E>` or an `Option<T>`?" from tree-sitter alone produces guesses. Either resolve via LSP/type-checker or abstain.
- **LSIF over SCIP for new indexing** (Sourcegraph migration doc): LSIF is deprecated; SCIP is ~4x smaller compressed and ~10x faster to produce. Any new indexing pipeline should target SCIP only.
- **Running the full LSP protocol when you only need diagnostics** (lanser-cli framing): one-shot CLI invocations (`rust-analyzer diagnostics`, `pyright`, `tsc --noEmit`) give the same answer as a long-running daemon for the diagnostic probe use-case and avoid the JSON-RPC lifecycle complexity.
- **Aider-style "skip LSP entirely"**: defensible for a hosted multi-language tool but leaves type/compile gates on the table. ELAI's auditor pipeline already runs cargo/pytest; adding LSP diagnostics is a strict superset for ~constant cost.

## Open questions
1. **Diagnostic-probe latency budget**: what's the p99 wall-time for `rust-analyzer diagnostics --json` on each ELAI crate? Acceptable as PostToolUse hook only if <2s; measure on `crates/cli` (medium) and the workspace root.
2. **Pyright recall on sidecar code**: does pyright (strict mode) catch the kinds of bugs the sidecar's NF4/SDPA path produces (dtype mismatches, missing await), or are those runtime-only? Measure: replay the last 20 sidecar bugs from gotchas.md through pyright; count caught vs missed.
3. **MGD payoff for ELAI's main model**: Monitor-Guided Decoding showed SantaCoder-1.1B+MGD ≈ text-davinci-003. Does it move the needle for Qwen 2.5-Coder-3B-Instruct in the cosmos-style verifier loop, or is the model already strong enough that LSP-constrained decoding is a no-op? Pilot test on 10 cross-module edits.
4. **multilspy stability on Windows**: ELAI host is Windows 11, sidecar is Linux. Does multilspy's hand-tuned config work on Windows-hosted rust-analyzer / pyright invocations, or do we need a Linux-side wrapper that proxies LSP over the existing remote-sidecar SSH tunnel?
5. **SCIP refresh incrementality**: per-crate `rust-analyzer scip` is fast (<10s) but does it produce stable symbol IDs across runs so we can merge incremental SCIP into the existing elai.map without rebuilding the whole file? If not, full-rebuild cost vs query freshness becomes the tradeoff to measure.
