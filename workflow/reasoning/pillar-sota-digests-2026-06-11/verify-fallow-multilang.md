verify-fallow-multilang.md
task_id: verify-fallow-multilang
date: 2026-06-11
scope: anti-slopsquat verification of per-language static-analysis gate backends

---

TOOL CATALOG TABLE

All data sourced from live GitHub fetches and web searches on 2026-06-11.
Columns: Tool | Primary Lang Target | Repo Exists | License | Stars (approx) | Last Release / Recency | JSON output | SARIF output

--- FALLOW ---

Tool: Fallow
Repo: fallow-rs/fallow
Exists: YES
License: MIT
Stars: 3,500
Last Release: v2.92.1, June 10 2026 (very fresh)
Lang Target: JS/TS ONLY (Oxc parser, zero Python/Rust/Go/Java support)
JSON: YES (--format json, carries actions[] + auto_fixable flag, kind discriminator)
SARIF: YES (--format sarif, note-level with per-category rule IDs + CWE tags, integrates with GitHub Security tab)
Notes: Free static layer = unused code, duplication, circular deps, complexity hotspots, architecture boundaries.
Paid runtime layer = production traffic coverage (V8/Istanbul dumps) for cold-path deletion evidence.
MCP skill repo: fallow-rs/fallow-skills. JS/TS ONLY is the critical constraint for a multi-lang catalog.

--- CROSS-LANGUAGE ---

Tool: CodeQL (engine + CLI)
Repo: github/codeql
Exists: YES
License: Engine/CLI = proprietary (NOT open source). Queries = MIT/Apache.
RESTRICTION: free for OSI open-source repos only; closed-source requires GitHub Advanced Security paid license.
Stars: ~9,500 (queries repo)
Last Release: 2.24.0, Jan 2026
JSON: NO (outputs SARIF; BQRS binary format also available)
SARIF: YES (canonical; codeql database analyze --format sarif-latest; this is its primary output)
Notes: HARD LICENSE RISK for ELAI's own closed-source use. ELAI generating code for a target closed-source project
would trigger the commercial license gate. Flag this as blocked for closed-source target projects.

Tool: Semgrep CE
Repo: semgrep/semgrep
Exists: YES
License: LGPL-2.1 (engine); rules repo separate
Stars: 15,500
Last Release: 1.160.0, April 2026
JSON: YES (--json)
SARIF: YES (--sarif, --sarif-output=file; SARIF fields documented for CE)
Cross-file dataflow: PAID-only (Semgrep AppSec Platform / Pro engine). CE = single-file analysis only.
Notes: CE cross-file dataflow gap means architecture-boundary violations (the Fallow-style use case) are NOT
covered by Semgrep CE. CE is a bug/vuln pattern scanner, not an architecture enforcer.

Tool: PMD + CPD
Repo: pmd/pmd
Exists: YES
License: BSD-style
Stars: 5,400
Last Release: 7.25.0, May 29 2026 (very fresh)
JSON: NO (no JSON formatter; output formats: text, xml, csv, html, sarif)
SARIF: YES (since v6.31.0; initial implementation, extended in later releases; confirmed in docs)
CPD (copy-paste): same tool, inherits PMD formatters including SARIF
Notes: Covers Java, Apex, JavaScript, XML, Python, Kotlin, Swift, Scala, etc. Good cross-language breadth.

Tool: Lizard
Repo: terryyin/lizard
Exists: YES
License: MIT (confirmed)
Stars: 2,400
Last Release: 1.23.0, June 2 2026 (fresh)
JSON: NO (native: text, XML/cppncss, CSV, HTML, Checkstyle XML; no JSON)
SARIF: NO
Notes: Cyclomatic complexity + duplicate detection across 27+ languages. CLI exit code non-zero on threshold
violations. Machine-parseable via XML/CSV, not JSON/SARIF natively. Useful but requires adapter.

Tool: Infer
Repo: facebook/infer
Exists: YES
License: MIT (Java analysis may pull GPL components)
Stars: 15,600
Last Release: v1.3.0, May 12 2026 (fresh)
JSON: YES (infer outputs report.json natively; the canonical output is JSON)
SARIF: NO native; third-party converters exist (SarifConverter CLI: infer_report.json -> .sarif)
Notes: Interprocedural analysis, multi-language (C/C++/Java/Objective-C/Erlang). Heavy: requires build
instrumentation. Not a lightweight shell-out; needs full infer compile + infer run cycle.

--- JS / TS ---

Tool: dependency-cruiser
Repo: sverweij/dependency-cruiser
Exists: YES
License: MIT
Stars: 6,700
Last Release: v17.4.3, May 29 2026 (fresh)
JSON: YES (--output-type json)
SARIF: NO (output types: json, csv, html, text/err, dot, mermaid, d2; no SARIF listed)
Notes: Architecture boundaries, circular deps, forbidden import rules for JS/TS. Already used by ELAI harness
(canvases the .elai_cc TS/JS leg). Confirmed correct pick for ELAI own-stack.

Tool: Knip
Repo: webpro-nl/knip
Exists: YES
License: ISC
Stars: 11,500
Last Release: 6.16.1, June 6 2026 (very fresh)
JSON: YES (--reporter json; outputs unused files and issues as array of objects per file)
SARIF: NO (no native SARIF reporter; custom reporters supported via --reporter <path>)
Notes: Dead code, unused exports, unused deps for JS/TS. Already used by ELAI harness. Confirmed correct pick.

--- PYTHON ---

Tool: import-linter
Repo: seddonym/import-linter
Exists: YES
License: BSD-2-Clause
Stars: 1,100
Last Release: 2.11, March 6 2026 (fresh)
JSON: NO (text output only; no --json flag found in docs; exit code non-zero on violation)
SARIF: NO
Notes: Architecture contracts (forbidden/independence/layers) for Python. Pass/fail verdict via exit code.
Usable as a gate (exit code) but no structured output for a normalizer.

Tool: Vulture
Repo: jendrikseipp/vulture
Exists: YES
License: MIT
Stars: 4,700
Last Release: v2.16, March 25 2026 (fresh)
JSON: NO (text output: "path:line: unused X (N% confidence)"; no --json flag)
SARIF: NO
Notes: Dead code finder for Python. Supports whitelists. Confidence scores in text output.
Parseable by regex; not JSON/SARIF native. Correct pick for ELAI Python sidecar dead-code gate.

--- RUST ---

Tool: cargo-modules
Repo: regexident/cargo-modules
Exists: YES
License: MPL-2.0
Stars: 1,200
Last Release: 0.25.0 (edition 2024, requires Rust 1.88.0; confirmed from Cargo.toml in repo search)
JSON: NO (native outputs: text tree for structure, DOT graph for dependencies, text for orphans)
SARIF: NO
Notes: Module structure / dependency visualization + orphan detection. Correct pick for ELAI Rust layer.
MPL-2.0 is copyleft-lite (file-level copyleft, not project-level); compatible with MIT/Apache projects
but worth noting for a plan unit.

Tool: cargo-udeps
Repo: est31/cargo-udeps
Exists: YES
License: MIT/Apache-2.0 (dual)
Stars: 2,100
Last Release: v0.1.61, April 29 2026 (fresh)
JSON: NO (text output only)
SARIF: NO
NIGHTLY REQUIRED: YES (confirmed; must run with +nightly toolchain)
Notes: Correct pick for unused dependency detection. Nightly constraint is a CI setup cost but not a blocker.

--- GO ---

Tool: goda
Repo: loov/goda
Exists: YES
License: MIT
Stars: 1,700
Last Release: v0.9.2, February 25 2026 (fresh; includes deadcode detection)
JSON: PARTIAL (supports -f template flag for custom formatting; no built-in --json flag confirmed)
SARIF: NO
Notes: Go dependency graph + deadcode detection. Correct pick. No native JSON; templated output.

Tool: go-callvis
Repo: ondrajz/go-callvis
Exists: YES
License: MIT
Stars: 6,500
Last Release: v0.7.1, January 2 2025 (18 months old as of June 2026 = STALE RISK)
JSON: NO (graphviz/DOT output for visualization)
SARIF: NO
Notes: Call graph VISUALIZATION tool, not a gate backend. Stale. Not suitable as a deterministic gate
backend. Drop from catalog or mark VISUALIZATION_ONLY.

--- JAVA ---

Tool: ArchUnit
Repo: TNG/ArchUnit
Exists: YES
License: Apache-2.0
Stars: 3,700
Last Release: 1.4.2, April 18 2026 (fresh)
JSON: NO (JUnit XML test reports via test framework; no standalone JSON output)
SARIF: NO
Notes: Architecture rules as JUnit tests (Java code). Outputs pass/fail via test framework. Not a CLI
shell-out; requires Java + Maven/Gradle project setup. Verdict via JUnit XML, not SARIF.

Tool: jQAssistant
Repo: jQAssistant/jqassistant
Exists: YES
License: GPL-3.0 (HARD license constraint for commercial/closed use)
Stars: 280 (low adoption)
Last Release: 2.9.1, February 17 2026
JSON: UNKNOWN (not confirmed from available docs; GraphML and XML reports documented)
SARIF: UNKNOWN
Notes: GPL-3.0 = copyleft infection risk for any project that links it. Low adoption. Deferred or drop.

--- C# ---

Tool: ArchUnitNET
Repo: TNG/ArchUnitNET
Exists: YES
License: Apache-2.0
Stars: 1,300
Last Release: 0.13.3, March 5 2026 (fresh)
JSON: NO (verdict via .NET test framework; no standalone JSON)
SARIF: NO
Notes: Same pattern as ArchUnit Java: rules as code, verdict via test output. No CLI shell-out model.

--- PHP ---

Tool: Deptrac (new canonical repo)
Repo: deptrac/deptrac
Exists: YES (old qossmic/deptrac ARCHIVED Feb 2025; new canonical: deptrac/deptrac)
License: MIT
Stars: 3,000 (new repo)
Last Release: 4.6.1, May 13 2026 (fresh)
JSON: YES (dedicated JSON formatter; confirmed in formatters docs)
SARIF: NO (formatters: baseline, console, githubactions, graphviz, mermaidjs, json, junit, table, codeclimate)
Notes: Architecture layers/boundaries for PHP. The old qossmic repo is archived; use deptrac/deptrac.

Tool: Psalm
Repo: vimeo/psalm
Exists: YES
License: MIT
Stars: 5,900
Last Release: 6.16.1, March 19 2026 (fresh)
JSON: YES (--report=psalm-report.json; format: .json among others)
SARIF: YES (--report=psalm-report.sarif or psalm-report.sarif.json; confirmed in docs)
Notes: PHP static analysis + taint tracking. Strong SARIF support. Separate from Deptrac but complementary.

--- RUBY ---

Tool: Packwerk
Repo: Shopify/packwerk
Exists: YES
License: MIT
Stars: 1,900
Last Release: 3.3.0, May 6 2026 (fresh)
JSON: NO (text output to stdout; no --format json confirmed)
SARIF: NO
Notes: Rails package boundary enforcement. Verdict via exit code + text. No structured output confirmed.

--- SWIFT ---

Tool: Periphery
Repo: peripheryapp/periphery
Exists: YES
License: MIT
Stars: 6,100
Last Release: 3.7.4, April 26 2026 (fresh)
JSON: YES (--format json confirmed)
SARIF: NO (SARIF for Swift diagnostics is a pitch in Swift Forums, not yet in Periphery)
Notes: Correct and well-maintained tool for Swift dead code. JSON output confirmed.

--- KOTLIN ---

Tool: detekt
Repo: detekt/detekt
Exists: YES
License: Apache-2.0
Stars: 7,000
Last Release: v1.23.8, February 21 2025 (stable; 2.0.0-alpha builds exist against Kotlin 2.3)
STALE RISK: Stable line at Feb 2025, ~16 months old. BUT 2.0 alpha active, project maintained.
JSON: NO (output formats: html, markdown, sarif, xml/checkstyle; no dedicated JSON)
SARIF: YES (confirmed in docs; --report SARIF format)
Notes: Not dangerously stale; 2.0 alpha shows active dev. SARIF confirmed.

Tool: Konsist
Repo: LemonAppDev/konsist
Exists: YES
License: Apache-2.0
Stars: 1,700
Last Release: v0.17.3, December 8 2024 (6 months old as of June 2026 = mild stale risk)
JSON: NO (JUnit-style test results; no CLI JSON output)
SARIF: NO
Notes: Architecture rules as Kotlin tests. Low recent activity. Mild staleness. Similar pattern to ArchUnit.

--- HASKELL ---

Tool: Weeder
Repo: ocharles/weeder
Exists: YES
License: BSD-3-Clause
Stars: 186 (niche)
Last Release: 2.10.0, August 1 2025 (10 months; acceptable for niche)
JSON: NO (text output: file:line: symbol)
SARIF: NO (a separate haskell sarif library exists: mbg/sarif; Weeder does not use it)
Notes: Dead code via GHC HIE files. Niche. No structured output.

--- SCALA ---

Tool: Scapegoat
Repo: sksamuel/scapegoat
Exists: YES
License: Apache-2.0
Stars: 557
Last Release: 3.3.5, June 3 2026 (fresh)
JSON: NO (formats: xml, html, scalastyle, markdown, gitlab-codeclimate)
SARIF: NO
Notes: Scala static analysis. No SARIF. XML/GitLab Code Quality are the machine-readable options.

---

QUESTION A: SARIF-EMITTING TOOLS

Tools with confirmed native SARIF output (suitable for a unified SARIF-ingest gate):

1. Fallow (JS/TS) - YES, --format sarif, note-level + CWE tags
2. CodeQL - YES, canonical SARIF output; LICENSE BLOCKED for closed-source use
3. Semgrep CE - YES, --sarif; CE = single-file only (cross-file is paid)
4. PMD + CPD (multi-lang) - YES, since v6.31.0
5. Psalm (PHP) - YES, .sarif extension confirmed
6. detekt (Kotlin) - YES, --report SARIF confirmed

Tools with JSON (not SARIF) as primary machine-readable output:

- Infer (JSON natively, SARIF via external converter)
- dependency-cruiser (JSON)
- Knip (JSON)
- Deptrac (JSON)
- Periphery (JSON)
- Fallow (JSON also available alongside SARIF)

Tools with NO machine-readable output (text/exit-code only):

- import-linter (exit code; text)
- vulture (text; parseable by regex)
- cargo-modules (tree/DOT)
- cargo-udeps (text)
- goda (template; no JSON flag)
- Packwerk (text/exit code)
- Weeder (text)
- ArchUnit / ArchUnitNET (JUnit XML via test framework; no standalone gate output)
- Konsist (JUnit via test framework)
- Scapegoat (XML/GitLab Code Quality, no SARIF)

SARIF-emitting tools that are practically usable as a normalizer input:
Semgrep CE, PMD, Psalm, detekt = clean SARIF out of the box.
Fallow = SARIF but JS/TS only.
CodeQL = SARIF but license-blocked for closed-source.

---

QUESTION B: TOOLS FOR ELAI'S OWN STACK

ELAI stack = Rust orchestrator + Python sidecar + Svelte/TS dashboard.

Rust:
- cargo-modules (MPL-2.0; structure/orphan detection) - CONFIRMED correct
- cargo-udeps (MIT/Apache; unused deps; nightly required) - CONFIRMED correct
- clippy (built-in; lint + machine-readable via --message-format=json) - already in workflow, complement

Python:
- vulture (MIT; dead code; text output, regex-parseable) - CONFIRMED correct
- import-linter (BSD-2; architecture contracts; exit-code gate) - CONFIRMED correct
- ruff (already used; lint + format; outputs JSON via --output-format json) - complement

TypeScript / Svelte:
- knip (ISC; unused code + deps; JSON output) - CONFIRMED correct
- dependency-cruiser (MIT; architecture boundaries + circular deps; JSON output) - CONFIRMED correct
  (dependency-cruiser already wired into the canvas TS/JS leg of ELAI)

Summary: the task-stated picks (cargo-modules/udeps/clippy, vulture/ruff, knip/dependency-cruiser)
are verified correct for ELAI's own stack. No substitutions needed.

---

QUESTION C: BIG CATALOG vs SMALL SARIF NORMALIZER

Evidence for SMALL SARIF normalizer covering 80%:

The 4 cross-language tools with SARIF output (Semgrep CE, PMD, Psalm, detekt) cover:
- Bug/security patterns: Semgrep CE (30+ languages, single-file)
- Style/quality/complexity: PMD (Java, Kotlin, JS, Python, Swift, Scala, XML, Apex...)
- PHP deep analysis: Psalm (dedicated)
- Kotlin dedicated: detekt

The architecture-boundary enforcement (the Fallow use case) is NOT covered by any of these.
The SARIF-emitting tools are primarily bug/lint/quality scanners, not architecture enforcers.
Architecture enforcement (circular deps, layer violations, boundary rules) in each language uses
tools that either output JSON (dep-cruiser, Deptrac, Periphery) or JUnit/exit-code (ArchUnit, Packwerk).

So the real split is:
- BUG/QUALITY lane: 3-4 SARIF tools = already ~80% coverage
- ARCHITECTURE lane: per-language non-SARIF tools; needs a JSON-ingest layer too (not just SARIF)

HONEST ANSWER:
A thin SARIF normalizer handles the bug/quality lane but misses the architecture-enforcement lane
entirely. The architecture lane needs a separate JSON/exit-code normalizer that handles tools like
dependency-cruiser, Deptrac, Packwerk, Periphery, import-linter (exit code), and Fallow (JSON/SARIF).

The plan unit should NOT be "big per-language catalog" (too wide, maintenance-heavy) and should NOT
be "SARIF normalizer only" (leaves architecture lane uncovered).

SHAPE-HINT (see below).

---

SHAPE-HINT: two-tier gate normalizer, not a big catalog

Tier 1 - SARIF normalizer (4 tools, cross-language bug/quality):
  Semgrep CE (30+ langs, single-file bugs), PMD (multi-lang quality),
  Psalm (PHP), detekt (Kotlin) = ingest via sarif-ingest.rs reading SARIF 2.1.0 schema.
  CodeQL excluded (license-blocked for closed-source target projects).

Tier 2 - JSON/exit-code architecture normalizer (per-stack, data-driven via hardware-catalog pattern):
  Each row: tool | lang | invoke_cmd | output_mode (json|exit_code|xml) | parser_fn
  Representative rows:
    dependency-cruiser | ts/js  | --output-type json | json
    knip               | ts/js  | --reporter json    | json
    fallow             | ts/js  | --format json      | json  (or --format sarif -> tier 1)
    Deptrac            | php    | --formatter json   | json
    Periphery          | swift  | --format json      | json
    import-linter      | python | (none)             | exit_code
    vulture            | python | (none)             | text_regex
    cargo-modules      | rust   | (none)             | text_tree (parse DOT or text)
    cargo-udeps        | rust   | (none)             | text_regex  [nightly only]
    Packwerk           | ruby   | (none)             | exit_code
    ArchUnit           | java   | (none)             | junit_xml
    detekt             | kotlin | --report SARIF     | sarif -> tier 1 OR direct
    Scapegoat          | scala  | (none)             | xml/gitlab-codeclimate

ELAI own-stack rows are a subset of Tier 2 (always enabled).
Target-project rows are selected by the hardware-catalog probe pattern (probe language mix, enable eligible tools).

This is a 2-crate plan unit: sarif_ingest + arch_gate (the catalog rows live in a TOML config file).
Size: medium. Not a "big catalog" (no per-tool monolithic wrappers) and not "SARIF only" (covers arch lane).

---

TOOLS THAT DO NOT EXIST OR ARE PROBLEMATIC

1. go-callvis (ondrajz/go-callvis): EXISTS but STALE (last release Jan 2025, 18 months).
   Is a VISUALIZATION tool (graphviz output), not a deterministic gate backend. DROP from catalog.

2. Konsist (LemonAppDev/konsist): EXISTS but mild stale (last release Dec 2024, 6 months).
   Architecture rules as Kotlin TESTS, not a CLI gate. Needs test-framework setup. DEFER or drop.

3. jQAssistant: EXISTS but GPL-3.0 license (copyleft infection risk) + low adoption (280 stars).
   DROP from catalog unless target project is itself GPL.

4. CodeQL: EXISTS and SARIF-native but LICENSE-BLOCKED for closed-source analysis.
   Safe for ELAI's own open-source output IF released open-source; BLOCKED for closed-source targets.
   Add a license-check gate in the probe step.

5. Old qossmic/deptrac: ARCHIVED Feb 2025. Use deptrac/deptrac (new canonical).

6. Fallow: JS/TS ONLY despite being Rust-native. Not a multi-language tool.
   Excellent for JS/TS; do not over-expand its scope in the plan unit description.

---

SOURCES (verified live 2026-06-11)
- github.com/fallow-rs/fallow (fetched directly)
- github.com/sverweij/dependency-cruiser (fetched directly)
- github.com/webpro-nl/knip (fetched directly)
- github.com/seddonym/import-linter (fetched + pypi.org/project/import-linter)
- github.com/jendrikseipp/vulture (fetched directly)
- github.com/regexident/cargo-modules (fetched + crates.io search)
- github.com/est31/cargo-udeps (fetched directly)
- github.com/loov/goda (fetched directly)
- github.com/ondrajz/go-callvis (fetched directly)
- github.com/TNG/ArchUnit (fetched directly)
- github.com/TNG/ArchUnitNET (fetched directly)
- github.com/deptrac/deptrac + deptrac.github.io/deptrac/formatters/ (fetched)
- github.com/vimeo/psalm (fetched directly)
- github.com/Shopify/packwerk (fetched directly)
- github.com/peripheryapp/periphery (fetched directly)
- github.com/detekt/detekt (fetched + web search)
- github.com/LemonAppDev/konsist (fetched + web search)
- github.com/ocharles/weeder (fetched directly)
- github.com/sksamuel/scapegoat (fetched directly)
- github.com/facebook/infer (fetched directly)
- github.com/terryyin/lizard (fetched directly)
- github.com/pmd/pmd (fetched + docs.pmd-code.org SARIF search)
- github.com/jQAssistant/jqassistant (fetched + web search)
- semgrep.dev/docs/semgrep-pro-vs-oss (LGPL/CE vs paid cross-file)
- semgrep.dev/docs/semgrep-appsec-platform/json-and-sarif (SARIF CE confirmed)
- docs.github.com/en/code-security CodeQL license + SARIF docs
- fallow-rs search confirming JSON/SARIF output formats
