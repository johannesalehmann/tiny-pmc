use super::{SubGameSolver, ValueIteration};
use crate::value_iteration::non_determinism::NonDeterminism;
use probabilistic_models::base_model::Mdp;
use probabilistic_models::traits::ReadStateSpace;
use typed_index_collections::{Index, RawIndex, To1};

pub struct OptimisticValueIteration {
    base_vi: ValueIteration,
    verification_bounds: Vec<(f64, f64)>,
}

impl SubGameSolver for OptimisticValueIteration {
    fn create(max_size: usize) -> Self {
        Self {
            base_vi: ValueIteration::create(max_size),
            verification_bounds: vec![(0.0, 0.0); max_size],
        }
    }

    fn solve<'a, ND: NonDeterminism, SI: Index, CI: Index, BI: Index>(
        &'a mut self,
        mdp: &Mdp<SI, CI, BI>,
        choice_exit_values: &To1<CI, f64>,
        mut eps: f64,
    ) -> &'a [f64] {
        let initial_eps = eps;
        // TODO: Surprisingly, the following is slower than just zeroing out the entire buffer.
        //  self.base_vi.values[0..mdp.states().len()].fill(0.0);
        //  However, this is based on fairly limited experiments and needs more investigation.

        self.base_vi.values.fill(0.0);

        loop {
            let values = self
                .base_vi
                .solve_raw::<ND, _, _, _>(mdp, choice_exit_values, eps);

            let verification_bounds = &mut self.verification_bounds[..mdp.states().len()];
            for state in mdp.states() {
                let value = values[state.raw().as_usize()];
                let upper = match value {
                    0.0 => 0.0,
                    v => (v * (1.0 + initial_eps)).min(1.0),
                };
                verification_bounds[state.raw().as_usize()] = (value, upper);
            }

            match verify_subgame_optimistic::<ND, _, _, _>(
                mdp,
                choice_exit_values,
                (1.0 / (2.0 * eps)).max(1.0) as usize,
                verification_bounds,
            ) {
                OptimisticValueIterationResult::UpperBoundVerified => {
                    for (_, (value, (lower, upper))) in mdp.states().into_iter().zip(
                        self.base_vi
                            .values
                            .iter_mut()
                            .zip(verification_bounds.iter()),
                    ) {
                        *value = 0.5 * (lower + upper);
                    }
                    break &self.base_vi.values[..mdp.states().len()];
                }
                OptimisticValueIterationResult::UpperBoundRefuted { error } => {
                    eps = error * 0.5;
                    for (_, (value, (lower, _))) in mdp.states().into_iter().zip(
                        self.base_vi
                            .values
                            .iter_mut()
                            .zip(verification_bounds.iter()),
                    ) {
                        *value = *lower;
                    }
                }
            }
        }
    }
}

fn verify_subgame_optimistic<ND: NonDeterminism, NewSI: Index, NewCI: Index, NewBI: Index>(
    mdp: &Mdp<NewSI, NewCI, NewBI>,
    choice_exit_values: &To1<NewCI, f64>,
    max_steps: usize,
    verification_bounds: &mut [(f64, f64)],
) -> OptimisticValueIterationResult {
    let mut error: f64 = 0.0;
    for _ in 0..max_steps {
        let mut all_up = true;
        let mut all_down = true;
        error = 0.0;
        let mut current_choice = NewCI::default();
        let mut current_branch = NewBI::default();
        let mut choices = mdp.choice_to_branch.entries_raw().iter();
        let mut branches = mdp
            .branch_probabilities
            .iter()
            .zip(mdp.branch_destinations.iter());
        let mut choice_exit_values = choice_exit_values.iter();
        for (state, &last_choice) in mdp.state_to_choice.entries_raw().iter().enumerate() {
            let mut new_lower_value = ND::neutral_value();
            let mut new_upper_value = ND::neutral_value();

            while current_choice < last_choice {
                let last_branch = *choices.next().unwrap();
                let exit_value = *choice_exit_values.next().unwrap();
                let mut lower_value = exit_value;
                let mut upper_value = exit_value;
                while current_branch < last_branch {
                    let (probability, destination) = branches.next().unwrap();
                    let (lower, upper) = verification_bounds[destination.raw().as_usize()];
                    lower_value += probability * lower;
                    upper_value += probability * upper;
                    current_branch += NewBI::RawType::one();
                }
                if ND::is_better(new_lower_value, lower_value) {
                    new_lower_value = lower_value;
                }
                if ND::is_better(new_upper_value, upper_value) {
                    new_upper_value = upper_value;
                }
                current_choice += NewCI::RawType::one();
            }

            let (lower, upper) = verification_bounds[state];
            if new_lower_value > 0.0 {
                error = error.max((new_lower_value - lower) / new_lower_value);
            }
            verification_bounds[state].0 = new_lower_value;
            if new_upper_value < upper {
                all_up = false;
                verification_bounds[state].1 = new_upper_value;
            } else if new_upper_value > upper {
                all_down = false;
            }

            if new_upper_value < new_lower_value {
                return OptimisticValueIterationResult::UpperBoundRefuted { error };
            }
        }

        if all_down {
            return OptimisticValueIterationResult::UpperBoundVerified;
        } else if all_up {
            return OptimisticValueIterationResult::UpperBoundRefuted { error };
        }
    }
    OptimisticValueIterationResult::UpperBoundRefuted { error }
}

enum OptimisticValueIterationResult {
    UpperBoundVerified,
    UpperBoundRefuted { error: f64 },
}
