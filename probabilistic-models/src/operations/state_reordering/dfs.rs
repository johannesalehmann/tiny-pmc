use super::StateOrdering;
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

pub fn compute_dfs_state_ordering<
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
