DIGEST: fallow-multilang-tools
SOURCE: e888cc0e-79b6-4d8a-bcdf-e0044c0f5367.txt

TOPIC: Open-source codebase-intelligence tools "similar to Fallow" (whole-project dead code, dependency graphs, architecture constraints, cross-file analysis, complexity/duplication metrics, workflow integration) across the top-20 programming languages.

REPORT VERDICTS:
- No single OSS tool reproduces Fallow's exact mix (project-wide dead-code + duplication + architecture boundaries + dependency analysis + optional runtime evidence) across all major languages. Best OSS answer = a STACK: one structure/dependency tool + one dead-code/reachability tool + one metrics/semantic analyzer per language.
- Strongest broad-platform foundation: combination of CodeQL + Semgrep CE + PMD/CPD + Lizard.
- Strategic caveat (adopt-with-care): CodeQL CLI/engine in separate repo, separately licensed, closed-source analysis may need commercial license; Semgrep CE's deeper cross-file/advanced security analysis is platform-only/proprietary — both have capability/licensing boundaries vs Fallow's open-workflow differentiator.
- Weakest categories relative to Fallow: cross-language whole-project architectural intelligence, and OSS static-plus-runtime correlation (esp. Dart, Shell, Perl, Scala, Lua, R).

FALLOW (the audited tool itself — everything the report says):
Fallow | repo fallow-rs/fallow | langs JavaScript, TypeScript | category whole-project intelligence; static analysis w/ optional runtime evidence | license MIT | features: unused code, duplication, circular dependencies, complexity hotspots, architecture boundaries; optional PAID runtime evidence from production traffic | Rust-native CLI, release-heavy | maturity 3.5k stars, 192 releases, latest June 10 2026 | caveat: runtime layer is optional paid functionality so the full "static + runtime" story is NOT wholly open-source | report's role: BASELINE, among closest OSS to target feature-mix but still JS/TS-specific.

CANDIDATES BY ECOSYSTEM (name | repo/url | report claim | license/stars):

CROSS-LANGUAGE LEADERS:
CodeQL | github/codeql | semantic query engine; stronger than Fallow on semantic/security querying, weaker on dead-code/duplication/architecture-drift workflow | MIT (repo); CLI separately licensed; 9.7k stars 169 tags
Semgrep Community Edition | semgrep/semgrep | 30+ langs lightweight rule-based static analysis; broader than Fallow in reach, weaker in CE on project-graph + deep cross-file | LGPL-2.1; cross-file/data-flow are paid
PMD / CPD | pmd/pmd | 16-lang rule-based static analysis (400+ rules) + CPD duplication across ~25 langs (Dart/Go/Lua/ObjC/Perl/PHP/PL-SQL/Scala/Swift/Python/Ruby/Rust/TS...); best OSS complement for duplication+cross-lang QA, not an architecture-graph platform | BSD-style; 5.4k stars 116 releases latest May 29 2026
Lizard | terryyin/lizard | very broad langs (C/C++/Go/Kotlin/Lua/ObjC/Perl/PHP/PL-SQL/Python/R/Ruby/Rust/Scala/Swift/TS); cyclomatic complexity, token/param counts, CPD; narrower than Fallow on architecture; measures how code "looks" not exact semantics | MIT
Infer | facebook/infer | Java/C/C++/ObjC; correctness/concurrency/cost (deadlock, data-race, Big-O cost); stronger than Fallow on correctness, weaker on dead-code/dup/architecture | MIT; 15.6k stars 31 releases latest May 12 2026; Java support may need GPL components

JAVASCRIPT / TYPESCRIPT (best stack: Fallow + dependency-cruiser + Knip):
dependency-cruiser | sverweij/dependency-cruiser | JS/TS/CoffeeScript dependency graph + architecture rules + circular-dep detection; TS/JSX/Vue/Svelte; closest OSS analogue to Fallow's architecture/dependency side; weaker on duplication/runtime | MIT; 6.7k stars 386 releases latest May 29 2026
Knip | webpro-nl/knip | JS/TS unused files/dependencies/exports; very close to Fallow on unused-asset cleanup; weaker on graphs/duplication | ISC; 11.4k stars 590 releases latest June 6 2026

PYTHON (best stack: Import Linter + Vulture + Lizard):
Import Linter | seddonym/import-linter | enforces import constraints between modules + browser UI; much stronger than Fallow on explicit Python architecture contracts; weaker on dead-code/dup | BSD-2-Clause; 1.1k stars 47 tags
Vulture | jendrikseipp/vulture | Python dead-code; whitelist support; closest Python analogue to Fallow dead-code; weaker on graphs/dup/architecture; needs whitelist for reflective/framework code | MIT; 4.7k stars 22 releases latest Mar 25 2026

JAVA (best stack: jQAssistant + ArchUnit + PMD/CPD + Infer):
jQAssistant | jQAssistant/jqassistant | graph-based architecture/structure analysis + plugins; closer than Fallow to "structure as queryable graph"; more setup-heavy | GPL-3.0; 280 stars 19 releases latest Feb 17 2026; own integration tests 30+ min whole-project run
ArchUnit | TNG/archUnit | architecture tests via bytecode in unit tests (package/class deps, layers, slices, cycles); stronger than Fallow on test-enforced contracts; weaker on dead-code/dup/dashboards | Apache-2.0

C / C++ (best stack: Infer + OCLint + Lizard + Cppcheck):
OCLint | oclint/oclint | C/C++/ObjC static quality+defect issues beyond compiler; closer to Fallow static-quality for ObjC/C; weaker on dependency intelligence; high memory + macro false positives on large projects | BSD-3-Clause; 3.9k stars latest 26.02 Feb 23 2026
Cppcheck | no-url (cited) | classic GPLv3 static analyzer | (stack member)

C# (best stack: ArchUnitNET + PMD/CPD + Lizard):
ArchUnitNET | TNG/ArchUnitNET | C# architecture tests via bytecode, rules as test code (classes/members/interfaces/assemblies); best Fallow-adjacent for .NET architecture-drift; weaker on dead-code/metrics | Apache-2.0; ~1.3k stars release 0.13.3 Mar 5 2026

GO (best stack: goda + go-callvis + Staticcheck):
go-callvis | ondrajz/go-callvis | Go call-graph visualizer (packages/types/internal-external/concurrency/deferred edges, Graphviz); stronger than Fallow on visual call-flow; weaker on dead-code/rules/dup; friction w/ submodules+install | MIT; 6.5k stars 13 releases latest Jan 2 2025
goda | loov/goda | Go dependency toolkit (graphs, dead-code-elimination queries, weight/binary-size diff, advanced graph queries); one of strongest whole-project Go dependency tools | MIT; 1.7k stars 23 releases latest Feb 25 2026
Staticcheck | no-url (cited) | advanced correctness/performance checks | (stack member)

RUST (best stack: cargo-modules + cargo-udeps + rust-code-analysis):
cargo-udeps | est31/cargo-udeps | detects unused Cargo dependencies; excellent for "unused dependency" slice, much narrower overall; needs Rust nightly to run | MIT or Apache-2.0; 2.1k stars 49 releases latest Apr 29 2026
cargo-modules | regexident/cargo-modules | prints structure trees, internal dependency graphs, detects orphaned/unlinked source files; one of strongest Rust answers to Fallow architectural-visibility; weaker on metrics/runtime | MPL-2.0; 1.2k stars 104 tags
rust-code-analysis | no-url (cited) | metrics | (stack member)

PHP (best stack: Deptrac + Psalm + PhpMetrics):
Deptrac | deptrac/deptrac | architecture layers + visualize deps + enforce layer/class rules; very close to Fallow architecture-boundary use case, PHP-specific | MIT; 3k stars 62 releases latest May 13 2026
Psalm | vimeo/psalm | error finding + taint analysis + type inference + security rules; stronger than Fallow on semantic/type, weaker on architecture graphing/dup; memory problems on large GitHub Actions | MIT; 5.9k stars
PhpMetrics | no-url (cited) | project dashboards/metrics | (stack member)

RUBY (best stack: Packwerk + debride + RubyCritic):
Packwerk | Shopify/packwerk | enforces package boundaries + modularizes Rails; closest Ruby analogue to Fallow architecture-drift; relies on Zeitwerk constant-loading, ignores most method-call/data-flow | MIT; 1.9k stars 27 releases latest May 6 2026
debride | no-url (cited) | finds dead methods | (stack member)
RubyCritic | no-url (cited) | quality/complexity reporting | (stack member)

SWIFT (best stack: Periphery + PMD/CPD + Lizard):
Periphery | peripheryapp/periphery | finds unused declarations incl advanced cases; closest Swift analogue to Fallow dead-code; documented Swift-index bugs + false-positives in large mixed Swift/ObjC | MIT; 6.1k stars 85 releases latest Apr 26 2026

KOTLIN (best stack: Konsist + detekt + PMD/CPD):
detekt | detekt/detekt | Kotlin static analysis/code-smells; Gradle/Bazel/GitHub-Actions; useful counterpart to Fallow quality/smell; weaker on project graphing | Apache-2.0; 7k stars 120 releases latest Feb 21 2025
Konsist | LemonAppDev/konsist | architecture-consistency tests (JUnit/Kotest); very strong for Kotlin architecture rule enforcement; weaker on dead-code/metrics/dup | Apache-2.0; 1.7k stars

SCALA (best stack: Scapegoat + Scalafix + sbt-dependency-graph):
Scapegoat | scapegoat-scala/scapegoat | compiler-plugin static analysis for suspicious Scala + SBT; better than Fallow for compiler-integrated inspections; weaker on graphs/dead-code-reachability | Apache-2.0; 557 stars 48 releases latest Jun 3 2026
Scalafix | no-url (cited) | semantically-aware rewrites | (stack member)
sbt-dependency-graph | no-url (cited) | still useful but archived/stale | (stack member)

OBJECTIVE-C (best stack: Infer + OCLint + Lizard) — covered above.

DART (best stack: dependency_validator + pana):
dependency_validator | Workiva/dependency_validator | finds missing/under-promoted/over-promoted/unused deps; supports pub workspaces/monorepos; best OSS Dart dependency-hygiene, narrower than Fallow | open-source; 66 stars 41 releases latest Mar 23 2026
pana | dart-lang/pana | Dart/Flutter package analysis, scored reports + JSON + doc-coverage; useful package-health complement, NOT designed for CI | BSD-3-Clause; 222 stars

SQL / PL-SQL (best stack: SchemaCrawler + PMD/PLSQL + SQLCheck):
SchemaCrawler | schemacrawler/schemacrawler | schema discovery, diagrams, scripting, lint via JDBC metadata; best OSS for whole-system structure visibility in DB layer (schema/dependency level, not source-level dead code) | open-source (multiple licenses); 1.8k stars 240 releases latest May 30 2026
SQLCheck | no-url (cited) | query anti-patterns across major SQL dialects | (stack member)

SHELL / BASH (best stack: ShellCheck):
ShellCheck | koalaman/shellcheck | semantic problems + portability traps beyond syntax; clear OSS quality baseline for shell, less "whole-project" than Fallow | GPLv3; 39.6k stars 30 releases latest Aug 4 2025

PERL (best stack: Perl::Critic + Devel::Cover + B::Xref):
Perl::Critic | Perl-Critic/Perl-Critic | policy-based static analysis + Test::Perl::Critic integration; better for policy-driven Perl QA, lacks whole-project graph/dup/runtime | same terms as Perl; 203 stars 118 tags
Devel::Cover | no-url (cited) | runtime coverage metrics | (stack member)
B::Xref | no-url (cited) | cross-reference reports | (stack member)

HASKELL (best stack: Weeder + Stan):
Weeder | ocharles/weeder | whole-program dead-code via GHC HIE files w/ symbol resolution+types, builds dependency graphs, finds unreachable code across modules; one of best true Fallow-like dead-code tools outside JS/TS; Template Haskell / type-family false positives | BSD-3-Clause; 186 stars 17 releases latest Aug 1 2025
Stan | kowainik/stan | Haskell inspections for bug-risk/performance/best-practices; not a graph/architecture platform; slow cadence | MPL-2.0; 593 stars 4 releases latest Oct 15 2023

LUA (best stack: lua-language-server + Luacheck):
lua-language-server | LuaLS/lua-language-server | LSP workspace-aware navigation; valuable where editor-level intelligence > batch reports; not Fallow-equivalent; ~1M VS Code installs | MIT; 4.3k stars
Luacheck | mpeterv/luacheck | undefined globals, unused vars/values, uninitialized-var access, unreachable code; best OSS Lua analyzer, shallower than Fallow; latest GitHub release Sept 2018, slow cadence | MIT; 2k stars 37 releases

R (best stack: pkgnet + goodpractice + cyclocomp):
pkgnet | uptake/pkgnet | graph representations of an R package + deps to support testing priorities + structural exploration; one of best Fallow-like graph tools in R | BSD-3-Clause; 169 stars 11 releases latest Jan 27 2026
goodpractice | no-url (cited) | package-level QA | (stack member)
cyclocomp | no-url (cited) | cyclomatic complexity | (stack member)

ATTACK-THE-DESIGN (report's criticisms of relying on these tools as Fallow-equivalents):
- Some broadest tools (CodeQL, Semgrep CE) are NOT equivalent in openness/capability to their marketing surface — CodeQL CLI separately licensed (commercial for closed-source); Semgrep deep cross-file analysis is paid platform-only. The differentiator vs Fallow is how much value is available in the open-source developer workflow, not raw breadth.
- No single OSS tool covers Fallow's full feature mix; cross-language whole-project architectural intelligence and OSS static-plus-runtime correlation remain the weakest OSS categories (Dart/Shell/Perl/Scala/Lua/R most fragmented + slowest-maintained).
- Maturity-tradeoff cases: pana not designed for CI; sbt-dependency-graph archived; Luacheck release cadence old (2018); detekt latest release Feb 2025; go-callvis latest Jan 2025; Stan latest Oct 2023.

NUMBERS (figure | claimed source = report's GitHub snapshot reads):
- Star/release counts transcribed inline per tool above (Fallow 3.5k/192; Knip 11.4k/590; CodeQL 9.7k stars/169 tags; Infer 15.6k/31; ShellCheck 39.6k/30; dependency-cruiser 6.7k/386; PMD 5.4k/116; etc).
- jQAssistant whole-project integration-test run 30+ min.
- lua-language-server ~1 million VS Code installs.
- PMD: 16 languages rule-based; CPD ~25 languages duplication. Semgrep CE 30+ languages. Codebase-graph note: n/a (this report).

NOTE: report relied on a GitHub static-page snapshot; contributor counts not consistently visible, so it used stars/releases/issues/tags as maturity signals. Several "stack member" tools (Cppcheck, Staticcheck, rust-code-analysis, PhpMetrics, debride, RubyCritic, Scalafix, sbt-dependency-graph, SQLCheck, Devel::Cover, B::Xref, goodpractice, cyclocomp) are named in the coverage table but given no-url and only the one-line stack rationale.
