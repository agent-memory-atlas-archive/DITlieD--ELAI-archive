// J-24 LEG1=A contract: steering boot variant.
//
// Verifies that steering_boot_flag() returns the correct argv flag based on
// the ELAI_STEERING_ENABLE env var:
// 1. ENABLE=1 -> "--steering" (and NOT "--no-steering")
// 2. unset    -> "--no-steering"
// 3. ENABLE=0 -> "--no-steering"
// 4. ENABLE=garbage -> "--no-steering" (only "1" is affirmative)

use elai_orchestrator::behavior_steering_ssb::{
    steering_boot_flag, ELAI_STEERING_ENABLE_ENV,
};
use std::env;

#[test]
fn steering_enabled_passes_steering_flag() {
    env::set_var(ELAI_STEERING_ENABLE_ENV, "1");
    let flag = steering_boot_flag();
    assert_eq!(flag, "--steering");
    assert_ne!(flag, "--no-steering");
    env::remove_var(ELAI_STEERING_ENABLE_ENV);
}

#[test]
fn steering_unset_passes_no_steering_flag() {
    env::remove_var(ELAI_STEERING_ENABLE_ENV);
    let flag = steering_boot_flag();
    assert_eq!(flag, "--no-steering");
}

#[test]
fn steering_explicit_zero_passes_no_steering_flag() {
    env::set_var(ELAI_STEERING_ENABLE_ENV, "0");
    let flag = steering_boot_flag();
    assert_eq!(flag, "--no-steering");
    env::remove_var(ELAI_STEERING_ENABLE_ENV);
}

#[test]
fn steering_garbage_value_passes_no_steering_flag() {
    env::set_var(ELAI_STEERING_ENABLE_ENV, "yes");
    let flag = steering_boot_flag();
    assert_eq!(flag, "--no-steering");
    env::remove_var(ELAI_STEERING_ENABLE_ENV);
}
