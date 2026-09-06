# Cross-language type sharing + drift prevention for ELAI (2024-2026)

## TL;DR
- Single source of truth must be the Rust struct/enum side (where serde already lives); generate TS + Python schemas downstream. Two-way "interface-first" only pays for binary protocols (Cap'n Proto / Protobuf), which ELAI doesnt need yet.
- ts-rs v10-12 (2024-Jan 2026) closed most of the generics + dep-tracking gap that pushed early adopters to specta; for ELAI's Tauri-Svelte pair it is still the lowest-friction option, and the actual upgrade work is enforcing `export_to` + `#[ts(export)]` across all 74 SSB action types plus a CI assertion that the emitted bindings are unchanged.
- For the Rust orchestrator <-> Python sidecar pair, JSON Schema via `schemars` + `pydantic.TypeAdapter`/`datamodel-code-generator` is the realistic 2025-2026 default; Protobuf/Cap'n Proto only earns its build complexity if you also want zero-copy or polyglot RPC, neither of which ELAI has asked for.
- Drift detection wants three layers: (a) compile-time (ts-rs / specta / pyo3 stub-checks), (b) repo-time (cross_lang_diff.py already wired) extended to call the generators and `git diff --exit-code`, (c) CI-time using a `prek`/`pre-commit` hook that re-runs `cargo test -p ts-bindings`, `schemars` snapshot, and `openapi-typescript` diff and fails on drift.
- Versioning: treat the SSB action_type union + sidecar JSON wire-format as a SemVer'd protocol. Adopt the proto3-style golden rules (never change a tag/discriminator, never remove, always add + opt-in) and stamp every wire message with a `schema_version` already on disk (currently SCHEMA_VERSION 45) - that gives runtime drift detection a hook even when build-time fails.

## Key sources
URL or repo | what | takeaway.
- https://github.com/Aleph-Alpha/ts-rs | ts-rs canonical repo (v12.0.0 released 2026-01-31, MSRV 1.88) | Active; generics + serde compat supported; `#[ts(concrete(..))]` lets you monomorphise generic params; dependency export now built-in.
- https://docs.rs/ts-rs/latest/ts_rs/ | ts-rs docs.rs | Documents `TS` trait, `serde-compat` feature, multi-file export with imports.
- https://github.com/specta-rs/tauri-specta | tauri-specta v2 | v2 is still RC (2.0.0-rc.21 in Jan 2025) but adds typed events (the gap that made TauRPC interesting); requires Tauri v2.
- https://docs.rs/tauri-specta | tauri-specta docs | API surface: `Builder::new().commands(..).events(..).export()`.
- https://github.com/specta-rs/specta | specta core | Trait+runtime-info approach (vs typeshare's static-analysis); recently overhauled in early 2025 to ship multi-file/namespace output.
- https://hackmd.io/CL8nfNAIQlKCRFw4JcxN4g | "Specta v2 - Nuanced Types" | Author explains ts-rs limitation: "deals with types individually... not possible to export a type and all of the other types it depends on" - root cause for choosing specta in dependency-heavy schemas.
- https://github.com/MatsDK/TauRPC | TauRPC | Built on specta; provides fully-typed Tauri command + event IPC; types generated at runtime; v0.5.x as of 2025.
- https://github.com/GREsau/schemars | schemars | JSON Schema codegen from Rust; serde-aware; the de-facto Rust bridge to JSON Schema.
- https://github.com/oxidecomputer/typify | typify | Reverse direction: JSON Schema -> idiomatic Rust types; supports cargo typify CLI + build.rs. Closes the loop if schema is the SoT.
- https://github.com/bcherny/json-schema-to-typescript | json-schema-to-typescript | TS-side reverse: schema -> .d.ts. Pairs with schemars for Rust->TS via schema bridge.
- https://github.com/1Password/typeshare | 1Password typeshare | Multi-lang (TS, Swift, Kotlin, Scala, Go, Python experimental, Java 2025) via static analysis + `#[typeshare]` attribute; doesnt need runtime info like specta.
- https://github.com/juhaku/utoipa | utoipa | Code-first Rust -> OpenAPI; pairs with openapi-typescript / openapi-generator-cli to emit TS.
- https://github.com/hey-api/openapi-ts | openapi-ts | Production-grade OpenAPI -> TS used by Vercel/PayPal; emits SDKs, Zod schemas, TanStack Query hooks.
- https://docs.pydantic.dev/latest/internals/architecture/ | Pydantic v2 | pydantic-core is Rust; `TypeAdapter.json_schema()` emits standard JSON Schema = bridge to schemars.
- https://github.com/capnproto/capnproto-rust | capnproto-rust v0.19 (2024) | Schema-first, zero-copy, polyglot; runtime reflection added in 0.17. Heavy build but real cross-lang.
- https://protobuf.dev/reference/protobuf/edition-2024-spec/ | protobuf edition 2024 | Defines field-stability + reserved/deprecated lifecycle; the canonical "never change a tag, never remove" ruleset.
- https://semver.org/ | SemVer 2.0 | Foundation for protocol versioning.
- https://modelcontextprotocol.io/specification/versioning | MCP versioning | Date-stamped versioning instead of SemVer; concrete prior art for a JSON-wire protocol that ELAI's SSB resembles.
- https://github.com/Stranger6667/jsonschema | jsonschema (Rust) | High-perf validator; can validate sidecar payloads at orchestrator boundary.
- https://github.com/j178/prek | prek | Rust-native pre-commit replacement; `prek auto-update --check` fails CI on frozen-ref drift - useful for the regen-on-change loop.
- https://github.com/colinhacks/zod | zod v4 | Runtime TS validation; `zod-from-json-schema` + `json-schema-to-zod` close the JSON-Schema -> TS-runtime gap.
- https://github.com/PyO3/pyo3 | pyo3 v0.28 + maturin | Production cross-lang FFI (Polars, Pydantic, Ruff). For ELAI's sidecar IPC this is overkill: the sidecar already talks JSON over HTTP/MCP-style protocol, no in-process FFI.
- https://blog.dochia.dev/blog/openapi-single-source/ | OpenAPI as SoT (2025) | Schema-first artefacts: mock server, SDKs, contract tests, drift CI.
- https://totalshiftleft.ai/blog/schema-first-api-development-testing-strategy | Schema-first 2026 guide | Codifies the "spec is contract" model; contract tests block deployment on schema drift.
- https://jsontotable.org/blog/protobuf/protobuf-schema-evolution | Protobuf evolution rules | The "never change field numbers, always add, never remove, test both directions" checklist - directly portable to SSB action_type discriminators.
- https://www.archdrift.com/ | ArchDrift | TS architecture drift product; relevant for the "silent enum addition" failure mode.

## Tooling landscape
| Tool | source-of-truth | langs supported | maintenance status | citation |
| --- | --- | --- | --- | --- |
| ts-rs | Rust struct/enum (`#[derive(TS)]`) | Rust -> TS | Active; v12.0.0 (2026-01-31), MSRV 1.88 | https://github.com/Aleph-Alpha/ts-rs |
| specta + tauri-specta | Rust struct/enum (`Type` trait, runtime) | Rust -> TS (+ JSDoc) | Active; tauri-specta v2 RC as of Jan 2025 | https://github.com/specta-rs/specta , https://github.com/specta-rs/tauri-specta |
| TauRPC | Rust trait (`#[taurpc::procedures]`) | Rust -> TS | Active; v0.5.x 2025 | https://github.com/MatsDK/TauRPC |
| 1Password typeshare | Rust struct/enum (`#[typeshare]`) | Rust -> TS, Swift, Kotlin, Scala, Go, Python (exp), Java (2025) | Active; static-analysis based | https://github.com/1Password/typeshare |
| schemars | Rust struct/enum (`#[derive(JsonSchema)]`) | Rust -> JSON Schema | Active; serde-aware | https://github.com/GREsau/schemars |
| typify | JSON Schema | JSON Schema -> Rust | Active (Oxide Computer) | https://github.com/oxidecomputer/typify |
| json-schema-to-typescript | JSON Schema | JSON Schema -> TS | Active | https://github.com/bcherny/json-schema-to-typescript |
| datamodel-code-generator | JSON Schema / OpenAPI | -> Pydantic v2 models | Active; standard for Python SoT-from-schema | https://pydantic.dev/docs |
| utoipa | Rust handler macros | Rust -> OpenAPI 3 | Active | https://github.com/juhaku/utoipa |
| openapi-ts (hey-api) | OpenAPI | OpenAPI -> TS (SDKs, Zod, TanStack) | Active; used by Vercel/PayPal | https://github.com/hey-api/openapi-ts |
| openapi-generator-cli | OpenAPI | 50+ langs incl. Rust + TS | Active | https://github.com/OpenAPITools/openapi-generator |
| capnproto-rust | .capnp schema | Rust + 10+ langs incl. Python | Active; v0.19 in 2024 | https://github.com/capnproto/capnproto-rust |
| tonic + prost | .proto schema | Rust client/server; pair w/ grpc-python | Active; idiomatic Rust gRPC | https://dockyard.com/blog/2025/04/08/grpc-basics-for-rust-developers |
| zod / zod-from-json-schema | TS schema or JSON Schema | TS runtime validation | Active; Zod v4 in 2025 | https://github.com/colinhacks/zod , https://www.npmjs.com/package/zod-from-json-schema |
| prek | pre-commit hooks | Rust-native runner | Active | https://github.com/j178/prek |
| jsonschema (Rust) | JSON Schema | Rust validator | Active | https://github.com/Stranger6667/jsonschema |

## ts-rs vs specta vs schemars+typify
ELAI's current state: Rust types in `.elai_cc/crates/*` are emitted to `dashboard-v2/src/lib/types` via `ts-rs`. The cross_lang_diff.py harness already flags missing pairs. The choice tree:

- **ts-rs (recommended for ELAI today, low-risk upgrade)**. v12 supports generics with the `#[ts(concrete(T = ConcreteType))]` workaround; the historic "ts-rs deals with types individually so dep types arent exported" complaint (https://hackmd.io/CL8nfNAIQlKCRFw4JcxN4g) is largely fixed by the `dependencies` registration the derive emits + multi-file `export_to` with auto-imports (https://docs.rs/ts-rs/latest/ts_rs/). Concretely, every SSB action_type variant should derive `TS, Serialize, Deserialize` and call `T::export()` from a single `tests/bindings.rs` test that asserts file hashes - that test is the drift gate. Limitation: ts-rs is Rust->TS only; if you ever want Python or Swift, you outgrow it.

- **specta + tauri-specta (move here only if/when ELAI hits the dep-graph wall)**. Specta uses traits + runtime info so the Tauri command handler signature itself becomes the schema; tauri-specta v2 adds typed *events* (which ts-rs cant model because there's no Rust enum to derive on the event channel). v2 is still RC (2.0.0-rc.21, Jan 2025) - viable behind a pinned version but not "set and forget" until Tauri v2 + specta v2 stable. Migration cost is non-trivial because of the trait approach.

- **schemars + typify (the bridge if you need Python too)**. schemars (https://github.com/GREsau/schemars) emits standard JSON Schema; typify (https://github.com/oxidecomputer/typify) goes the other way. Pair that with `datamodel-code-generator` (Pydantic) and `json-schema-to-typescript` and the schema becomes the lingua franca. Cost: you give up ts-rs's serde-attribute fidelity (rename_all, tag/content layout) unless you carefully mirror them. Win: one Python+TS+Rust drift gate instead of two.

For ELAI specifically: keep ts-rs for the dashboard pair, add schemars for any *new* sidecar JSON message types, do not migrate everything in one go.

## Rust<->Python sidecar bridging
Verified facts: the sidecar talks Anthropic-protocol HTTP (memory: project_dola_tq_findings.md), and the orchestrator is the only Rust consumer. So in-process FFI (pyo3) is wrong tool; this is a wire-protocol problem.

Recommended pattern (citations below):
1. **JSON over HTTP with JSON Schema contract**. Schemars on Rust side (https://github.com/GREsau/schemars), Pydantic v2 on Python side (https://docs.pydantic.dev/latest/internals/architecture/). Pydantic v2 generates JSON Schema via `TypeAdapter.json_schema()`; schemars emits the same draft. CI runs both, diffs them, fails on mismatch.
2. **OpenAPI overlay** for the HTTP surface specifically. utoipa annotates the Rust handlers (https://github.com/juhaku/utoipa); hey-api/openapi-ts generates the TS dashboard client (https://github.com/hey-api/openapi-ts); openapi-python-client or datamodel-code-generator generates the sidecar's Pydantic models. That gives you mock servers + contract tests for free (https://blog.dochia.dev/blog/openapi-single-source/).
3. **Validate at the boundary** with `jsonschema` (Rust, https://github.com/Stranger6667/jsonschema) and Pydantic's parse-with-schema on the Python side. Runtime validation catches drift the build missed.

Do **not** adopt Protobuf/Cap'n Proto for this pair. Cap'n Proto's zero-copy advantage (https://github.com/capnproto/capnproto-rust) is irrelevant for HTTP-bounded JSON traffic; Protobuf's wire compactness (60-80% smaller per https://www.gravitee.io/blog/protobuf-vs-json) doesnt pay for the protoc/build complexity at ELAI's message volumes (B-37 says runtime decisions per request).

## Drift detection patterns
- **Compile-time gate (build.rs / cargo test)**. ts-rs's recommended pattern: emit bindings in a `#[test]` so `cargo test` fails on missing `#[ts(export)]`. Same for schemars: snapshot the generated schema into `tests/snapshots/*.json` and use `insta` to fail on diff. Cited: https://docs.rs/ts-rs/latest/ts_rs/ ; schemars README.
- **Repo-time gate (pre-commit)**. prek (https://github.com/j178/prek) or upstream pre-commit runs `cargo test -p bindings`, then `git diff --exit-code dashboard-v2/src/lib/types python/elai_inference/schemas`. Any diff = unstaged regen = block. Reference: https://github.com/doublify/pre-commit-rust.
- **CI gate**. ArchDrift (https://www.archdrift.com/) and openapi-diff style tools verify the protocol baseline didnt move. For ELAI, simpler: on PR, run the same regen + `git diff --exit-code`; if non-empty, fail with a "you forgot to regen bindings" message. Verified pattern in OpenAPI ecosystem (https://blog.dochia.dev/blog/openapi-single-source/).
- **Runtime gate**. The wire format already carries SCHEMA_VERSION (memory: project_b41_shipped.md, currently 45). On every message, validate against schema and refuse on version mismatch beyond the documented compat window. This is how MCP versioning works in practice (https://modelcontextprotocol.io/specification/versioning).
- **Cross-language pair detection**. cross_lang_diff.py already flags missing pairs; extend it to *call the generators* and assert no diff, not just to flag "this Rust changed but the TS sibling didnt".

## Best additions for ELAI
Ranked by ROI vs cost.

1. **Promote `ts-rs` from "we use it" to "we enforce it" with a binding-snapshot test + pre-commit regen.** | grounding: ts-rs v12 docs (https://docs.rs/ts-rs/latest/ts_rs/) + 1Password typeshare blog pattern of "snapshot the emitted file" | integration hook: extend `.claude/tools/cross_lang_diff.py` to (a) `cargo test -p <crate-with-ts-derives>` and (b) `git diff --exit-code dashboard-v2/src/lib/types`. Make it a PreToolUse hook on any Edit to a `#[derive(TS)]` file.
2. **Add `schemars` to every Rust struct that crosses the orchestrator<->sidecar HTTP boundary, and snapshot the schema in `tests/schemas/`.** | grounding: https://github.com/GREsau/schemars , https://github.com/oxidecomputer/typify | integration hook: new make target `make sidecar-schemas` that regenerates schemas + runs `datamodel-code-generator` to emit `python/elai_inference/schemas/_generated.py`. Failure = drift.
3. **Treat the 74 SSB action_types as a versioned discriminator union and apply Protobuf's "never change a tag, always add, never remove" lifecycle.** | grounding: https://protobuf.dev/reference/protobuf/edition-2024-spec/ , https://jsontotable.org/blog/protobuf/protobuf-schema-evolution | integration hook: a `crates/ssb/tests/action_type_lock.rs` golden file enumerating every action_type string + variant index; renaming a variant fails the test, additions are append-only. Already half-implemented via SCHEMA_VERSION; this formalises it.
4. **Add runtime validation at the sidecar boundary with `jsonschema` (Rust) + Pydantic's strict-mode parse on the Python side.** | grounding: https://github.com/Stranger6667/jsonschema , https://docs.pydantic.dev/latest/ | integration hook: orchestrator wraps `serde_json::from_str` with `schema.validate(&value)?` for sidecar replies in dev profile only (zero cost in release).
5. **Adopt `utoipa` if/when the sidecar grows beyond ~5 endpoints, then emit a TS dashboard client via `openapi-ts`.** | grounding: https://github.com/juhaku/utoipa , https://github.com/hey-api/openapi-ts | integration hook: replaces ad-hoc `fetch` calls in the dashboard with a generated SDK. Defer until endpoint count justifies it.
6. **Evaluate tauri-specta v2 only after Tauri v2 + specta v2 stable.** | grounding: tauri-specta v2 still RC as of Jan 2025 (https://docs.rs/tauri-specta) | integration hook: a spike branch behind a feature flag; do not migrate yet.
7. **Add a "schema golden file" CI job that runs on every PR and fails if Rust schemas, TS bindings, or Python models diff from regenerated output.** | grounding: schema-first contract testing (https://totalshiftleft.ai/blog/schema-first-api-development-testing-strategy) | integration hook: GitHub Actions workflow stage between `cargo check` and `cargo test --workspace`.

## Anti-patterns
- **"We'll keep the TS interface manually in sync."** Empirically loses against any non-trivial codebase; the 74 SSB action_types is past the threshold. (https://nordicapis.com/using-a-schema-first-design-as-your-single-source-of-truth/)
- **Renaming an enum variant or struct field without a `#[serde(rename = "<old>")]` shim.** Silent wire-format break; serde will deserialize fine until the other side updates. Identical to the Drift ORM enum-index drift failure documented at https://drift.simonbinder.eu/type_converters/.
- **Removing a discriminator tag (action_type) "because nothing uses it"**. The Protobuf rule: never remove, always reserve. (https://protobuf.dev/reference/protobuf/edition-2024-spec/)
- **Adopting Protobuf/Cap'n Proto for a JSON-over-HTTP boundary**. Build complexity (protoc, schema repo, language plugins) without the payoff of zero-copy or polyglot RPC. https://www.swatinem.de/blog/rust-grpc-capnp/ (referenced in capnproto-rust search results).
- **Using pyo3 for the orchestrator<->sidecar pair when they communicate over HTTP**. pyo3 is for in-process Python<->Rust (Polars, Pydantic-core pattern). Wrong tool for separated processes. (https://github.com/PyO3/pyo3)
- **Trusting tauri-specta v2 in production while it's still RC.** Pin versions if you must, but the RC label means breaking changes are still on the table. (https://docs.rs/tauri-specta)
- **`#[ts(type = "any")]` escape hatches**. ts-rs supports them, but each one is a future drift bug. Treat as a TODO with a tracking issue.

## Open questions
1. **Measured cost of `cargo test -p ts-bindings` on every PR for ELAI's binding count.** Hypothesis: <5s. Verify before adopting as PreToolUse hook (would block local Edit).
2. **Does adding `schemars` to ~200 Rust structs grow compile time materially?** schemars derives are non-trivial; measure on a representative crate (`crates/sidecar` or `crates/core`) before rolling out.
3. **Is Pydantic v2's JSON Schema output literally byte-identical to schemars'?** Both follow JSON Schema draft-2020-12, but `additionalProperties` defaulting and `$defs` placement may differ. Snapshot both, diff, and either (a) accept a normalisation step or (b) pick one as canonical.
4. **Can tauri-specta v2 emit the same wire format ts-rs currently emits, byte-for-byte?** If yes, migration is a one-shot replace; if no, two pages of action_types break on cutover. Verify with a dual-emit spike.
5. **What's the right SemVer mapping for SCHEMA_VERSION?** Today it monotonically increments (45). MCP uses dated versioning instead (https://modelcontextprotocol.io/specification/versioning). Decide: semver for the *Rust crate*, monotonic int for the *wire protocol*, dated tag for *MCP-style external surface* - or unify on one scheme. Measurable via compat-window tests across versions N, N-1, N-2.
