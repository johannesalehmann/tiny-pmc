use super::{SubModelSolver, ValueIteration};
use crate::value_iteration::non_determinism::NonDeterminism;
use probabilistic_models::base_model::Mdp;
use probabilistic_models::traits::ReadStateSpace;
use typed_index_collections::{Index, RawIndex, To1};

// Relative amount by which an upper bound may increase (or fall below the corresponding lower
// bound) during verification before this is attributed to the bound being too small rather than
// to floating-point rounding.
const ROUNDING_TOLERANCE: f64 = 4.0 * f64::EPSILON;

pub struct OptimisticValueIteration {
    base_vi: ValueIteration,
    verification_bounds: Vec<(f64, f64)>,
}

impl SubModelSolver for OptimisticValueIteration {
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
        max_value: f64,
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
                // Rounding can push the lower bound slightly above `max_value`. We clamp it to
                // ensure that it does not exceed the upper bound.
                let value = values[state.raw().as_usize()].min(max_value);
                let upper = match value {
                    0.0 => 0.0,
                    v => (v * (1.0 + initial_eps)).min(max_value),
                };
                verification_bounds[state.raw().as_usize()] = (value, upper);
            }

            match verify_submodel_optimistic::<ND, _, _, _>(
                mdp,
                choice_exit_values,
                (1.0 / (2.0 * eps)).max(1.0) as usize,
                max_value,
                verification_bounds,
            ) {
                OptimisticValueIterationResult::UpperBoundVerified => {
                    write_midpoints(&mut self.base_vi.values, verification_bounds);
                    break &self.base_vi.values[..mdp.states().len()];
                }
                OptimisticValueIterationResult::UpperBoundRefuted { error } => {
                    if error > f64::EPSILON {
                        eps = error * 0.5;
                        write_lower_bounds(&mut self.base_vi.values, verification_bounds);
                    } else {
                        // The lower bound no longer improves, so further iterations would not change
                        // anything. Check whether it is a fixed point by checking whether it
                        // verifies as its own upper bound.
                        for (lower, upper) in verification_bounds.iter_mut() {
                            *upper = *lower;
                        }
                        match verify_submodel_optimistic::<ND, _, _, _>(
                            mdp,
                            choice_exit_values,
                            1,
                            max_value,
                            verification_bounds,
                        ) {
                            OptimisticValueIterationResult::UpperBoundVerified => {
                                write_midpoints(&mut self.base_vi.values, verification_bounds);
                            }
                            OptimisticValueIterationResult::UpperBoundRefuted { .. } => {
                                println!(
                                    "Warning: Optimistic value iteration failed to verify the \
                                upper bound for a sub-model with {} states due to floating-point \
                                rounding. The provided values might not be epsilon-correct.",
                                    mdp.states().len()
                                );
                                write_lower_bounds(&mut self.base_vi.values, verification_bounds);
                            }
                        }
                        break &self.base_vi.values[..mdp.states().len()];
                    }
                }
            }
        }
    }

    fn requires_unique_fixed_point() -> bool {
        true
    }
}

fn verify_submodel_optimistic<ND: NonDeterminism, NewSI: Index, NewCI: Index, NewBI: Index>(
    mdp: &Mdp<NewSI, NewCI, NewBI>,
    choice_exit_values: &To1<NewCI, f64>,
    max_steps: usize,
    max_value: f64,
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

            let new_lower_value = new_lower_value.min(max_value); // .max to guard against floating-point round-ups
            let (lower, upper) = verification_bounds[state];
            if new_lower_value > 0.0 {
                error = error.max((new_lower_value - lower) / new_lower_value);
            }
            verification_bounds[state].0 = new_lower_value;
            if new_upper_value < upper {
                all_up = false;
                verification_bounds[state].1 = new_upper_value;
            } else if new_upper_value > upper * (1.0 + ROUNDING_TOLERANCE) {
                all_down = false;
            }

            // Increases of the upper bound within the tolerance are ignored above, so the lower
            // bound may exceed the upper bound by the same tolerance.
            if new_upper_value < new_lower_value * (1.0 - ROUNDING_TOLERANCE) {
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

fn write_midpoints(values: &mut [f64], verification_bounds: &[(f64, f64)]) {
    for (value, &(lower, upper)) in values.iter_mut().zip(verification_bounds) {
        *value = 0.5 * (lower + upper);
    }
}

fn write_lower_bounds(values: &mut [f64], verification_bounds: &[(f64, f64)]) {
    for (value, &(lower, _)) in values.iter_mut().zip(verification_bounds) {
        *value = lower;
    }
}

enum OptimisticValueIterationResult {
    UpperBoundVerified,
    UpperBoundRefuted { error: f64 },
}
