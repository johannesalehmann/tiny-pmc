mod optimistic_value_iteration;
pub use optimistic_value_iteration::OptimisticValueIteration;

mod value_iteration;
pub use value_iteration::ValueIteration;

use crate::value_iteration::non_determinism::NonDeterminism;
use probabilistic_models::base_model::Mdp;
use typed_index_collections::{Index, To1};

pub trait SubGameSolver {
    fn create(max_size: usize) -> Self;
    fn solve<'a, ND: NonDeterminism, SI: Index, CI: Index, BI: Index>(
        &'a mut self,
        mdp: &Mdp<SI, CI, BI>,
        choice_exit_values: &To1<CI, f64>,
        eps: f64,
    ) -> &'a [f64];
}
