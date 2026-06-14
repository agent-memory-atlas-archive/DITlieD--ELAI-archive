//! I-20 — property-based-testing (PBT) relation adapter.
//!
//! Extracted from `metamorphic_testing.rs` to keep that file under the
//! 1000-line maintainability boundary. The PBT lane surfaces a proptest
//! (Rust) / Hypothesis (Python) property as a `MetamorphicRelation`
//! (`PropertyBased`), registered behind the EXISTING `RelationLibrary` enum
//! dispatch — the I-20 plan's "D-3 adapter trait" is, in live code, that enum
//! dispatch (GD-1 / IC-14: no new dispatch layer is introduced).

use elai_core::clock::ClockProvider;

use crate::metamorphic_testing::{MetamorphicRelation, MrResult};

/// Run a property-based-testing relation: check `property` against every input
/// in `corpus`. The relation PASSES iff the property holds on all inputs, and
/// FAILS with the first violating (hex-encoded) input as the counterexample.
///
/// Composes the existing `MrResult` counterexample channel (no separate PBT
/// result type).
// WIRE-DARK[I-20-pbt]: registered behind the RelationLibrary dispatch (PropertyBased arm + parse_relation + run_all, reachable via [testing.metamorphic] config); the bespoke property corpus that drives this fn from check_metamorphic Step 7 is a Tier-2 follow-on. Backing row: stubs.md will_resolve_in=I-20-pbt-corpus.
pub fn run_property_based<P>(property: P, corpus: &[Vec<u8>]) -> MrResult
where
    P: Fn(&[u8]) -> bool,
{
    let start = elai_core::clock::OsClock.instant_now();
    let sample_size = u32::try_from(corpus.len()).unwrap_or(u32::MAX);
    for input in corpus {
        if !property(input) {
            let elapsed = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
            return MrResult::failed(
                MetamorphicRelation::PropertyBased,
                sample_size,
                format!("property violated on input 0x{}", hex_encode(input)),
                elapsed,
            );
        }
    }
    let elapsed = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    MrResult::passed(MetamorphicRelation::PropertyBased, sample_size, elapsed)
}

/// Lowercase-hex encode (counterexample rendering). Local to this module so
/// it does not widen the `metamorphic_testing` private surface.
fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pbt_passes_when_property_holds() {
        let corpus = vec![vec![1u8, 2, 3], vec![4u8, 5]];
        let r = run_property_based(|input| !input.is_empty(), &corpus);
        assert!(r.passed, "property holds on all inputs -> pass");
        assert_eq!(r.sample_size, 2);
        assert!(r.counterexample.is_none());
    }

    #[test]
    fn pbt_fails_with_counterexample() {
        let corpus = vec![vec![1u8], vec![]]; // second input violates "non-empty"
        let r = run_property_based(|input| !input.is_empty(), &corpus);
        assert!(!r.passed, "property violated -> fail");
        let ce = r.counterexample.expect("counterexample on failure");
        assert!(!ce.is_empty(), "counterexample must be non-empty");
    }
}
