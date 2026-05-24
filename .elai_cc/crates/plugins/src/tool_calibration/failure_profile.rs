//! C-47: Five-dimensional failure profile per CAST paper (arXiv:2605.15041).
//!
//! The CAST framework decomposes tool-call failures into five disjoint
//! dimensions: `name`, `key`, `type`, `constraint`, `value`. The paper's
//! R_tool reward is six-dimensional (these 5 + exact-match); for the
//! telemetry-only surface in C-47 we record the 5 dimensions per
//! [`AggregatedCase`] so the dashboard can render per-dimension trends and
//! the hinter can pick the loudest failure mode when emitting a corrective
//! hint.
//!
//! Critical invariant (testable in the contract test):
//!   `len(failure_profile_dimensions()) == 5` — paper Section 3.2.

use serde::{Deserialize, Serialize};

/// Five-dimensional failure profile aggregated across all historical calls
/// for one (tool, call_shape_hash) pair. Each field is a monotonically
/// increasing counter; deterministic semantics avoid the
/// non-deterministic-eviction stub_ban from the WIRING_CONTRACT.
///
/// Field meaning per CAST paper Section 3.2:
///   * `name_err`     — caller invoked an unknown tool name
///   * `key_err`      — argument key absent / extra / typo'd
///   * `type_err`     — value type mismatch (string vs int etc.)
///   * `constraint_err` — value violated a schema constraint (range,
///                        regex, enum)
///   * `value_err`    — value type ok but semantically wrong (e.g. path
///                      does not exist; auth token expired)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureProfile {
    pub name_err: u32,
    pub key_err: u32,
    pub type_err: u32,
    pub constraint_err: u32,
    pub value_err: u32,
}

impl FailureProfile {
    /// Total error count across all five dimensions. Equivalent to
    /// summing every field; held distinct from the [`AggregatedCase`]
    /// `error_count` because that one rolls forward even on
    /// dimension-unknown failures (kept consistent via
    /// [`Self::record`]).
    pub fn total(&self) -> u32 {
        self.name_err
            .saturating_add(self.key_err)
            .saturating_add(self.type_err)
            .saturating_add(self.constraint_err)
            .saturating_add(self.value_err)
    }

    /// Increment the bucket matching `dim`. Saturates at `u32::MAX` to
    /// avoid wrap-around (a long-running session with billions of
    /// failures of the same kind is implausible but worth a guard).
    pub fn record(&mut self, dim: FailureDimension) {
        let slot = match dim {
            FailureDimension::Name => &mut self.name_err,
            FailureDimension::Key => &mut self.key_err,
            FailureDimension::Type => &mut self.type_err,
            FailureDimension::Constraint => &mut self.constraint_err,
            FailureDimension::Value => &mut self.value_err,
        };
        *slot = slot.saturating_add(1);
    }

    /// Return the dimension with the highest count (ties broken in
    /// declaration order: Name > Key > Type > Constraint > Value).
    /// Returns `None` when all dimensions are zero (i.e. no recorded
    /// failures). Used by the hinter to pick the loudest signal.
    pub fn dominant_dimension(&self) -> Option<FailureDimension> {
        let mut best: Option<(FailureDimension, u32)> = None;
        for (dim, count) in [
            (FailureDimension::Name, self.name_err),
            (FailureDimension::Key, self.key_err),
            (FailureDimension::Type, self.type_err),
            (FailureDimension::Constraint, self.constraint_err),
            (FailureDimension::Value, self.value_err),
        ] {
            if count == 0 {
                continue;
            }
            match best {
                None => best = Some((dim, count)),
                Some((_, prev)) if count > prev => best = Some((dim, count)),
                _ => {}
            }
        }
        best.map(|(d, _)| d)
    }
}

/// One of the five CAST failure dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureDimension {
    Name,
    Key,
    Type,
    Constraint,
    Value,
}

impl FailureDimension {
    /// Stable wire-string used by the hinter when assembling the
    /// corrective message.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Key => "key",
            Self::Type => "type",
            Self::Constraint => "constraint",
            Self::Value => "value",
        }
    }
}

/// Returns the canonical declaration order of the five dimensions.
/// Used by the contract test to assert `len == 5` per paper invariant.
pub const fn failure_profile_dimensions() -> [FailureDimension; 5] {
    [
        FailureDimension::Name,
        FailureDimension::Key,
        FailureDimension::Type,
        FailureDimension::Constraint,
        FailureDimension::Value,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_count_is_five_per_paper() {
        assert_eq!(failure_profile_dimensions().len(), 5);
    }

    #[test]
    fn dominant_dimension_picks_highest() {
        let mut p = FailureProfile::default();
        for _ in 0..3 {
            p.record(FailureDimension::Key);
        }
        for _ in 0..7 {
            p.record(FailureDimension::Value);
        }
        assert_eq!(p.dominant_dimension(), Some(FailureDimension::Value));
        assert_eq!(p.total(), 10);
    }

    #[test]
    fn dominant_dimension_none_on_empty() {
        assert_eq!(FailureProfile::default().dominant_dimension(), None);
    }

    #[test]
    fn dominant_tie_broken_in_declaration_order() {
        let mut p = FailureProfile::default();
        p.record(FailureDimension::Type);
        p.record(FailureDimension::Constraint);
        // First-seen wins on ties (declaration order).
        assert_eq!(p.dominant_dimension(), Some(FailureDimension::Type));
    }
}
