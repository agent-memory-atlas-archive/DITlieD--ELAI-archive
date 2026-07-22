//! J-39 T1: observe-only admission ceiling for the phase sizing splitter.
//!
//! Replaces the fixed [`crate::phase_sizing_splitter::DEFAULT_TOKEN_BUDGET`]
//! threshold with one derived from
//! [`crate::context_policy::effective_context_window`] under
//! [`crate::dispatcher::BackendMode::Local`]. When the effective window is 0
//! (misconfigured env), falls back to `DEFAULT_TOKEN_BUDGET` so prior
//! behaviour is preserved rather than silently skipping every split.

use crate::context_policy::effective_context_window;
use crate::dispatcher::BackendMode;
use crate::phase_sizing_splitter::DEFAULT_TOKEN_BUDGET;

/// Admission ceiling for the splitter threshold, derived from the live
/// effective context window under Local mode.
///
/// `(effective_context_window(Local) * budget_ratio) as u32`, falling back
/// to `DEFAULT_TOKEN_BUDGET` when the window is 0.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn admission_ceiling_threshold(budget_ratio: f32) -> u32 {
    let ceiling = effective_context_window(BackendMode::Local);
    if ceiling == 0 {
        return DEFAULT_TOKEN_BUDGET;
    }
    ((ceiling as f32) * budget_ratio) as u32
}
