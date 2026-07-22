//! J-1.T2 — perception scaffold (two-call flow structure).
//!
//! Source plan: `docs/thefinalglance/J-plan/J-1_perception_gateway.md`
//! (lines 61-64). Call 1 = describe-literal-elements to structured JSON
//! (temp 0, 0-1000 normalized coords, instruction BEFORE image). Call 2 =
//! the task model reasons over description+image. Scaffold fires ONLY on
//! image-bearing dispatches; injected AFTER the last cache breakpoint
//! (HD-2 — never busts the byte-stable prefix; the prompt passed to
//! `handle_infer_request` is the post-assembly volatile tail).
//!
//! T2 ships the two-call STRUCTURE + the wiring at
//! `production_agent_rpc.rs::handle_infer_request`. The actual VLM calls
//! (call 1 describe-literal-elements, call 2 task-model-reasons) are
//! STUBBED in T2 (USER_OVERRIDE for legs 2/4 — T5 replaces the stubs with
//! real sidecar calls, sidecar-gated W-10). The `scaffold` fn is WIRED —
//! no WIRE-DARK[J-1] marker, no allowlist row.
//!
//! J-8 T2 extension: schema v2 coord validation gate. Call-1
//! literal-element coords MUST be normalized 0..=SCAFFOLD_COORD_NORMALIZATION_MAX
//! (1000). A raw-pixel-looking emission (any bbox value >1000) FAILS
//! validation -> re-prompt once -> on second failure TypedRefusal. The
//! `apply_schema_v2` fn is the shared production fn contracts MUST call.

use serde::{Deserialize, Serialize};

use super::preprocess::PerceptionError;

/// Call-1 temperature (describe-literal-elements): 0.0 — deterministic
/// structured description (SRC-3 mechanism 4).
pub const SCAFFOLD_CALL1_TEMPERATURE: f32 = 0.0;

/// Coordinate normalization max for the describe-literal-elements output
/// (0..=1000 normalized coords, SRC-3 mechanism 4).
pub const SCAFFOLD_COORD_NORMALIZATION_MAX: u32 = 1000;

/// Maximum literal elements the describe-literal-elements call may emit
/// (plan stub_ban: bounded cost on image-heavy sessions; GD-4 measure
/// before default-ON for call-2-always).
pub const SCAFFOLD_MAX_LITERAL_ELEMENTS: usize = 256;

/// Marker the scaffold's image-detection helper looks for in the prompt
/// string. The runtime InferRequest frame is text-only today
/// (`InputContentBlock` has no Image variant — exploration plan GAP at
/// `ipc_gateway.rs:274`); images are embedded as `data:image/...` or
/// `base64,...` markers per plan line 42.
pub const PROMPT_IMAGE_MARKER_DATA: &str = "data:image/";
/// Second marker — bare base64 prefix (less common; covered for parity).
pub const PROMPT_IMAGE_MARKER_BASE64: &str = "base64,";

/// Outcome of the scaffold step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaffoldOutcome {
    /// Scaffold applied — call 1 (describe-literal-elements) + call 2
    /// (task model) both completed; `literal_elements` carries the
    /// structured description.
    ScaffoldEmitted,
    /// No image present in the input — scaffold is a no-op.
    NoImagePresent,
    /// Scaffold failed (call-1 error, call-2 error, sidecar unavailable,
    /// etc.). The raw image forwards UNCONDITIONED with an explicit marker
    /// — never a silent fallthrough (plan stub_ban).
    DegradedPassthrough,
    /// J-8 T2: call-1 elements failed schema v2 coord validation on the
    /// first attempt; the caller should re-prompt call-1 once with the
    /// coord constraint re-emphasized, then call
    /// [`scaffold_with_call1`] with `schema_attempt = 1`.
    SchemaReprompt,
    /// J-8 T2: call-1 elements failed schema v2 coord validation on the
    /// second attempt. The scaffold refuses to emit unvalidated
    /// coordinates; the raw image forwards with a typed refusal marker.
    TypedRefusal,
}

/// One literal element emitted by the describe-literal-elements call.
/// `bbox` is in normalized `[0, SCAFFOLD_COORD_NORMALIZATION_MAX]` coords
/// (SRC-3 mechanism 4: 0-1000 normalized coords).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiteralElement {
    pub label: String,
    /// `[x0, y0, x1, y1]` in `[0, SCAFFOLD_COORD_NORMALIZATION_MAX]`.
    pub bbox: [u32; 4],
}

/// Result of the scaffold step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScaffoldResult {
    pub outcome: ScaffoldOutcome,
    /// Literal elements on `ScaffoldEmitted`; empty otherwise.
    pub literal_elements: Vec<LiteralElement>,
    /// Call-1 temperature actually used (pinned to
    /// `SCAFFOLD_CALL1_TEMPERATURE`).
    pub call1_temperature: f32,
}

/// Scaffold input. The dispatch seam constructs this from the prompt
/// string via [`ScaffoldInput::from_prompt`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaffoldInput {
    /// Image bytes extracted from the prompt (empty when the prompt
    /// carries no image marker).
    pub image_bytes: Vec<u8>,
    /// Task prompt (the non-image portion of the dispatch prompt).
    pub task_prompt: String,
}

impl ScaffoldInput {
    /// Construct a [`ScaffoldInput`] from a fully-assembled prompt string.
    /// The runtime InferRequest frame is text-only today; images are
    /// embedded as `data:image/...` or `base64,...` markers (plan line 42,
    /// exploration plan GAP at `ipc_gateway.rs:274`). When no image marker
    /// is present, `image_bytes` is empty and `task_prompt` is the full
    /// prompt (the scaffold then returns `NoImagePresent`).
    #[must_use]
    pub fn from_prompt(prompt: &str) -> Self {
        if prompt_carries_image(prompt) {
            // T2 ships the structure; the actual image-byte extraction
            // (parse the data-URI / base64 payload) lands in T5 with the
            // live sidecar call. Until then, carry the marker presence as
            // a non-empty image_bytes signal so the scaffold's
            // NoImagePresent / DegradedPassthrough arms stay
            // distinguishable.
            Self {
                image_bytes: vec![0u8],
                task_prompt: prompt.to_string(),
            }
        } else {
            Self {
                image_bytes: Vec::new(),
                task_prompt: prompt.to_string(),
            }
        }
    }
}

/// True when the prompt string carries an image marker (`data:image/` or
/// `base64,`). The runtime InferRequest frame is text-only today; this is
/// the has_image signal the scaffold + router reuse (exploration plan
/// `perception::scaffold::prompt_carries_image`).
#[must_use]
pub fn prompt_carries_image(prompt: &str) -> bool {
    prompt.contains(PROMPT_IMAGE_MARKER_DATA) || prompt.contains(PROMPT_IMAGE_MARKER_BASE64)
}

/// J-8 T2: verdict of the schema v2 coord validation gate applied to
/// call-1 literal elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaV2Verdict {
    /// All element bboxes are within `[0, SCAFFOLD_COORD_NORMALIZATION_MAX]`.
    Accepted,
    /// At least one bbox value exceeds the normalization max on the first
    /// attempt; re-prompt call-1 once.
    RepromptOnce,
    /// At least one bbox value exceeds the normalization max on the second
    /// attempt; refuse to emit unvalidated coordinates.
    TypedRefusal,
}

/// J-8 T2: true iff every value in `bbox` is within
/// `[0, SCAFFOLD_COORD_NORMALIZATION_MAX]` (0..=1000).
#[must_use]
pub fn coords_within_schema_v2(bbox: &[u32; 4]) -> bool {
    bbox.iter().all(|&v| v <= SCAFFOLD_COORD_NORMALIZATION_MAX)
}

/// J-8 T2: true iff every `LiteralElement` in `elements` has a bbox that
/// passes [`coords_within_schema_v2`].
#[must_use]
pub fn elements_pass_schema_v2(elements: &[LiteralElement]) -> bool {
    elements.iter().all(|e| coords_within_schema_v2(&e.bbox))
}

/// J-8 T2: apply the schema v2 coord validation gate to `elements` at the
/// given `attempt` (0 = first call-1, 1 = after re-prompt). This is the
/// shared production fn that both [`scaffold_with_call1`] and contract
/// tests call.
///
/// - If all bboxes pass -> [`SchemaV2Verdict::Accepted`]
/// - If any bbox fails and `attempt == 0` -> [`SchemaV2Verdict::RepromptOnce`]
/// - If any bbox fails and `attempt >= 1` -> [`SchemaV2Verdict::TypedRefusal`]
#[must_use]
pub fn apply_schema_v2(elements: &[LiteralElement], attempt: u8) -> SchemaV2Verdict {
    if elements_pass_schema_v2(elements) {
        SchemaV2Verdict::Accepted
    } else if attempt == 0 {
        SchemaV2Verdict::RepromptOnce
    } else {
        SchemaV2Verdict::TypedRefusal
    }
}

/// J-8 T2: run the perception scaffold with call-1 literal elements
/// provided. When `call1_elements` is `Some`, the schema v2 coord
/// validation gate runs via [`apply_schema_v2`]. When `None`, the scaffold
/// falls through to the T2 DegradedPassthrough stub (T5 wires the live
/// sidecar call-1).
///
/// `schema_attempt` is 0 on the first call-1, 1 after a re-prompt.
///
/// # Errors
/// Returns [`PerceptionError::ScaffoldFailed`] only on internal
/// inconsistency. Otherwise degrades to passthrough rather than panicking.
pub fn scaffold_with_call1(
    input: &ScaffoldInput,
    call1_elements: Option<&[LiteralElement]>,
    schema_attempt: u8,
) -> Result<ScaffoldResult, PerceptionError> {
    if input.image_bytes.is_empty() {
        return Ok(ScaffoldResult {
            outcome: ScaffoldOutcome::NoImagePresent,
            literal_elements: Vec::new(),
            call1_temperature: SCAFFOLD_CALL1_TEMPERATURE,
        });
    }
    if let Some(elements) = call1_elements {
        return match apply_schema_v2(elements, schema_attempt) {
            SchemaV2Verdict::Accepted => Ok(ScaffoldResult {
                outcome: ScaffoldOutcome::ScaffoldEmitted,
                literal_elements: elements.to_vec(),
                call1_temperature: SCAFFOLD_CALL1_TEMPERATURE,
            }),
            SchemaV2Verdict::RepromptOnce => Ok(ScaffoldResult {
                outcome: ScaffoldOutcome::SchemaReprompt,
                literal_elements: Vec::new(),
                call1_temperature: SCAFFOLD_CALL1_TEMPERATURE,
            }),
            SchemaV2Verdict::TypedRefusal => Ok(ScaffoldResult {
                outcome: ScaffoldOutcome::TypedRefusal,
                literal_elements: Vec::new(),
                call1_temperature: SCAFFOLD_CALL1_TEMPERATURE,
            }),
        };
    }
    // T2 skeleton — the two-call flow STRUCTURE is in place (call-1
    // describe-literal-elements at temp 0, 0-1000 normalized coords,
    // instruction-before-image; call-2 task model reasons over
    // description+image). T5 replaces this DegradedPassthrough arm with
    // the live sidecar calls. Never a silent fallthrough (plan stub_ban).
    Ok(ScaffoldResult {
        outcome: ScaffoldOutcome::DegradedPassthrough,
        literal_elements: Vec::new(),
        call1_temperature: SCAFFOLD_CALL1_TEMPERATURE,
    })
}

/// Run the two-call perception scaffold on `input`. T2 ships the
/// two-call STRUCTURE (call-1 describe-literal-elements dispatch shape +
/// call-2 task-model dispatch shape + the structured-JSON handoff). The
/// live VLM calls are STUBBED in T2 (USER_OVERRIDE for legs 2/4 — T5
/// replaces the stubs with real sidecar calls, sidecar-gated W-10).
///
/// J-8 T2: delegates to [`scaffold_with_call1`] with `call1_elements =
/// None` and `schema_attempt = 0`, preserving the existing
/// DegradedPassthrough stub behavior when no call-1 elements are
/// provided (the live infer path still uses `from_prompt` which carries
/// no call-1 elements until T5 feeds them).
///
/// # Errors
/// Returns [`PerceptionError::ScaffoldFailed`] only on internal
/// inconsistency. Otherwise degrades to passthrough rather than panicking.
pub fn scaffold(input: &ScaffoldInput) -> Result<ScaffoldResult, PerceptionError> {
    scaffold_with_call1(input, None, 0)
}
