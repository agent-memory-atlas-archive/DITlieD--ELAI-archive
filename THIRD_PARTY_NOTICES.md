# Third-party notices

This inventory covers third-party material found in the tracked public archive during a bounded review on 2026-09-06. It is an attribution and license record, not legal advice, complete provenance, or blanket legal clearance. The review used `git ls-files` and inspected the provenance of the named source, data, and frontend fixture paths.

Original ELAI source and plan material use MIT. Upstream material keeps its own terms.

## Covered material

| Material | Tracked paths | Terms and required attribution | Primary source |
| --- | --- | --- | --- |
| Claw Code Rust port starting point | Tracked Rust workspace portions derived from the starting workspace, including active sources under `.elai_cc/crates/**` and workspace manifests; the exact boundary is unresolved | MIT. Retain: Copyright (c) 2026 UltraWorkers and Claw Code contributors. The exact fetched text is in [third-party-licenses/claw-code-MIT.txt](third-party-licenses/claw-code-MIT.txt). | [Original repository URL](https://github.com/instructkr/claw-code) (redirects to [ultraworkers/claw-code](https://github.com/ultraworkers/claw-code)); [upstream LICENSE](https://raw.githubusercontent.com/instructkr/claw-code/main/LICENSE) |
| TruthfulQA-derived calibration material | `.elai_cc/benchmarks/calibration_bootstrap/truthful_qa_790.jsonl`; `.elai_cc/benchmarks/calibration_bootstrap/fixtures/truthfulqa_full_790.jsonl`; `.elai_cc/benchmarks/calibration_bootstrap/fixtures/truthfulqa_subset.jsonl` | Apache License 2.0. The 790-row file normalizes the published `TruthfulQA.csv` fields into JSONL and adds `source` and `row_id`. The 1,580-row file deterministically projects each question into a best-answer row and a first-incorrect-answer row with local IDs, scores, and timestamps. The 50-row subset is an ELAI representative calibration projection using that schema, not a verbatim CSV copy. The exact upstream text is in [third-party-licenses/truthfulqa-Apache-2.0.txt](third-party-licenses/truthfulqa-Apache-2.0.txt). | [TruthfulQA repository](https://github.com/sylinrl/TruthfulQA); [TruthfulQA LICENSE](https://raw.githubusercontent.com/sylinrl/TruthfulQA/main/LICENSE); [TruthfulQA source data](https://raw.githubusercontent.com/sylinrl/TruthfulQA/main/TruthfulQA.csv) |

## Excluded local inputs

The public Git snapshot omits these four local HaluEval/MMLU inputs; their local originals are preserved for existing local workflows:

- `.elai_cc/benchmarks/calibration_bootstrap/halueval_qa_500.jsonl`
- `.elai_cc/benchmarks/calibration_bootstrap/fixtures/halueval_qa_500_full.jsonl`
- `.elai_cc/benchmarks/calibration_bootstrap/mmlu_verified_500.jsonl`
- `.elai_cc/benchmarks/calibration_bootstrap/fixtures/mmlu_verified_500_full.jsonl`

Their HaluEval, MMLU, and HotpotQA source/license records under `third-party-licenses/` are review records for omitted local inputs; they do not represent those datasets as released archive content. [DATA-NOTES.md](.elai_cc/benchmarks/calibration_bootstrap/DATA-NOTES.md) records the reproducibility consequence.

## Reviewed material with no separate third-party notice

- `.elai_cc/calibration/seed/calibration_seed.jsonl` is documented as a 20-tuple synthetic corpus in `.elai_cc/calibration/seed/README.md`. The UniCR and conformal-prediction references are citations, not copied text or data.
- The dashboard source uses ordinary package dependencies such as Svelte, Three.js, Lucide, Troika, Cytoscape, Sigma, Graphology, JetBrains Mono, and Tauri. Their source is not vendored in the tracked archive; this review does not attempt a complete Cargo/npm dependency license inventory.
- The dashboard file `.elai_cc/crates/dashboard-v2/src/lib/streaming/frame_pacer.ts` names Vercel AI SDK `smoothStream` as a design reference and explicitly says “no code reuse.” `.elai_cc/crates/dashboard-v2/src/lib/orbit/lcg.ts` copies a pattern from the local `Starfield.svelte`; `.elai_cc/crates/dashboard-v2/src/lib/orbit/node-mandala.ts` borrows geometry from the local `planet-scan.ts`. These are internal references or stated inspiration; no external copied JS component was identified.
- The small companion files `32x32.png`, `128x128.png`, `128x128@2x.png`, and `icon.ico` in the Tauri icon directory are tiny transparent placeholder assets (83, 143, 334, and 105 bytes respectively). They contain no separate third-party artwork identified in this review. The 1280px default Tauri logo bitmap is excluded.
- The tracked minified_project JavaScript/CSS/Go files under `.elai_cc/crates/knowledge/tests/fixtures/` are synthetic chunking fixtures: repeated trivial IIFEs, repeated CSS rules, filler strings, and a generated-looking test stub. No external component identity or license header was found.
- The 80 `syco_synth` fixtures are marked ELAI-synthetic. The ARC-AGI-2 mini fixture and PRMBench mini fixtures are documented as synthetic or hand-written rather than verbatim upstream benchmark data. External benchmark wrappers under `.elai_cc/benches/external/` are host-side scaffolding that loads datasets at runtime; no corresponding raw corpus is tracked.
- The starter files under `.elai_cc/templates/web/` are marked Generated by ELAI. Their npm packages are dependencies, not copied package source.

## Provenance limits and final exclusions

The architecture specification says that the project started from a claw-code-rust-port Rust workspace (`docs/specs/architecture-overview.txt`, section 2). The downloaded copy at `docs/.source/claw-code-rust-port/claw-code-rust-port` is ignored and is not part of the public archive. The tracked Rust workspace declares `license = "MIT"` in `.elai_cc/Cargo.toml`, but no original upstream LICENSE is included alongside that tracked workspace. The current upstream LICENSE was therefore fetched from the primary source above. The imported commit or tag is not recorded here, so the exact file-by-file boundary of the port remains unresolved. This is a provenance limitation, not a claim of complete provenance.

The four HaluEval/MMLU inputs are local-only in the final public scope. The older benchmark checks that consumed them are not reproducible from the archive alone; `DATA-NOTES.md` records that fact. Their source uncertainty is not treated as an active public-release blocker because the inputs are excluded.

The default Tauri logo bitmap (`icons/icon.png`) is omitted. Its upstream
logo/design terms differ from the framework's software license. The source and
attribution record is retained in [tauri-logo-terms.txt](third-party-licenses/tauri-logo-terms.txt).
The small transparent icons referenced by the bundle configuration remain.

The 20 `syco_sharma_*.json` fixtures and the generator containing their source
rows are omitted because their recorded adaptation provenance did not establish
upstream reuse terms. The 80 locally synthetic fixtures remain. This omission
changes the historical benchmark corpus and leaves its old 100-fixture claims
unreproducible from this snapshot. See the [provenance record](third-party-licenses/sycophancy-eval-provenance.txt).

No blanket legal clearance is asserted. Transitive Cargo/npm package terms,
excluded runtime/session data, bundled tool archives, downloaded repositories,
raw paper PDFs and upstream prompt dumps are outside this bounded inventory.

The synthetic calibration seed and synthetic minified fixtures produced no separate release issue in this review.
