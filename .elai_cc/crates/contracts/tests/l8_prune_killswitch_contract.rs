// This lives in its own binary because the var is process-global and the L-8 wire tests read it.

use elai_orchestrator::claim::prune::quarantine_disabled;

#[test]
fn quarantine_killswitch_is_process_local_to_this_test_binary() {
    std::env::remove_var("ELAI_STALENESS_CASCADE_KILL");
    assert!(!quarantine_disabled());
    std::env::set_var("ELAI_STALENESS_CASCADE_KILL", "1");
    assert!(quarantine_disabled());
    std::env::remove_var("ELAI_STALENESS_CASCADE_KILL");
}
