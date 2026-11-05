use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadChoiceDestinations, ReadInitialStates, StateSet,
};
use probabilistic_models::{AtomicPropositionIndex, ChoiceIndex, Ix, StateIndex};
use std::collections::VecDeque;
use typed_index_collections::To1;

pub fn is_reachable<M>(
    model: &M,
    objective: AtomicPropositionIndex<M::Ix>,
) -> ReachableResult<M::Ix>
where
    M: ReadChoiceDestinations + ReadInitialStates + ReadAtomicPropositions,
{
    let mut open_list = VecDeque::new();
    let mut predecessors = To1::with_entries(vec![Predecessor::Unvisited; model.states().len()]);
    for initial in model.initial_states().iter() {
        if !matches!(predecessors[initial], Predecessor::Unvisited) {
            continue;
        }
        predecessors[initial] = Predecessor::Initial;

        if model.is_atomic_proposition_set(initial, objective) {
            return ReachableResult::Reachable(get_path(&predecessors, initial));
        }

        open_list.push_back(initial);
    }

    while let Some(state) = open_list.pop_front() {
        for choice in model.choices_of_state(state) {
            let destination = model.choice_destination(choice);
            if matches!(predecessors[destination], Predecessor::Unvisited) {
                predecessors[destination] = Predecessor::Some { state, choice };

                if model.is_atomic_proposition_set(destination, objective) {
                    return ReachableResult::Reachable(get_path(&predecessors, destination));
                }

                open_list.push_back(destination);
            }
        }
    }

    ReachableResult::Unreachable
}

fn get_path<I: Ix>(
    predecessors: &To1<StateIndex<I>, Predecessor<I>>,
    target: StateIndex<I>,
) -> Path<I> {
    let mut states = vec![target];
    let mut choices = Vec::new();
    let mut current = target;
    while let Predecessor::Some { state, choice } = predecessors[current] {
        choices.push(choice);
        states.push(state);
        current = state;
    }

    states.reverse();
    choices.reverse();

    Path { states, choices }
}

pub enum ReachableResult<I: Ix> {
    Reachable(Path<I>),
    Unreachable,
}

/// A path through the model. `choices[i]` is the choice taken in `states[i]` to reach
/// `states[i + 1]`.
pub struct Path<I: Ix> {
    pub states: Vec<StateIndex<I>>,
    pub choices: Vec<ChoiceIndex<I>>,
}

#[derive(Clone, Copy)]
enum Predecessor<I: Ix> {
    Unvisited,
    Initial,
    Some {
        state: StateIndex<I>,
        choice: ChoiceIndex<I>,
    },
}
