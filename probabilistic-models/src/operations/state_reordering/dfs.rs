use super::{PermuteStates, PermuteStatesWithContext, StateOrdering};
use crate::Model;
use crate::traits::{ReadInitialStates, ReadStateSpace, StateSet};
use num_traits::Bounded;
use typed_index_collections::{Index, RawIndex, To1};

// In which order the successors of a state are expanded
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SuccessorOrder {
    // No guarantee on the order is given
    Arbitrary,
    // The states that are reached by actions with the highest probability are expanded first.
    HighestProbabilityFirst,
}

impl<
    SI: Index,
    CI: Index,
    M: PermuteStates<StateIndex = SI>
        + ReadStateSpace<StateIndex = SI, ChoiceIndex = CI>
        + ReadInitialStates<StateIdx = SI>,
    Ini: PermuteStatesWithContext<SI, CI>,
    ChLabel: PermuteStatesWithContext<SI, CI>,
    BrLabel: PermuteStatesWithContext<SI, CI>,
    Obs: PermuteStatesWithContext<SI, CI>,
    APs: PermuteStatesWithContext<SI, CI>,
    Rew: PermuteStatesWithContext<SI, CI>,
    Ann: PermuteStatesWithContext<SI, CI>,
    StateVals: PermuteStatesWithContext<SI, CI>,
    Preds: PermuteStatesWithContext<SI, CI>,
> Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    #[must_use]
    pub fn reorder_dfs(&self, successor_order: SuccessorOrder) -> Self {
        assert!(
            self.states().len() < SI::RawType::max_value().as_usize(),
            "dfs reordering needs the maximal value of the state index type to be larger than the number of states of the model"
        );
        let state_ordering = compute_dfs_state_ordering(&self.base, successor_order);

        let base_model = self.base.permute_states(&state_ordering);
        Model {
            initial: self.initial.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            choice_labels: self.choice_labels.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            branch_labels: self.branch_labels.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            observations: self.observations.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            atomic_propositions: self.atomic_propositions.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            rewards: self.rewards.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            annotations: self.annotations.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            state_valuations: self.state_valuations.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            predecessors: self.predecessors.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            base: base_model,
        }
    }
}

fn compute_dfs_state_ordering<
    SI: Index,
    M: ReadStateSpace<StateIndex = SI> + ReadInitialStates<StateIdx = SI>,
>(
    model: &M,
    successor_order: SuccessorOrder,
) -> StateOrdering<SI> {
    let unassigned = SI::from_raw(SI::RawType::max_value());

    let mut old_to_new: To1<SI, SI> = To1::with_entries(vec![unassigned; model.states().len()]);
    let mut new_to_old: To1<SI, SI> = To1::with_capacity(model.states().len());
    let mut state_counter = SI::from_raw(SI::RawType::zero());

    let mut open_states = Vec::new();
    for initial in model.initial_states().iter() {
        open_states.push(initial);
    }
    // Allocate a vector here for SuccessorOrder::HighestProbabilityFirst to reuse
    let mut successor_buffer = Vec::new();
    while let Some(state) = open_states.pop() {
        old_to_new[state] = state_counter;
        new_to_old.add_checked(state_counter, state);
        state_counter += SI::RawType::one();

        match successor_order {
            SuccessorOrder::Arbitrary => {
                for choice in model.choices_of_state(state) {
                    for branch in model.branches_of_choice(choice) {
                        let destination = model.branch_destination(branch);
                        if old_to_new[destination] == unassigned {
                            open_states.push(model.branch_destination(branch));
                        }
                    }
                }
            }
            SuccessorOrder::HighestProbabilityFirst => {
                successor_buffer.clear();
                for choice in model.choices_of_state(state) {
                    for branch in model.branches_of_choice(choice) {
                        let destination = model.branch_destination(branch);
                        if old_to_new[destination] == unassigned {
                            successor_buffer.push((
                                model.branch_probability(branch),
                                model.branch_destination(branch),
                            ));
                        }
                    }
                }
                successor_buffer.sort_by(|(p1, _), (p2, _)| p2.partial_cmp(p1).unwrap());
                for &(_, dest) in &successor_buffer {
                    open_states.push(dest);
                }
            }
        }
    }

    StateOrdering {
        old_to_new,
        new_to_old,
    }
}
