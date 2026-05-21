// E-27 contract: new AWQ ModelCapabilityProfile entries for 3 VRAM tiers.
//
// Verifies:
// 1. awq_7b_12gb / awq_3b_12gb / awq_3b_8gb factory methods compile and produce
//    structurally sound profiles (non-zero ceilings, valid slot weights, etc).
// 2. quant_format field is set to "AWQ-Marlin+FA2+FP8KV" for all AWQ profiles.
// 3. kv_quant_bits == 8 for all AWQ profiles (FP8 KV = 8-bit).
// 4. measured == false for all AWQ profiles (projected, not yet measured).

use elai_orchestrator::context_budget::ModelCapabilityProfile;

#[test]
fn awq_7b_12gb_quant_format() {
    let p = ModelCapabilityProfile::awq_7b_12gb();
    assert_eq!(p.quant_format, "AWQ-Marlin+FA2+FP8KV");
}

#[test]
fn awq_7b_12gb_kv_quant_bits() {
    let p = ModelCapabilityProfile::awq_7b_12gb();
    assert_eq!(p.kv_quant_bits, 8, "FP8 KV = 8 quant bits");
}

#[test]
fn awq_7b_12gb_not_measured() {
    let p = ModelCapabilityProfile::awq_7b_12gb();
    assert!(!p.measured, "AWQ profiles are projected, not measured");
}

#[test]
fn awq_7b_12gb_ceiling_nonzero() {
    let p = ModelCapabilityProfile::awq_7b_12gb();
    assert!(p.effective_ceiling > 0);
    assert!(p.recommended_prompt_budget > 0);
    assert!(p.effective_ceiling >= p.recommended_prompt_budget);
}

#[test]
fn awq_7b_12gb_slot_weights_sum_to_one() {
    let p = ModelCapabilityProfile::awq_7b_12gb();
    assert!(p.slot_weights.validate_dynamic_sum(), "slot weights must sum to 1.0");
}

#[test]
fn awq_3b_12gb_quant_format() {
    let p = ModelCapabilityProfile::awq_3b_12gb();
    assert_eq!(p.quant_format, "AWQ-Marlin+FA2+FP8KV");
}

#[test]
fn awq_3b_12gb_ceiling_larger_than_nf4() {
    let nf4 = ModelCapabilityProfile::micro_3b_12gb();
    let awq = ModelCapabilityProfile::awq_3b_12gb();
    assert!(
        awq.effective_ceiling >= nf4.effective_ceiling,
        "AWQ 3B ceiling {awq_ceil} should be >= NF4 3B ceiling {nf4_ceil}",
        awq_ceil = awq.effective_ceiling,
        nf4_ceil = nf4.effective_ceiling,
    );
}

#[test]
fn awq_3b_8gb_quant_format() {
    let p = ModelCapabilityProfile::awq_3b_8gb();
    assert_eq!(p.quant_format, "AWQ-Marlin+FA2+FP8KV");
}

#[test]
fn awq_3b_8gb_kv_quant_bits() {
    let p = ModelCapabilityProfile::awq_3b_8gb();
    assert_eq!(p.kv_quant_bits, 8);
}

#[test]
fn awq_3b_8gb_slot_weights_valid() {
    let p = ModelCapabilityProfile::awq_3b_8gb();
    assert!(p.slot_weights.validate_dynamic_sum());
}

#[test]
fn existing_micro_7b_12gb_quant_format_nf4() {
    let p = ModelCapabilityProfile::micro_7b_12gb();
    assert_eq!(p.quant_format, "NF4+FA2");
}

#[test]
fn existing_micro_3b_12gb_quant_format_nf4() {
    let p = ModelCapabilityProfile::micro_3b_12gb();
    assert_eq!(p.quant_format, "NF4+FA2");
}