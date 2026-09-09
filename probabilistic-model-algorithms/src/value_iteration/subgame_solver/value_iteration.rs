use super::SubGameSolver;
use crate::value_iteration::non_determinism::NonDeterminism;
use probabilistic_models::base_model::Mdp;
use probabilistic_models::traits::ReadStateSpace;
use typed_index_collections::{Index, RawIndex, To1};

pub struct ValueIteration {
    pub(crate) values: Vec<f64>,
}

impl ValueIteration {
    // Solves the MDP without resetting the values vector. This can be used to do warm starts of the
    // value iteration
    pub fn solve_raw<'a, ND: NonDeterminism, SI: Index, CI: Index, BI: Index>(
        &'a mut self,
        mdp: &Mdp<SI, CI, BI>,
        choice_exit_values: &To1<CI, f64>,
        eps: f64,
    ) -> &'a [f64] {
        let mut values = &mut self.values[0..mdp.states().len()];
        loop {
            let mut converged = true;
            // Iterate states manually instead of relying on built-in functions such as
            // choices_of_state() because this is about 15% faster:
            let mut current_choice = CI::default();
            let mut current_branch = BI::default();
            let mut choices = mdp.choice_to_branch.entries_raw().iter();
            let mut branches = mdp
                .branch_probabilities
                .iter()
                .zip(mdp.branch_destinations.iter());
            let mut choice_exit_values = choice_exit_values.iter();

            for (state, &last_choice) in mdp.state_to_choice.entries_raw().iter().enumerate() {
                let mut best_value = ND::neutral_value();
                while current_choice < last_choice {
                    let last_branch = *choices.next().unwrap();
                    let mut value = *choice_exit_values.next().unwrap();
                    while current_branch < last_branch {
                        let (probability, destination) = branches.next().unwrap();
                        value += probability * values[destination.raw().as_usize()];
                        current_branch += BI::RawType::one();
                    }
                    if ND::is_better(best_value, value) {
                        best_value = value;
                    }
                    current_choice += CI::RawType::one();
                }

                if converged {
                    let absolute_error = best_value - values[state];
                    // The condition is equivalent to `absolute_error / best_value >= eps`:
                    if absolute_error >= eps * best_value {
                        converged = false;
                    }
                }
                values[state] = best_value;
            }
            if converged {
                break;
            }
        }
        values
        // &self.values[0..mdp.states().len()]
    }
}

impl SubGameSolver for ValueIteration {
    fn create(max_size: usize) -> Self {
        Self {
            values: vec![0.0; max_size],
        }
    }

    fn solve<'a, ND: NonDeterminism, SI: Index, CI: Index, BI: Index>(
        &'a mut self,
        mdp: &Mdp<SI, CI, BI>,
        choice_exit_values: &To1<CI, f64>,
        eps: f64,
    ) -> &'a [f64] {
        // TODO: Only reset the part that was actually dirtied on the previous call?
        //  See also the comment for optimistic value iteration which found that only resetting the
        //  beginning might actually be slower.
        self.values.fill(0.0);
        self.solve_raw::<ND, _, _, _>(mdp, choice_exit_values, eps)
    }
}
