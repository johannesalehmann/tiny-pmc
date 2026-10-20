mod optimistic_value_iteration;
pub use optimistic_value_iteration::OptimisticValueIteration;

mod value_iteration;
pub use value_iteration::ValueIteration;

#[cfg(test)]
mod tests;

use crate::value_iteration::non_determinism::NonDeterminismResolver;
use probabilistic_models::base_model::Mdp;
use probabilistic_models::{ChoiceIndex, Ix};
use typed_index_collections::To1;

#[derive(Clone, Debug)]
pub struct SolveStatistics {
    pub vi_iterations: usize,
    pub verification_iterations: usize,
}

pub trait SubModelSolver {
    fn create(max_size: usize) -> Self;
    fn solve_with_statistics<'a, ND: NonDeterminismResolver, I: Ix>(
        &'a mut self,
        mdp: &Mdp<I>,
        choice_exit_values: &To1<ChoiceIndex<I>, f64>,
        eps: f64,
        max_value: f64,
    ) -> (&'a [f64], SolveStatistics);

    fn solve<'a, ND: NonDeterminismResolver, I: Ix>(
        &'a mut self,
        mdp: &Mdp<I>,
        choice_exit_values: &To1<ChoiceIndex<I>, f64>,
        eps: f64,
        max_value: f64,
    ) -> &'a [f64] {
        let (values, _) =
            self.solve_with_statistics::<ND, _>(mdp, choice_exit_values, eps, max_value);
        values
    }
    fn requires_unique_fixed_point() -> bool;
}
