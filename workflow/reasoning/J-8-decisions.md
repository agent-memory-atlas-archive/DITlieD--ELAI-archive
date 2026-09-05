# J-8 decisions — vlm perception research upgrade (Tier-1 landed, Tier-2 owner-gated)

## Sweet-spot table (VR-1, T1 @3e05c50)

Shipped rows in `data/perception/sweet_spots.toml`:

| model_key | longest_side_px | native_res |
|-----------|----------------|------------|
| claude    | 1568           | false      |
| gpt       | 2048           | false      |
| qwen_vl   | 1280           | false      |
| gemini    | 0              | true       |

Values are ELAI-measured, not corpus figures.  The toml header carries
HD-1/EXTERNAL SRC-3 design-rationale tags: every shipped number is
ELAI-measured and must be re-verified on model swaps (G-10 canary is the
runtime sibling).  Provider-doc re-check posture: re-verify per row at
model release boundaries; the toml is the single source, code carries
zero magic px numbers (stub_ban enforced).

Gemini native_res=true is wired via string-key API only (no Gemini arm
in TargetModel enum); intentional minimal blast radius.  The
sweet_spot::longest_side_px and sweet_spot::native_res fns are consumed
by preprocess() at preprocess.rs ~175-178, replacing the prior hardcoded
match arms.

Falsification: T1-falsification.md — RED (delete sweet_spot wiring,
restore hardcoded zeros) → contract FAIL; GREEN (restore) → PASS.

Named uncovered: live sidecar resize still WIRE-DARK[J-1]
(DegradedPassthrough); router does not yet consume native_res for
passthrough branching (T4 residual).

## Schema v2 (VR-2, T2 @717027e)

Shape: call-1 JSON schema gains a REQUIRED coords contract —
[ymin,xmin,ymax,xmax] normalized 0-1000.  SCAFFOLD_COORD_NORMALIZATION_MAX
= 1000 (existing constant at scaffold.rs:28).  apply_schema_v2 is called
inside scaffold_with_call1; a raw-pixel emission (any value >1000 or
schema-tagged px) FAILS validation, re-prompts once, then types a
refusal.  Instruction-before-image stays asserted.

Falsification: T2-falsification.md — RED (replace apply_schema_v2 match
arm with unconditional DegradedPassthrough) → 3 contract cases FAIL
(accepted, reprompt_once, typed_refusal); GREEN (restore) → 17 passed.

Named uncovered: live production_agent_rpc still calls scaffold() →
scaffold_with_call1(input, None, 0); schema v2 gate is not on the live
infer path until T5 feeds call-1 elements.  The contract falsifies the
fn API, not the live e2e path.

## Multi-image + micro-text routing (VR-4/VR-5, T4 @6f7de39)

RouterInput gains image_count field.  N>=MULTI_IMAGE_MIN_COUNT routes to
RouteToTiledComposite; micro-text routes to tiling then
confidence-gated typed refusal.  Both emit their routing Kind with GD-5
residue note in the payload.

Falsification: T4-falsification.md — RED (disable multi-image arm via
`_ if false && ...`) → 2 contract cases FAIL
(multi_image_normal_routes_to_tiled_composite,
counting_threshold_with_and_without_multi_image); GREEN (restore) → j8
24 passed, j1 23 passed (+2 ignored).

Named uncovered: live InferRequest frame is text-only — production sets
image_count to 0 or 1 only; tiled-composite arm is not exercised on the
live path until multi-image frames land.  B-24 Mode B / live tiler
dispatch not wired — Kind+residue SSB only.  Production SSB payload
fields (kind, residue) have no contract assertion this slice.

## Prompt-shape library (VR-10/VR-6, T7 @5436f3c)

prompt_shapes.rs carries the component-inventory template
{type, verbatim text, hierarchy, layout cues} and the MoAI citation
string (arXiv:2403.07508).  Exported for H-14's future consumption;
H-14's file untouched.

Falsification: T7-falsification.md — RED (delete arXiv:2403.07508 from
MOAI_CITATION const) → moai_citation_contains_arxiv_id FAIL; GREEN
(restore) → j8 34 passed, j1 23 passed (+2 ignored).

Named uncovered: no production call site (scaffold / router /
handle_infer_request / H-14) consumes component_inventory_template or
moai_citation this slice.  Pub fns are WIRE-DARK[J-8] (allowlist-backed,
see R8 below).

## Vision ladder catalog seed (VR-13/VR-11, T8 @cc8f985)

`data/perception/vision_ladder.toml` ships 8 rows for J-9 seeding:

| tier_key | model_id | mmmu_design | source_tag |
|----------|----------|-------------|------------|
| sub_8gb | Phi-4-Multimodal | 55.1 | [DESIGN] VR-13 sub-6B/8GB |
| vram_24gb_ocr_basic | Qwen2.5-VL-7B | 0.0 | [DESIGN] VR-13 basic OCR |
| vram_24gb_qat | Gemma-3-27B-QAT | 64.9 | [DESIGN] VR-13 QAT |
| vram_48gb_32b | Qwen2.5-VL-32B | 0.0 | [DESIGN] VR-13 32B |
| vram_48gb_72b | Qwen2.5-VL-72B | 70.2 | [DESIGN] VR-13 72B |
| vram_48gb_internvl | InternVL3-78B | 72.2 | [DESIGN] VR-13 InternVL3 |
| cloud | anthropic-cloud-vision | 0.0 | [DESIGN] VR-13 cloud |
| fastvlm_apple | FastVLM | 0.0 | [DESIGN] VR-11 Apple-silicon |

ALL figures are [DESIGN] / HD-1 — design rationale only, re-verify at
eval.  No figure appears as an ELAI claim anywhere.

Falsification: T8-falsification.md — RED (delete FastVLM block from
toml) → fastvlm_row_present_and_tagged FAIL; GREEN (restore) → j8 31
passed, j1 23 passed (+2 ignored).

Named uncovered: J-9 vision/OCR lane does not yet consume
vision_ladder.toml (seed handoff only).  J-1 leg-4 probe-model selection
does not call local_rows_fitting_vram yet.  No live probe routing /
E-51 MLX FastVLM path claimed.

## JC-11 prefer-reuse SSB decision (T9 @a154ade)

Decision: extend PerceptionScaffoldEventKind with new arms
(SchemaApplied, ResizeRuleApplied, RouteRuleApplied, EvalScheduled)
instead of adding a new SsbActionType variant.  No SCHEMA_VERSION bump
(stays at 269).  No new action_type.  This is the lower-risk JC-11
reuse path vs the +1 variant alternative.

Landed remaps:
- SchemaApplied remaps scaffold success emit
- RouteRuleApplied remaps RouteToTiledComposite emit
- ResizeRuleApplied + EvalScheduled are unused arms (preprocess not on
  handle_infer_request; T6 EVAL owner-gated) — Preprocessed pattern,
  honestly declared

Falsification: T9-falsification.md — RED (mutate SchemaApplied as_str to
"scaffold_emitted") → as_str contract FAIL; GREEN (restore) → PASS.

Named uncovered: deleting the production Kind emit does NOT flip
production_caller to DARK for Pattern-2 Kind discriminators (definition
+ as_str match keep the symbol "WIRED").  No contract asserts the live
emit remap this slice.  Unit as_str falsification covers the Kind API;
emit presence is recorded by the GREEN ref list only.

## R8 wire-dark allowlist (T7/T8, @d83bd09)

7 WIRE-DARK[J-8] allowlist rows registered in wire-dark-allowlist.toml:
vision_ladder×4 + prompt_shapes×3.  Each row carries owner, expiry,
promotion_plan_id=J-9, smoke_test=existing j8 contracts.  Dark-with-a-row
is correct; no fake consumer wired to dodge the row.  Gate freeze
re-locked after allowlist edit (owner-authorized, out-of-band).

T7/T8 pub fns stay REGISTERED-DARK until J-9 / H-14 consumers mount.

## What stayed external-figure-only

All vision_ladder.toml mmmu_design values are [DESIGN] figures from the
corpus, not ELAI-measured.  No corpus figure appears as an ELAI claim or
in any metrics TOML.  The sweet_spots.toml values are ELAI-measured (not
corpus figures) but carry the HD-1/EXTERNAL SRC-3 re-verify tag.  The
MoAI citation (arXiv:2403.07508) is a library const, not a user-facing
claim.  Everything in vision_ladder [DESIGN] rows is design rationale
only until J-9 measures at eval.

## Landed Tier-1 commits

| Task | Commit | Summary |
|------|--------|---------|
| T1   | 3e05c50 | sweet_spots.toml + sweet_spot lookup |
| T2   | 717027e | apply_schema_v2 + scaffold_with_call1 |
| T4   | 6f7de39 | multi-image + micro-text routing |
| T8   | cc8f985 | vision_ladder.toml J-9 seed |
| T7   | 5436f3c | prompt_shapes.rs + MoAI citation |
| R8   | d83bd09 | wire-dark allowlist (7 rows, J-9 promotion) |
| T9   | a154ade | PerceptionScaffoldEventKind JC-11 reuse arms |

## frontend_pair / W-6

J-8 inherits J-1's drawer-tile justification for W-6 absence.  W-4 rides
J-1's pair: the upgrade-rule + eval rows surface in the same drawer tile
J-1 chose for perception rows.  No new dashboard mount this plan.  J-1
justified W-6 absence; J-8 restates that justification (plan
frontend_pair bullet).

## Named UNCOVERED mutations (honest, not claimed as covered)

- T1: live sidecar resize still WIRE-DARK[J-1] (DegradedPassthrough);
  router does not yet consume native_res for passthrough branching
- T2: live production_agent_rpc calls scaffold() →
  scaffold_with_call1(input, None, 0); schema v2 gate not on live infer
  path until T5 feeds call-1 elements
- T4: live InferRequest frame is text-only (image_count 0 or 1);
  tiled-composite arm not exercised on live path; B-24 Mode B / live
  tiler dispatch not wired; SSB payload fields (kind, residue) have no
  contract assertion
- T7: no production call site consumes component_inventory_template or
  moai_citation; WIRE-DARK[J-8] allowlist-backed
- T8: J-9 does not yet consume vision_ladder.toml; J-1 leg-4 probe
  selection does not call local_rows_fitting_vram; no live probe routing
  claimed
- T9: production emit-site delete does not flip production_caller to
  DARK for Kind discriminators; no contract asserts live emit remap

## R4 standing rule; R1-R8 CLOSED

R4 remains a standing rule: no SCHEMA_VERSION/TOTAL_VARIANTS restatement
in new variant comments.  R1 through R8 are all CLOSED; do not re-open.
Vacuous-verification doctrine binds (ledger D-174): a tautological
assert that cannot fail is not a proof; an identity+hardcoded-true
debug_assert games production_caller_check without enforcing anything.

## RESIDUALS (OWNER-GATED)

- T3 DOM-leg control-flow: OWNER-GATED.  Router.rs Counting arm
  currently returns TypedRefusal.  Adding a DOM-leg dispatch is a new
  control-flow arm in the production gate at
  production_agent_rpc.rs:1934-1946, not a data swap.  Requires owner
  seam decision; do NOT invent without owner GO.

- T5 cache-placement assert: SET_ASIDE.  Two attempts
  (identity+hardcoded-true, then always-Some(u32)) were
  VERIFIER_REJECTED as vacuous.  handle_infer_request has no
  ContextBudget / cache_breakpoint object in scope; threading a real
  breakpoint validity signal into AgentRpc would invent an I-10-class
  guard, which GD-1 explicitly forbids.  Owner-gated until a real
  post-breakpoint token/signal exists at the AgentRpc seam (or owner
  accepts a different proof shape).  Do NOT re-attempt tautological
  debug_assert forms.

- T6 EVAL harness + JC-7/JC-8: OWNER-GATED.  bench_vlm_eval.rs not
  shipped.  dots.ocr / Qwen3-VL-Embedding package names unresolved
  (anti-slopsquat JC-7/JC-8).  Harness is Tier-2, sidecar/RunPod-gated
  W-10.  Tier-1 ships independently of this gate.

- T7/T8 REGISTERED-DARK: allowlist-backed @d83bd09
  (promotion_plan_id=J-9 / H-14).  Pub fns stay dark until J-9 / H-14
  consumers mount.

- J-39 EXP-1 / SplitTarget.entry_files / default-ON: remain in
  escalation_queue until owner GO.

- J-40 CLI --mode: OWNER-GATED residual remains in escalation_queue.
