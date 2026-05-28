"""Transfer-check contract: re-eval gap > 2pp triggers overfit flag."""

from elai_optimizer.transfer_check import validate_transfer


def test_no_gap_no_overfit():
    r = validate_transfer(
        val_lift_pct=8.0,
        target_reeval_lift_pct=7.5,
    )
    assert r.overfit_risk is False
    assert r.epsilon_pct == 2.0


def test_gap_above_epsilon_triggers_overfit():
    r = validate_transfer(
        val_lift_pct=10.0,
        target_reeval_lift_pct=5.0,
    )
    assert r.overfit_risk is True


def test_gap_at_epsilon_boundary_does_not_overfit():
    r = validate_transfer(
        val_lift_pct=8.0,
        target_reeval_lift_pct=6.0,
    )
    assert r.overfit_risk is False


def test_canary_results_passed_through():
    r = validate_transfer(
        val_lift_pct=8.0,
        target_reeval_lift_pct=7.5,
        canary_results=[("math", 1.5), ("code", -0.5)],
    )
    assert r.canary_results == [("math", 1.5), ("code", -0.5)]


def test_custom_epsilon_respected():
    r = validate_transfer(
        val_lift_pct=5.0,
        target_reeval_lift_pct=4.5,
        epsilon_pct=0.1,
    )
    assert r.overfit_risk is True
