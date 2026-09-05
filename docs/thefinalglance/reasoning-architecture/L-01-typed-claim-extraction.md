# Typed claim extraction substrate (L-1)

- **ELAI ids:** L-1 / L-T1 / NN-1 / HG-3 / WAVE 1
- **Spec status (as designed):** PLANNED (`L-plan/L-1_typed_claim_extraction_substrate.md:3`). Live tree already has a **cousin fill** in `crates/orchestrator/src/claim/extract.rs` (types + structural parse + contracts); that fill is **not** the rebuild organ until the report-seam hook, G-1 taint field, and ClaimSetExtracted (or documented reuse) land as specified.
- **Crate / module (spec):** `crates/orchestrator/src/claim/mod.rs` + `crates/orchestrator/src/claim/extract.rs`; production caller is the orchestrator post-worker report/claim seam (`report_fns`), never the worker (`L-plan/L-1_typed_claim_extraction_substrate.md:35-41`).
- **Purpose of this file:** enough architecture to rebuild the off-agent typed ClaimSet intake every CGG predictor reads. Not a status ledger.

## 1. What it is

The **typed claim extraction substrate** is a deterministic, off-agent structural parser that turns **any** model output (answer / plan / patch / explanation / dependency recommendation) into a `ClaimSet = Vec<Claim{kind: ClaimKind, span, refs, taint}>`. `ClaimKind` is the closed set `{File, Symbol, Dependency, Api, Behavior, Causal, Reachability}` from the notes' claim table (LS-2). Parsing uses cited paths, symbols, package names, and signatures. It does **not** ask a model “what did you claim”. The model that emitted the output never types its own claims (wiring R8, TD-3, HG-3) (`L-plan/L-1_typed_claim_extraction_substrate.md:5`, `L-plan/L-SUMMARY_INDEX.md:11-17`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:78-81`).

## 2. Why it exists

The CGG cannot score what it cannot name. Today claim “extraction” is implicit inside the D-10 grounding-sweep, coupled to the agent’s own output and not a reusable object (`L-plan/L-1_typed_claim_extraction_substrate.md:5`). The paper and the notes agree the gate’s first job is to turn fluent output into discrete **checkable** claims; round-trip / inter-seed / do-test predictors do not run on a paragraph (`L-plan/L-1_typed_claim_extraction_substrate.md:13`, arXiv:2606.27326 [EXTERNAL/DESIGN-RATIONALE]).

If this organ is missing, L-2/L-3/L-4/L-5 have no unit of work. If it is faked (prompt “list your claims”, or the worker tagging its own spans), the self-grading loop is back: the same model that hallucinated a package also decides whether that package was a claim. A claim whose refs cannot be resolved is **not** dropped; dropping it hides the cheapest medium-risk signal L-2 reads (`L-plan/L-1_typed_claim_extraction_substrate.md:5,13,44`).

HARD BOUNDARY: L-1 only **types** and **resolves refs**. It does **not** score risk (L-2) and does **not** verify (L-3/L-4/L-5) (`L-plan/L-1_typed_claim_extraction_substrate.md:5`).

## 3. How it works (runtime)

1. **Who calls it.** The orchestrator, at the post-worker report/claim-emission seam (`report_fns` role-classification). A `#[cfg(test)]` / contracts caller **never** satisfies the production hook (R7) (`L-plan/L-1_typed_claim_extraction_substrate.md:39-41`). The worker never calls the extractor (wiring R8, stub_ban) (`L-plan/L-1_typed_claim_extraction_substrate.md:45`).
2. **Input.** Raw model output text (answer / plan / patch / explanation / dependency recommendation) plus G-1 `Provenance` of that output when the output derives from an untrusted ingest (`L-plan/L-1_typed_claim_extraction_substrate.md:5,28`).
3. **Parse (deterministic, no LLM).** Walk structural cues (`L-plan/L-1_typed_claim_extraction_substrate.md:28`):
   - cited backtick paths → `File`
   - `Symbol::method` forms → `Symbol`
   - package-name + version forms → `Dependency`
   - signature forms → `Api`
   - `"fixes/causes/improves"` verbs → `Behavior` / `Causal`
   - `"reaches/used/unused"` verbs → `Reachability`
4. **Resolve refs (deterministic).** Each `ClaimRef` is a resolved-or-unresolved pointer (path / symbol / pkg / signature). Refs that do not resolve to a real path/symbol/package are kept with `resolved=false`. Never silently discarded (`L-plan/L-1_typed_claim_extraction_substrate.md:27-28,44`).
5. **Taint (deterministic, G-1).** When the output derives from an untrusted ingest, tag `taint: Some(Provenance)` at extraction. Do not re-implement taint propagation (`L-plan/L-1_typed_claim_extraction_substrate.md:28,38`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:52-54`).
6. **Commit.** Return the `ClaimSet` for L-2 to score. Emit `ClaimSetExtracted` **or** reuse a grounding-verification row (net 0..+1, decide at edit, W-9) (`L-plan/L-1_typed_claim_extraction_substrate.md:51`, `L-plan/L-0_INDEX.md:130`).
7. **Abstain / fail-closed.** There is no “untyped and dropped” path for a structurally recognized claim. Unresolved refs stay in the set. The extractor does not refuse the worker output; it types it. `ClaimUntypable` is a journal Kind for output that yields no typed claims — exact trigger **UNCERTAIN** (§16) (`L-plan/L-1_typed_claim_extraction_substrate.md:51`).
8. **The model may generate** the output text (and, later, an optional L-22 `falsifier` field on the ClaimSet). It may **not** choose `ClaimKind`, set `resolved`, write `taint`, or invoke the extractor.

Live cousin (do not rebuild as this organ): `extract_claims(text: &str) -> ClaimSet` in `claim/extract.rs:39` is a pure structural parse with backtick classification + sentence lexicon, defaulting all refs through `UnresolvedResolver` (`claim/extract.rs:36-38`). It has **no** `taint` field, names the row `ExtractedClaim` not `Claim`, and is **not** called from `report_fns` (cli grep of `extract_claims` is empty). `claim.rs:301-311` documents the D-10 vs L-1 split: D-10 `ClaimType` types what a claim **means**; `extract` types what it **refers to**. Composed, never merged.

## 4. Types, enums, objects

Copy from `L-plan/L-1_typed_claim_extraction_substrate.md:27`. Do not invent fields.

```
enum ClaimKind {                 // closed; LS-2 table
  File,                          // check later: filesystem
  Symbol,                        // AST/LSP
  Dependency,                    // lockfile/registry
  Api,                           // typecheck/source
  Behavior,                      // test/trace
  Causal,                        // counterfactual (L-5; HG-1 labels causal ONLY after do-test)
  Reachability                   // call-graph/trace
}

ByteSpan { start, end }          // must survive streaming assembly (§16)

ClaimRef                         // resolved-or-unresolved pointer
  // plan names: path / symbol / pkg / signature
  // live cousin: { raw: String, resolved: bool }   (claim/extract.rs:15)

Claim {                          // plan name; live cousin = ExtractedClaim
  kind: ClaimKind,
  span: ByteSpan,
  refs: Vec<ClaimRef>,
  taint: Option<Provenance>,     // G-1; live cousin omits this field
}

ClaimSet                         // wraps Vec<Claim>
```

**Per-kind check method (L-1 context table, consumed by L-3/L-5, not run here)** (`L-plan/L-1_typed_claim_extraction_substrate.md:13`):

| ClaimKind | Downstream check |
|---|---|
| File | filesystem |
| Symbol | AST/LSP |
| Dependency | lockfile/registry |
| Api | typecheck/source |
| Behavior | test/trace |
| Causal | counterfactual (L-5). Extraction **typing** a sentence as Causal is not a causal **label**. HG-1 forbids calling the claim causal until an interventional pass. |
| Reachability | call-graph/trace |

**G-1 taint type (consumed, not redefined):** `Provenance { source: TextSource, trust: TrustLevel }` in `crates/safety/src/provenance.rs:131-134`. `TextSource` includes `UserPrompt`, `System`, `ToolOutput`, `Web`, `File`, `SandboxOutput`, `Memory` (`provenance.rs:34-50`). Untrusted ingest → tag at extraction.

**Do not confuse with D-10 `Claim` / `ClaimType`.** Live D-10 `ClaimType` is `{Factual, Procedural, Normative, Speculative, Hypothetical, Computational, MetaQuery}` (`claim.rs:35-43`). Plan D-10 `ClaimType` is `{Code, Math, Prose, Empirical, Definitional, Counterfactual, EthicalAesthetic}` (`D-10-mode-routing-predicate.md` §4). Neither taxonomy is L-1 `ClaimKind`. L-3 later maps kinds onto D-10 modes; that map is L-3’s, not a merge (`claim/roundtrip.rs:62` is a live cousin map).

**L-22 rider (additive, not L-1 scope):** an optional `falsifier` field on the ClaimSet, elicited at generation time (`L-plan/L-SUMMARY_INDEX.md:241-242`). Rebuild L-1 without it; L-22 adds the field.

## 5. Control flow / state machine

**Owner of the loop:** orchestrator extraction, once per worker output at the report/claim seam. No token-level FSM. No GRC states inside this organ (`L-plan/L-1_typed_claim_extraction_substrate.md:29,39-41`).

**Forced rules:**

- Extraction is **always-on intake**. No enable-flag. Behavior gates are L-7 (`L-plan/L-1_typed_claim_extraction_substrate.md:71`).
- Off-agent only. Worker-side extraction is banned (`L-plan/L-1_typed_claim_extraction_substrate.md:45`).
- Zero model calls inside `extract.rs` (`L-plan/L-1_typed_claim_extraction_substrate.md:43,58`).
- Unresolved refs stay (`resolved=false`).
- Parallel-safe with none in WAVE 1 (L-2 depends on L-1); disjoint from L-3..L-10 files (`L-plan/L-1_typed_claim_extraction_substrate.md:9`).

**Cue precedence (live cousin, not in the L-1 plan file).** Live `extract.rs:51-59` classifies backtick bodies in this order: Api (`(` AND `->`) → Symbol (`::` or ends with `()`) → Dependency (`is_dependency`) → File (`is_file`) → skip. Sentence lexicon (`extract.rs:94-96`): Reachability words beat Causal words beat Behavior words. The plan says “seed from the notes' table, OBSERVE-first… Decide the cue lexicon at authoring” (`L-plan/L-1_typed_claim_extraction_substrate.md:96`). Rebuild: pin a lexicon in `workflow/reasoning/L-1-decisions.md`; do not treat the live order as the spec unless that decisions file records it. That file is **absent** at architecture time.

## 6. Grounding, receipts, gates

This organ **does not verify**. A well-formed File claim with `resolved=true` is not a receipt that the file contents support the sentence. Resolution is existence (path exists / symbol in index / package in lockfile), not truth.

**Selector hierarchy (what L-1 is allowed to assert):**

| Assertion | Allowed? |
|---|---|
| “this span is kind K by structural cue” | yes, extractor |
| “this ref exists in workspace/lockfile/index” | yes, resolver |
| “this claim is supported / causal / low-risk” | **no** — L-3/L-5/L-2 |
| “the model said it cited a real file” | **no** — never a confidence signal |

**Never allowed as a confidence signal / typer:** LLM self-listing of claims, logprobs, window fill, D-10 mode as a substitute for `ClaimKind`, worker-attached tags (`L-plan/L-1_typed_claim_extraction_substrate.md:5,13,43-45`, `L-plan/L-0_INDEX.md:47,162`).

**GD-1:** do **not** re-implement D-10 grounding methods (FILE/SOURCE/TEST/CosmosCheck). Lift the claim object **out** of the sweep (`L-plan/L-1_typed_claim_extraction_substrate.md:5`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:21-24`).

**HG-1 at this seam:** typing `ClaimKind::Causal` is a **cue class**, not an interventional label. Only L-5’s executable pass may label a claim causal (`L-plan/_grounding/L-CGG-GROUNDING-MAP.md:99-102`).

## 7. Budgets, limits, ceilings

L-1 specifies **no** per-topic GRC budget, no token budget, no N-resample. Effort estimate is ~1 session, in-process, no network (`L-plan/L-1_typed_claim_extraction_substrate.md:7`).

| Knob | Value | Scope |
|---|---|---|
| types | ~40 LOC | `claim/mod.rs` (plan) |
| extractor | ~90 LOC | `claim/extract.rs` |
| seam hook | ~25 LOC | report/claim seam |
| contract tests | ~70 LOC | `l1_claim_extraction_contract.rs` |
| SCHEMA | prefer-reuse grounding-verification **or** +1 `ClaimSetExtracted` | W-9; baseline at L-0 authoring ~268/577, **re-grep** (`L-plan/L-0_INDEX.md:129-130`) |
| live SCHEMA | derived `SCHEMA_VERSION_DDL_EPOCH + len` (`ssb.rs:1675-1689`) | do not rewind; emit the payload shape |
| resolver (live cousin, unspecified in L-1 file) | `PaperStoreRefResolver` caps 500 md files, 256 KiB each (`claim/resolve.rs:147-165`) | not in L-1 plan; L-9 consumes papers |

Overflow of spans / unbounded claim count: **UNCERTAIN** (§16).

## 8. Roles and neighbors

L-1 is **role-agnostic**. Any backend’s worker output (B-37) is intake (HG-3) (`L-plan/L-1_typed_claim_extraction_substrate.md:63`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:106-108`).

**Consumes:** worker output at `report_fns`; D-10 `select_grounding_mode` as the sweep it lifts the object out of (compose, never re-implement); G-1 `Provenance` (`L-plan/L-1_typed_claim_extraction_substrate.md:36-38`). Live `select_grounding_mode` Constructive branch is `grounding_mode.rs:157-166` (plan pin `grounding_mode.rs:157`); that live enum is the D-10 cousin — do not fork a second selector.

**Produces:** `ClaimSet` for L-2/L-3/L-4/L-5/L-8; optional `ClaimSetExtracted` SSB row.

**Must not fork:** a worker-side typer; a prompt “extract claims”; merging `ClaimKind` into D-10 `ClaimType`; dropping unresolved refs; scoring inside extract.

**Frontend:** a claims tile on the trust panel (decide at edit, W-4) **or** justified N/A — extraction feeds L-2’s risk tile (`L-plan/L-1_typed_claim_extraction_substrate.md:47-48`).

## 9. Config / flags / defaults

- **No enable-flag.** Extraction is always-on intake. The behavior gate is L-7 (`L-plan/L-1_typed_claim_extraction_substrate.md:71`).
- Kill-switch for the whole CGG is later (`ELAI_HALLUCINATION_GATE_DISABLE` on L-7, W-2). L-1 stays on so L-7 can still observe (`L-plan/L-SUMMARY_INDEX.md:57`).
- Cue lexicon: observe-first to bound mis-typing before any downstream gate blocks (GD-4) (`L-plan/L-1_typed_claim_extraction_substrate.md:96`).
- Decision doc required: `workflow/reasoning/L-1-decisions.md` (`L-plan/L-1_typed_claim_extraction_substrate.md:33`). **Absent** at architecture time.
- Hash: blake3 only. No `.unwrap()` / `.expect()` on prod (`L-plan/L-1_typed_claim_extraction_substrate.md:46,93`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/claim/mod.rs` | `ClaimKind` / `Claim` / `ClaimSet` (`L-plan/L-1_typed_claim_extraction_substrate.md:35`) |
| `crates/orchestrator/src/claim/extract.rs` | deterministic structural extractor, off-agent (`:36`) |
| report/claim seam (`report_fns`) | production hook (~25 LOC) (`:86`) |
| `crates/contracts/tests/l1_claim_extraction_contract.rs` | named contract (`:87`) |
| `workflow/reasoning/L-1-decisions.md` | cue lexicon + W-4 tile vs N/A (`:88`) |
| G-1 `Provenance` | `crates/safety/src/provenance.rs:131` |
| D-10 lift point | `grounding_mode.rs:157` (plan pin) |

**Live neighbors (implementation truth, not the rebuild names):** submodule declared at `claim.rs:301-311` (`pub mod extract`); types live in `extract.rs` not `claim/mod.rs` (that file does not exist — `claim.rs` is the module root). `pub use extract::{extract_claims, ClaimKind, ClaimSet, ExtractedClaim}` (`claim.rs:311`). Resolver extras `WorkspaceRefResolver` / `PaperStoreRefResolver` in `claim/resolve.rs` (not in the L-1 file list). Contract `crates/contracts/tests/l1_claim_extraction_contract.rs` exists and asserts File / invented-package Dependency `resolved=false` / Causal vs Behavior / zero-handle function pointer / traversal refusal / lockfile package resolve.

## 11. Journal / SSB events

Plan (`L-plan/L-1_typed_claim_extraction_substrate.md:51`, `L-plan/L-0_INDEX.md:130`):

```
ClaimSetExtracted   // Pattern-2; OR reuse a grounding-verification row
  Kind: ClaimTyped | ClaimUntypable
```

Net 0..+1 from the **live** baseline at edit (W-9). L-0 authoring snapshot was ~268/577; live `SCHEMA_VERSION` is derived (`ssb.rs:1684`). Rebuild: **emit this Kind pair or document the reused family**; do not rewind a schema number.

`ClaimSetExtracted` / `ClaimTyped` / `ClaimUntypable` **do not appear** in the live tree at architecture time (grep empty). Live extraction currently journals nothing of its own.

Round-trip / GD-2: the ClaimSet is recomputable from the worker output under `ELAI_SEED` (`L-plan/L-1_typed_claim_extraction_substrate.md:78`). Payload fields beyond Kind are unspecified (§16).

## 12. Tests and observables

Named contract (`L-plan/L-1_typed_claim_extraction_substrate.md:81`):

| Id | Assert |
|---|---|
| (a) | file-claim typed `File` with a resolved ref when the path exists |
| (b) | invented-package claim typed `Dependency` with `resolved=false` (not dropped) |
| (c) | `"this fixes X"` typed `Behavior`/`Causal` |
| (d) | extraction makes zero model calls (mock backend asserts no invocation) |
| (e) | untrusted-sourced claim carries a G-1 taint tag |
| (f) | extraction runs off-agent (worker mock never calls it) |

Wiring checks (`L-plan/L-1_typed_claim_extraction_substrate.md:55-59`):

```
rg "enum ClaimKind|struct ClaimSet" …/claim/mod.rs
rg -c "llm|generate|complete\(" …/claim/extract.rs   # MUST be 0
test -f workflow/reasoning/L-1-decisions.md
```

**observable_effect** (`L-plan/L-1_typed_claim_extraction_substrate.md:49-52`, `L-plan/L-SUMMARY_INDEX.md:17`): a model output with a file-claim + an invented-package-claim yields a ClaimSet with **both** typed. Negative: a claim referencing a nonexistent file is typed File + flagged refs-unresolved (never silently dropped). Off-agent: extractor runs in the orchestrator, not the worker.

**W-1:** done = the live report seam produces a typed ClaimSet on a real worker output (not cargo-green) (`L-plan/L-1_typed_claim_extraction_substrate.md:70`).

Adversarial: worker-side extraction; LLM typer; drop unresolved; merge with D-10 `ClaimType`; `.unwrap()` on prod; sha2 hashes.

Live contract extras (not in the L-1 bullet list): `extractor_is_plain_synchronous_function` (`fn(&str) -> ClaimSet` witness), workspace path real-vs-ghost, `../../etc/passwd` unresolved, lockfile `serde@1.0.0` vs invented package (`l1_claim_extraction_contract.rs:29-76`). Those are compatible with (a)(b)(d); they do **not** cover (e) taint or (f) worker-mock off-agent.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Closed `ClaimKind` with exactly `{File, Symbol, Dependency, Api, Behavior, Causal, Reachability}`.
- `Claim` carries `kind`, `ByteSpan`, `refs`, and `taint: Option<Provenance>`.
- `ClaimSet` is a `Vec` of those claims; unresolved refs are kept with `resolved=false`.
- Extractor is a **deterministic structural parse** with **zero** model calls (rg of llm/generate/complete = 0).
- Extractor runs **off-agent** at the real report/claim seam (R7); the emitting worker never types claims (R8).
- Untrusted ingest tags G-1 `Provenance` at extraction.
- Invented-package claim is typed `Dependency` + unresolved, not dropped, not typed as supported.
- D-10 grounding methods are **not** re-implemented here.
- `ClaimKind::Causal` is a cue class only; interventional “causal” is L-5/HG-1.
- L-1-decisions.md exists. Contract tests (a)–(f) exist and pass.
- No enable-flag on extraction.

A prompt “please list your claims”, a D-10 `ClaimType` rename, or the live `ExtractedClaim` (no taint, no report-seam hook, no SSB Kind) **alone** is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Risk scoring (L-2), round-trip verify (L-3), semantic entropy (L-4), do-test (L-5) (`L-plan/L-1_typed_claim_extraction_substrate.md:5`).
- Re-implementing D-10/D-11 grounding methods (GD-1).
- Model call to type claims (TD-3) (`L-plan/L-1_typed_claim_extraction_substrate.md:43`).
- Dropping unresolved-ref claims (`:44`).
- Worker-side extraction (`:45`).
- Merging L-1 `ClaimKind` with D-10 `ClaimType` (`claim.rs:301-303`).
- Calling a File/import-edge claim “causal” (HG-1 / LS-2 fake-precision) (`L-plan/_grounding/L-CGG-GROUNDING-MAP.md:99-102`).
- LLM self-eval as a typer; logprobs as a required input (HG-3).
- Enable-flag that can skip intake (`L-plan/L-1_typed_claim_extraction_substrate.md:71`).
- `sha2` / `blake2`; `.unwrap()` / `.expect()` on prod (`:46`).

## 15. Source citations

Primary: `docs/thefinalglance/L-plan/L-1_typed_claim_extraction_substrate.md` (full file). Index: `L-plan/L-SUMMARY_INDEX.md:11-17`. Doctrine: `L-plan/L-0_INDEX.md:45-54,87-88,130`. Grounding map: `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:21-24,52-54,78-81,99-111`. Quality-bar neighbors: `reasoning-architecture/D-10-mode-routing-predicate.md` (D-10 ClaimType cousin). Live code (implementation truth, not spec): `crates/orchestrator/src/claim.rs:301-311`, `claim/extract.rs`, `claim/resolve.rs`, `crates/contracts/tests/l1_claim_extraction_contract.rs`, `crates/safety/src/provenance.rs:131-134`, `grounding_mode.rs:157-166`. Paper: arXiv:2606.27326 [EXTERNAL/DESIGN-RATIONALE] (`L-plan/L-0_INDEX.md:11,168`).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Exact cue lexicon and Behavior vs Causal vs Reachability verb lists. Plan defers to authoring + L-1-decisions.md (`L-plan/L-1_typed_claim_extraction_substrate.md:96`). Live `extract.rs:80-96` is one fill, not blessed by that missing decisions file.
- Span fidelity across streaming assembly (`:97`). `ByteSpan` units (bytes vs chars) unspecified; live uses byte/`char_indices` mix (`extract.rs:43-76`).
- `Claim` vs live `ExtractedClaim` naming; whether `claim/mod.rs` must exist as a file or `claim.rs` as module root is acceptable.
- `ClaimRef` field list beyond “path/symbol/pkg/signature” and live `{raw, resolved}`.
- When to emit `ClaimUntypable` vs empty `ClaimSet` vs skip the row.
- Payload fields of `ClaimSetExtracted` besides Pattern-2 Kind.
- Prefer-reuse grounding-verification vs new family (explicitly “decide at edit”).
- Frontend tile vs N/A (W-4, decide at edit).
- Max claims per output / span overflow policy.
- How G-1 “4 ingest sites” bind onto `TextSource` variants at this seam (map names four sites; `provenance.rs` has more variants).
- Whether `extract_claims_with(resolver)` / `WorkspaceRefResolver` / paper-store resolver are in-scope for L-1 or later L-3/L-9 fills (L-1 file list does not name `resolve.rs`).
- `"this fixes X"` → Behavior **or** Causal (test (c) allows both; live sentence lexicon maps `fixes` to Behavior and `causes` to Causal).
- L-22 optional `falsifier` field interaction with L-1 span/refs (additive later).
