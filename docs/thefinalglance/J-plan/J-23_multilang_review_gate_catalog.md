# J-23: multi-language review-gate catalog — SARIF-normalizer + arch-gate TOML over per-language OSS tools

- Track J (pillar-SOTA recheck extensions) | lever AUDITABLE | grounding STRONG (24 tools repo-verified, SARIF support confirmed) | nature BUILD | effort M | WAVE 2 | depends VibeGate F-1/F-2 (extends, does not rebuild)

- Premise: the user found Fallow (github fallow-rs/fallow, Rust CLI, MIT) — whole-project codebase intelligence (dead code, duplication, circular deps, complexity, architecture boundaries) — and wants ELAI's multi-language review/gates pillar to be much better. KEY FACT (verify-fallow): Fallow is JS/TS-ONLY; it is NOT a multi-language tool and must not be described as one. No single OSS tool reproduces Fallow's mix across languages — the right answer is a CATALOG of per-language OSS tools as deterministic Layer-1 review backends for code ELAI generates in a target language, shaped like the J-9 hardware catalog (data-driven, probe-once, per-target eligibility). Two tiers: a SARIF-normalizer for the tools that emit SARIF, and an arch-gate TOML for the JSON/exit-code architecture tools.

- SHAPE (verify-fallow's verdict): NOT a big per-language catalog and NOT SARIF-only — a TWO-TIER normalizer. Tier-1 = sarif_ingest consuming the SARIF-emitting tools; Tier-2 = arch_gate TOML with per-tool invoke_cmd + parser_fn (json | exit_code | text_regex) for the architecture-boundary tools that emit JSON/exit-code.

## Steal/build
1. sarif_ingest (crates, new): a SARIF v2.1.0 parser that normalizes findings from SARIF-emitting tools into ELAI's gate-verdict shape. CONFIRMED SARIF emitters (verify-fallow): Semgrep CE (--sarif; NOTE CE = single-file only, cross-file/dataflow is PAID), PMD+CPD (SARIF since v6.31), Psalm (PHP), detekt (Kotlin). CodeQL emits SARIF natively but is LICENSE-BLOCKED for closed-source target projects (requires GitHub Advanced Security) — catalog it as license-gated, never invoke it on a closed-source target without the license check passing.
2. arch_gate TOML catalog (data-driven, the J-9 shape): per-language entries with { tool, invoke_cmd, parser_fn: json|exit_code|text_regex, sarif: bool, license, staleness_flag }. The architecture-boundary tools (Fallow's lane) mostly emit JSON or exit-code, NOT SARIF: dependency-cruiser (JSON), Knip (JSON), Deptrac (JSON), Periphery (JSON), import-linter (exit-code), Packwerk (exit-code). The catalog is the DATA; a thin runner shells out + parses per the parser_fn.
3. GATE ELAI'S OWN CODE FIRST (the first consumer, verify-fallow B): Rust = cargo-modules (orphaned/unlinked source) + cargo-udeps (unused deps, needs +nightly — CI prerequisite) + clippy; Python (sidecar) = vulture (dead code) + ruff; TS/Svelte (dashboard) = knip + dependency-cruiser. This dogfoods the catalog on ELAI before any target-language generation uses it.
4. FRONTEND DESIGN/A11Y GATE (the de-AI-UI deterministic subset, folded here): a design-token-drift check (flag spacing/radii/font-size values not in the project token scale) + an axe-core a11y-tree gate (the deai-ui report's strongest deterministic finding: a polished hover with role=generic / name=none / focusable=false is a deterministic FAIL). The taste judgments ("looks generic", "weak hierarchy") stay the design-taste.md / visual-ui-loop.md ADVISORY lens (LLM-judgment, not a gate); only the mechanically-checkable subset (token drift, a11y-tree, banned-trope presence-grep) becomes a deterministic gate.
EXCLUDED: calling Fallow multi-language (it is JS/TS-only); CodeQL on closed-source targets without the license gate; the 3 STALE/unfit tools verify-fallow dropped (go-callvis stale Jan-2025 viz-only, jQAssistant GPL-3.0, Konsist Dec-2024 test-framework-only); a big monolithic per-language catalog (the two-tier normalizer is the shape); the taste-rubric as a gate (advisory only, TD-3); building our own per-language analyzers (shell out to the verified OSS tools).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- VIBEGATE UNIFIED WRITE GATE [WIRED] fail-closed multi-axis PreToolUse: code-vuln (AST/GREP) + secret (GREP) Tier-1; dep-existence + skill-scan Tier-2 (feature map 1.8; F-1) — the gate chain J-23's review backends extend.
- CODE-VULN RULESET [WIRED code] F-2a Py/TS BLOCK rules + F-2c 4 CWE TOMLs (gate_recall=0.6875) — the deterministic-rule precedent; J-23 adds external-tool backends as additional Layer-1 evidence.
- J-9 HARDWARE MODEL CATALOG [PLANNED] probe-once data-driven per-lane eligibility TOML (hardware_catalog.rs) — the EXACT structural precedent J-23 mirrors for review tools (catalog-as-data + a thin runner).
- VISUAL UI LOOP [WIRED, .claude] ui-diff.mjs + ax-scaffold.mjs (AX-tree) + schema-constrained critique + design-taste.md — the a11y-tree the design gate reads is the ax-scaffold output; the taste critique stays advisory.
- cross_lang_diff.py + .claude/tools/ harness tools — the harness-side analog; J-23's catalog is the .elai_cc runtime gate version.

## Integration
- crates (new): sarif_ingest (~200 LOC SARIF v2.1.0 parser -> gate verdict) + arch_gate runner (~150 LOC: read the TOML, shell out per invoke_cmd, parse per parser_fn) + the arch_gate.toml catalog data.
- the design/a11y gate rides the existing visual-ui-loop ax-scaffold output (the a11y tree) + a token-scale check (~80 LOC); dashboard-only.
- +ReviewGateCatalogEvent SSB (Pattern-2 single-variant; Kind: CatalogBuilt / GateInvoked / SarifIngested / VerdictEmitted). baseline+1 from LIVE 231/542 (grep 2026-06-12); re-read at edit (W-9). Full 6-part variant discipline.
- frontend_pair: a review-gate findings surface on the existing audit/vibesec panel + the a11y/token findings on the dashboard's own QA surface (W-4 cross_lang_diff for the dashboard leg).

## WIRING_CONTRACT
produces_symbols:
- crates sarif_ingest::parse_sarif (SARIF v2.1.0 -> ELAI gate verdict)
- crates arch_gate::run_catalog (TOML-driven shell-out + per-tool parser_fn)
- the arch_gate.toml per-language tool catalog (data)
- the design-token-drift + axe-core a11y-tree deterministic dashboard gate
consumes_symbols:
- F-1 VibeGate PreToolUse chain (the gate chain J-23's backends extend; NOT reauthored)
- the external OSS tool binaries (shelled out, version-pinned; anti-slopsquat per tool)
- the visual-ui-loop ax-scaffold a11y tree (the a11y gate's input; NOT reauthored)
reachable_from:
- a real `elai` review/audit run on generated code in a cataloged language (the AUDITING path -> arch_gate runner -> tool invoke -> verdict)  [PRODUCTION]
- ELAI's own CI gating its Rust/Python/TS (the dogfood consumer)
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO calling Fallow on non-JS/TS (it is JS/TS-only; the catalog routes per language)
- NO CodeQL invocation on a closed-source target without the license-gate passing
- NO unpinned / registry-resolved tool install (pin each tool version; anti-slopsquat; flag the +nightly cargo-udeps prerequisite)
- NO stale-tool default (the 3 dropped tools stay out; staleness_flag in the catalog)
- NO taste-rubric as a deterministic gate (advisory lens only; only token-drift / a11y-tree / banned-trope-presence are gates; TD-3)
- NO BLOCK default until the per-tool FP rate is bounded on >=3 real runs (GD-4) + a wire-dark-allowlist row (W-2/R9)
frontend_pair: review-gate findings on the vibesec/audit panel + a11y/token on the dashboard QA surface (W-4) or W-6-justified absence.
observable_effect:
- trigger: "an `elai` review run on a generated Python file with dead code + a generated Svelte component with a div-onClick non-semantic button, through the real AUDITING path"
- expect_ssb: "CatalogBuilt on first run (the per-language tool eligibility); GateInvoked=vulture on the Python file; SarifIngested if the tool emits SARIF; VerdictEmitted with the dead-code finding; the a11y gate flags the div-onClick (role=generic/name=none/focusable=false)"
- expect_artifact: "vulture's dead-code finding + the a11y-tree FAIL are surfaced as deterministic gate verdicts (not LLM opinion); ELAI's own Rust gated by cargo-modules/udeps. NEGATIVE (R2): a SARIF tool that returns no findings yields a clean VerdictEmitted (not a false flag); a tool binary missing on the box FAILS-LOUD (TOOL_MISSING, never a silent pass); CodeQL on a closed-source target without the license is REFUSED, not run; a clean component passes the a11y gate."

## Grounding
- verify-fallow (all repos GitHub-verified 2026-06-11): Fallow JS/TS-only (3.5k stars, MIT, v2.92.1 active). SARIF emitters confirmed: Semgrep CE (LGPL, cross-file PAID), PMD+CPD (BSD, SARIF v6.31+), Psalm (MIT), detekt (Apache). CodeQL SARIF-native but license-blocked for closed-source. Arch tools JSON/exit-code: dependency-cruiser, Knip, Deptrac, Periphery, import-linter, Packwerk. Dropped: go-callvis (stale), jQAssistant (GPL-3.0), Konsist (stale, test-only). cargo-udeps needs +nightly; qossmic/deptrac archived -> deptrac/deptrac canonical.
- deai-ui digest (deai-ui.md): the a11y-tree FAIL (Frontend Masters: role=generic/name=none/focusable=false on a polished AI button) + token-drift are the deterministic subset; the rest is the design-taste.md advisory lens.
- J-9 hardware catalog (the structural precedent); F-1/F-2 VibeGate (the gate chain extended). live SSB 231/542 (grep 2026-06-12).

## COMPOSES-WITH
F-1/F-2 VibeGate (J-23 adds external-tool Layer-1 backends), J-9 hardware catalog (same data-driven probe-once shape), visual-ui-loop + design-taste.md (the a11y/token gate is the deterministic floor under the advisory critique), E-43 skill-import skillspector (the same "shell out to a verified external tool, parse a verdict" pattern), J-16 blast-radius (both are auditor-owned deterministic Layer-1 gates). Restates none.

## Open questions / risks
- tool sprawl vs the 80%: verify-fallow C asks whether a thin SARIF layer over 3-4 cross-language tools (Semgrep/PMD-CPD/Lizard) covers 80% — START there (the cross-language SARIF core), add per-language arch tools (Deptrac/Periphery/cargo-modules) only where the target language demands architecture-boundary checks the cross-language tools miss. Do not author all 24 at once; the catalog grows per demonstrated need (no silent over-build).
- license gates: CodeQL (closed-source), Semgrep cross-file (paid) — the catalog must carry the license field and the runner must REFUSE a license-gated invocation on an ineligible target, never silently degrade to a weaker check that looks like a pass.
- staleness: catalog the last-release date; a stale tool (go-callvis Jan-2025) is dropped, not invoked — re-check staleness on catalog rebuild.

## Next
The cross-language SARIF core (Semgrep/PMD-CPD) is the cheapest first slice and immediately dogfoods on ELAI's own Python/TS. The arch-gate TOML grows per target-language demand. This is the deterministic Layer-1 review backend the user asked for — Fallow's lane, done across languages, as data not prose.

End of J-23.
