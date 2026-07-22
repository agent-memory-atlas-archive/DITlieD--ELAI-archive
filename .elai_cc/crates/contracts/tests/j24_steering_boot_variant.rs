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
use std::sync::Mutex;

static STEERING_ENV_MUTEX: Mutex<()> = Mutex::new(());

struct EnvRestoreGuard {
    key: &'static str,
    prior: Option<String>,
}

impl EnvRestoreGuard {
    fn capture(key: &'static str) -> Self {
        EnvRestoreGuard {
            key,
            prior: env::var(key).ok(),
        }
    }
}

impl Drop for EnvRestoreGuard {
    fn drop(&mut self) {
        match &self.prior {
            Some(val) => env::set_var(self.key, val),
            None => env::remove_var(self.key),
        }
    }
}

#[test]
fn steering_enabled_passes_steering_flag() {
    let _mutex = STEERING_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = EnvRestoreGuard::capture(ELAI_STEERING_ENABLE_ENV);
    env::set_var(ELAI_STEERING_ENABLE_ENV, "1");
    let flag = steering_boot_flag();
    assert_eq!(flag, "--steering");
    assert_ne!(flag, "--no-steering");
}

#[test]
fn steering_unset_passes_no_steering_flag() {
    let _mutex = STEERING_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = EnvRestoreGuard::capture(ELAI_STEERING_ENABLE_ENV);
    env::remove_var(ELAI_STEERING_ENABLE_ENV);
    let flag = steering_boot_flag();
    assert_eq!(flag, "--no-steering");
}

#[test]
fn steering_explicit_zero_passes_no_steering_flag() {
    let _mutex = STEERING_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = EnvRestoreGuard::capture(ELAI_STEERING_ENABLE_ENV);
    env::set_var(ELAI_STEERING_ENABLE_ENV, "0");
    let flag = steering_boot_flag();
    assert_eq!(flag, "--no-steering");
}

#[test]
fn steering_garbage_value_passes_no_steering_flag() {
    let _mutex = STEERING_ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = EnvRestoreGuard::capture(ELAI_STEERING_ENABLE_ENV);
    env::set_var(ELAI_STEERING_ENABLE_ENV, "yes");
    let flag = steering_boot_flag();
    assert_eq!(flag, "--no-steering");
}
