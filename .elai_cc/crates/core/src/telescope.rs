//! Context telescope types for hierarchical prompt context injection.
//!
//! `ContextTelescope` provides 4-layer vertical context (project -> concept -> phase -> step).
//! `SurgicalTelescope` provides a shallower 3-layer alternative for edit-existing-code tasks.

use crate::bounded_string::BoundedString;

// ---------------------------------------------------------------------------
// Named constants — max tokens per telescope layer
// ---------------------------------------------------------------------------

pub const TELESCOPE_VISION_MAX: usize = 200;
pub const TELESCOPE_CONCEPT_MAX: usize = 150;
pub const TELESCOPE_PHASE_MAX: usize = 100;
pub const TELESCOPE_STEP_MAX: usize = 100;

pub const SURGICAL_VISION_MAX: usize = 200;
pub const SURGICAL_BLAST_MAX: usize = 200;
pub const SURGICAL_STEP_MAX: usize = 200;

/// Label overhead estimate (tokens consumed by labels/formatting in render output).
pub const LABEL_OVERHEAD_TOKENS: usize = 20;

// ---------------------------------------------------------------------------
// ContextTelescope — 4-layer vertical context
// ---------------------------------------------------------------------------

/// Hierarchical context for baby step prompts.
///
/// Layer 1: project_vision (always present)
/// Layer 2: concept_summary (optional — from ConceptNode)
/// Layer 3: phase_goal (optional — from PhaseEntry)
/// Layer 4: step_context (optional — from StepNode)
pub struct ContextTelescope {
    pub project_vision: BoundedString<TELESCOPE_VISION_MAX>,
    pub concept_summary: Option<BoundedString<TELESCOPE_CONCEPT_MAX>>,
    pub phase_goal: Option<BoundedString<TELESCOPE_PHASE_MAX>>,
    pub step_context: Option<BoundedString<TELESCOPE_STEP_MAX>>,
}

impl ContextTelescope {
    /// Render the telescope layers into a single string for prompt injection.
    ///
    /// Omits layers that are None. Always includes project_vision.
    pub fn render(&self) -> String {
        let mut parts = Vec::with_capacity(4);
        parts.push(format!("[Project] {}", self.project_vision.as_str()));

        if let Some(ref concept) = self.concept_summary {
            parts.push(format!("[Concept] {}", concept.as_str()));
        }
        if let Some(ref phase) = self.phase_goal {
            parts.push(format!("[Phase] {}", phase.as_str()));
        }
        if let Some(ref step) = self.step_context {
            parts.push(format!("[Step] {}", step.as_str()));
        }

        parts.join("\n")
    }

    /// Estimate total token count of the rendered output.
    ///
    /// Uses word count of each present layer plus label overhead.
    pub fn token_estimate(&self) -> usize {
        let mut total = word_count(self.project_vision.as_str());
        if let Some(ref c) = self.concept_summary {
            total += word_count(c.as_str());
        }
        if let Some(ref p) = self.phase_goal {
            total += word_count(p.as_str());
        }
        if let Some(ref s) = self.step_context {
            total += word_count(s.as_str());
        }
        total + LABEL_OVERHEAD_TOKENS
    }
}

// ---------------------------------------------------------------------------
// SurgicalTelescope — 3-layer shallow context for surgical mode
// ---------------------------------------------------------------------------

/// Shallower telescope for edit-existing-code (surgical) tasks.
///
/// Layer 1: project_vision (auto-generated from DKG concept annotations)
/// Layer 2: blast_radius (from blast radius analysis)
/// Layer 3: step_task (the specific task)
///
/// Total budget: ~400-900 tokens.
pub struct SurgicalTelescope {
    pub project_vision: BoundedString<SURGICAL_VISION_MAX>,
    pub blast_radius: Option<BoundedString<SURGICAL_BLAST_MAX>>,
    pub step_task: Option<BoundedString<SURGICAL_STEP_MAX>>,
}

impl SurgicalTelescope {
    /// Render the surgical telescope layers into a single string.
    pub fn render(&self) -> String {
        let mut parts = Vec::with_capacity(3);
        parts.push(format!("[Project] {}", self.project_vision.as_str()));

        if let Some(ref blast) = self.blast_radius {
            parts.push(format!("[Blast Radius] {}", blast.as_str()));
        }
        if let Some(ref task) = self.step_task {
            parts.push(format!("[Task] {}", task.as_str()));
        }

        parts.join("\n")
    }

    /// Estimate total token count of the rendered output.
    pub fn token_estimate(&self) -> usize {
        let mut total = word_count(self.project_vision.as_str());
        if let Some(ref b) = self.blast_radius {
            total += word_count(b.as_str());
        }
        if let Some(ref t) = self.step_task {
            total += word_count(t.as_str());
        }
        total + LABEL_OVERHEAD_TOKENS
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn word_count(s: &str) -> usize {
    if s.is_empty() {
        return 0;
    }
    s.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bounded_string::WordTokenCounter;

    #[test]
    fn telescope_render_omits_none_layers() {
        let c = WordTokenCounter;
        let ct = ContextTelescope {
            project_vision: BoundedString::new("vision", &c).unwrap(),
            concept_summary: None,
            phase_goal: Some(BoundedString::new("phase goal", &c).unwrap()),
            step_context: None,
        };
        let rendered = ct.render();
        assert!(rendered.contains("[Project] vision"));
        assert!(rendered.contains("[Phase] phase goal"));
        assert!(!rendered.contains("[Concept]"));
        assert!(!rendered.contains("[Step]"));
    }

    #[test]
    fn surgical_render_all_layers() {
        let c = WordTokenCounter;
        let st = SurgicalTelescope {
            project_vision: BoundedString::new("vision", &c).unwrap(),
            blast_radius: Some(BoundedString::new("radius info", &c).unwrap()),
            step_task: Some(BoundedString::new("do the thing", &c).unwrap()),
        };
        let rendered = st.render();
        assert!(rendered.contains("[Project]"));
        assert!(rendered.contains("[Blast Radius]"));
        assert!(rendered.contains("[Task]"));
    }

    #[test]
    fn token_estimate_nonzero() {
        let c = WordTokenCounter;
        let ct = ContextTelescope {
            project_vision: BoundedString::new("hello world", &c).unwrap(),
            concept_summary: None,
            phase_goal: None,
            step_context: None,
        };
        assert!(ct.token_estimate() > 0);
    }
}
