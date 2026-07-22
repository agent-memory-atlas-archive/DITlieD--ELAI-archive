pub const WRAP_RRA_DISABLE_ENV: &str = "ELAI_WRAP_RRA_DISABLE";
#[must_use]
pub fn rra_disabled() -> bool {
    std::env::var(WRAP_RRA_DISABLE_ENV).unwrap_or_default() == "1"
}
