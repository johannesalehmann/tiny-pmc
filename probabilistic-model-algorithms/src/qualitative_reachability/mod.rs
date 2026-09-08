use crate::state_description::StateDescription;
use probabilistic_models::traits::{ReadAtomicPropositions, ReadPredecessors, ReadStateSpace};
use typed_index_collections::To1;
// The functions in this file implement Algorithms 4.1 to 4.4 from "Forejt, V., Kwiatkowska, M.,
// Norman, G., & Parker, D. (2011). Automated verification techniques for probabilistic systems"

pub fn s0_min<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = M::StateIndex>
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        >,
>(
    model: &M,
    goal: &StateDescription<M>,
) -> To1<M::StateIndex, bool> {
    let mut open_list = Vec::new();
    let mut result = To1::with_entries(vec![true; model.states().len()]);
    // For every state, the number of its choices that do not (yet) have a branch into the
    // complement of `result`. A state leaves `result` only once this counter reaches zero, i.e.
    // once *every* choice of the state reaches the complement with positive probability.
    let mut remaining_choices: To1<M::StateIndex, u32> =
        To1::with_capacity(model.states().len());
    // Records the choices that are already known to have a branch into the complement of `result`,
    // so that a choice with several such branches decrements its state's counter only once.
    let mut choice_reaches_complement: To1<M::ChoiceIndex, bool> =
        To1::with_entries(vec![false; model.choices().len()]);
    for state in model.states() {
        remaining_choices.add(model.choices_of_state(state).len() as u32);
        if goal.is_set(state) {
            result[state] = false;
            open_list.push(state);
        }
    }
    while let Some(state) = open_list.pop() {
        for predecessor in model.predecessors_of_state(state) {
            let choice = model.choice_of_predecessor(predecessor);
            if choice_reaches_complement[choice] {
                continue;
            }
            choice_reaches_complement[choice] = true;
            let predecessor_state = model.state_of_choice(choice);
            if !result[predecessor_state] {
                continue;
            }
            remaining_choices[predecessor_state] -= 1;
            if remaining_choices[predecessor_state] == 0 {
                result[predecessor_state] = false;
                open_list.push(predecessor_state);
            }
        }
    }
    result
}

pub fn s1_min<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = M::StateIndex>
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        >,
>(
    model: &M,
    goal: &StateDescription<M>,
    s0_states: &To1<M::StateIndex, bool>,
) -> To1<M::StateIndex, bool> {
    let mut result = To1::with_capacity(model.states().len());
    let mut open_list = Vec::new();
    for (state, &value) in s0_states.enumerate() {
        if value {
            open_list.push(state);
        }
        result.add(!value);
    }
    while let Some(state) = open_list.pop() {
        for predecessor in model.predecessors_of_state(state) {
            let predecessor_state = model.source_state_of_predecessor(predecessor);
            if result[predecessor_state] && !goal.is_set(predecessor_state) {
                result[predecessor_state] = false;
                open_list.push(predecessor_state);
            }
        }
    }
    result
}

pub fn s0_max<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = M::StateIndex>
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        >,
>(
    model: &M,
    goal: &StateDescription<M>,
) -> To1<M::StateIndex, bool> {
    let mut open_list = Vec::new();
    let mut result = To1::with_entries(vec![true; model.states().len()]);
    for state in model.states() {
        if goal.is_set(state) {
            result[state] = false;
            open_list.push(state);
        }
    }
    while let Some(state) = open_list.pop() {
        for predecessor in model.predecessors_of_state(state) {
            let predecessor_state = model.source_state_of_predecessor(predecessor);
            if result[predecessor_state] {
                result[predecessor_state] = false;
                open_list.push(predecessor_state);
            }
        }
    }
    result
}

// TODO: There are asymptotically faster algorithms for this that might be worth investigating.
pub fn s1_max<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = M::StateIndex>
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        >,
>(
    model: &M,
    goal: &StateDescription<M>,
) -> To1<M::StateIndex, bool> {
    let mut result = To1::with_entries(vec![true; model.states().len()]);
    let mut inner_buffer = To1::with_entries(vec![false; model.states().len()]);
    let mut open_list = Vec::new();
    // Tracks whether all branches of this choice stay in result.
    let mut stays_in_result = To1::with_entries(vec![true; model.choices().len()]);
    let goal_states: Vec<_> = model
        .states()
        .into_iter()
        .filter(|&state| goal.is_set(state))
        .collect();
    loop {
        inner_buffer.fill(false);
        for &state in &goal_states {
            inner_buffer[state] = true;
            open_list.push(state);
        }
        // Repeatedly add states to `inner_buffer` that reach `goal_states` via a transition that
        // stays in `result`.
        while let Some(state) = open_list.pop() {
            for predecessor in model.predecessors_of_state(state) {
                let predecessor_choice = model.choice_of_predecessor(predecessor);
                if !stays_in_result[predecessor_choice] {
                    continue;
                }
                let predecessor_state = model.state_of_choice(predecessor_choice);
                if !inner_buffer[predecessor_state] {
                    inner_buffer[predecessor_state] = true;
                    open_list.push(predecessor_state);
                }
            }
        }

        // Update `stays_in_result`: Remove choices with a branch to a state that is both in
        // `result` and not in `inner_buffer`
        let mut any_choices_removed = false;
        for state in model.states() {
            if !result[state] || inner_buffer[state] {
                continue;
            }
            result[state] = false;
            for predecessor in model.predecessors_of_state(state) {
                let predecessor_choice = model.choice_of_predecessor(predecessor);
                if stays_in_result[predecessor_choice] {
                    stays_in_result[predecessor_choice] = false;
                    any_choices_removed = true;
                }
            }
        }
        if !any_choices_removed {
            break;
        }
    }
    result
}
