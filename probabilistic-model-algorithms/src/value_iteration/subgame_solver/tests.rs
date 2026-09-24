use super::{OptimisticValueIteration, SubGameSolver, ValueIteration};
use crate::value_iteration::non_determinism::{Maximise, Minimise};
use probabilistic_models::mdp;
use typed_index_collections::To1;

// Verifies that convergence is detected if some states can reach the goal with zero reward.
#[test]
fn value_iteration_zero_valued_state() {
    mdp!(mdp = {
        s -> 0.5: t,
        t -> 1.0: s,
        t ->
    });
    let exit_values = To1::with_entries(vec![1.0, 1.0, 0.0]);
    let mut solver = ValueIteration::create(2);
    let values = solver.solve::<Minimise, _, _, _>(&mdp, &exit_values, 1e-6, f64::INFINITY);
    assert_eq!(values, [1.0, 0.0]);
}

#[test]
fn optimistic_value_iteration_zero_valued_state() {
    mdp!(mdp = {
        s -> 0.5: t,
        t -> 1.0: s,
        t ->
    });
    let exit_values = To1::with_entries(vec![1.0, 1.0, 0.0]);
    let mut solver = OptimisticValueIteration::create(2);
    let values = solver.solve::<Minimise, _, _, _>(&mdp, &exit_values, 1e-6, f64::INFINITY);
    assert_eq!(values, [1.0, 0.0]);
}

// Checks that OVI converges even if it sets eps to 0 when checking the upper bound.
#[test]
fn value_iteration_eps_zero() {
    mdp!(mdp = {
        s -> 0.5: t,
        t -> 0.5: s
    });
    let exit_values = To1::with_entries(vec![0.5, 0.5]);
    let mut solver = ValueIteration::create(2);
    let values = solver.solve::<Maximise, _, _, _>(&mdp, &exit_values, 0.0, 1.0);
    for value in values {
        assert!((value - 1.0).abs() < 1e-15, "Expected 1, got {value}");
    }
}
