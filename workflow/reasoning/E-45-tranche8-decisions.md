E-45 Tranche 8 decisions (2026-05-31, session s-21636)

Scope: Tranche 8A (schemars + JSON Schema fixtures) + Tranche 8B
(Pydantic mirror regen via datamodel-code-generator). Tranche 8C
(ts-rs / specta) DEFERRED per D-10 DECISION-8; see
workflow/audits/E-45-ts-rs-revisit.md.

DECISION-1: schemars 1.2.1 pin (EXACT, not caret)
Grounded in: crates.io API (verified 2026-05-31 via WebFetch -- max_stable
1.2.1, MIT license, repo GREsau/schemars). Snapshot tests at
crates/orchestrator/tests/schemas_snapshot.rs assert byte-equality against
on-disk fixtures, so the derive macro output MUST stay stable across
rebuilds. Caret pin (^1.2) would let a 1.3 release regenerate the JSON
Schema with reordered keys + invalidate every snapshot.
Pinned at .elai_cc/Cargo.toml workspace.dependencies as `schemars = "=1.2.1"`.

DECISION-2: 5 boundary types scope (NOT 100+ ssb.rs payloads)
Grounded in: context-package-tranche8-E45.md Open Question 2 (compile-cost
scope-limit) + exploration tranche8-E45-wiring.md gap analysis. SsbPayload
+ DurabilityEventPayload + RuntimeHandoffEventPayload + RoutingDecision +
SkillManifestDto are dashboard-IPC, NOT sidecar wire -- adding schemars
derives to all of them would balloon compile time without buying a
sidecar-protocol invariant. Sidecar boundary IS providers::types
(MessageRequest / SystemBlock / InputMessage / ToolDefinition /
MessageResponse), per the Anthropic-shim wire shape the sidecar
implements at /v1/messages. Scope limited to those 5 + their transitive
field types (CacheControl / InputContentBlock / ToolResultContentBlock /
ToolChoice / OutputContentBlock / Usage -- which all needed the derive
to keep the parent types resolvable).

DECISION-3: Snapshot test pattern (mirror action_type_lock)
Grounded in: action_type_lock.rs pattern (CI test + #[ignore] regen
helpers) ships in this repo since 2026-03 and has caught 4+ wire-tag
drifts on the SsbActionType enum. schemas_snapshot.rs follows the same
shape: 5 CI tests (one per type) + 6 #[ignore] regen helpers (5 per-type
+ 1 regen-all). Drift produces a panic with the exact regen invocation
in the message. Canonicalization sorts JSON object keys recursively so
fixtures stay diff-stable under derive macro field-order reorderings.

DECISION-4: Regen via cargo test --ignored (no xtask)
Grounded in: Glob over .elai_cc/xtask returned empty -- no cargo xtask
infrastructure today. Adding xtask just for this would be over-scoped
(action_type_lock has used `cargo test --ignored` happily for 8 months).
Regen path: `cargo test -p elai-orchestrator --test schemas_snapshot
regen_all_schemas -- --ignored --nocapture` rewrites all 5 fixtures.
The Python tool .claude/tools/regen_sidecar_schemas.py then consumes
those fixtures via datamodel-codegen.

DECISION-5: datamodel-code-generator 0.59.0 (PyPI, MIT)
Grounded in: PyPI API verified 2026-05-31 (latest 0.59.0, no yank), GitHub
license verified via raw.githubusercontent.com/koxudaxi/
datamodel-code-generator/master/LICENSE on 2026-05-31 (MIT, Koudai Aono,
2018-2026). Tool is invoked via subprocess from
.claude/tools/regen_sidecar_schemas.py with explicit
--output-model-type pydantic_v2.BaseModel and --target-python-version
3.11 to lock the codegen target across regen runs. The tool is a
DEVELOPMENT-time dep (regen happens at author time, not at sidecar
startup); the sidecar runtime does not import datamodel-codegen.

DECISION-6: messages.py SCHEMA_VERSION bump 138 -> 191
Grounded in: exploration tranche8-E45-wiring.md flagged a 53-version
drift (Pydantic side at 138 vs Rust SCHEMA_VERSION_PUBLIC at 191 after
the cumulative bumps through Tranche 6 E-31). Pydantic mirror hadn't
been touched since E-45 land. The e45_f1 contract floor is `>= 137`
(crates/contracts/tests/e45_cross_language_drift_contract.rs:44), so
191 satisfies the floor with the same margin Rust already had. Bump is
purely a constant update -- no schema-shape changes here (schemars
fixtures cover the shape surface separately). DriftDetector's
SchemaVersionMismatch beacon will stop firing on this pair after the
bump lands.

DECISION-7: 8C (ts_bindings) DEFER per D-10 DECISION-8
Grounded in: 10x in-tree DEFERRED comment lock + 2 D-N reversal cost
(D-5 + D-10). Tranche 8 honors the existing decision and ships only 8A
+ 8B per Option C HYBRID. The DriftDetector substitute substrate
(cross_lang_drift module + 6-pair registry + cross_lang_diff.py) does
the job ts-rs would do without the 150 LOC of new crate + 30 LOC of
derive wrappers + diff overhead of committed .d.ts files. Revisit
conditions documented in workflow/audits/E-45-ts-rs-revisit.md.
