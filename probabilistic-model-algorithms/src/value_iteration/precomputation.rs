use probabilistic_models::traits::{ReadAtomicPropositions, ReadPredecessors, ReadStateSpace};
use typed_index_collections::To1;

// The functions in this file implement Algorithms 4.1 to 4.4 from "Forejt, V., Kwiatkowska, M.,
// Norman, G., & Parker, D. (2011). Automated verification techniques for probabilistic systems"

pub fn s0_min<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
>(
    model: &M,
    goal: M::APIdx,
) -> To1<<M as ReadStateSpace>::StateIdx, bool> {
    let mut open_list = Vec::new();
    let mut result = To1::with_entries(vec![true; model.states().len()]);
    // For every state, the number of its choices that do not (yet) have a branch into the
    // complement of `result`. A state leaves `result` only once this counter reaches zero, i.e.
    // once *every* choice of the state reaches the complement with positive probability.
    let mut remaining_choices: To1<<M as ReadStateSpace>::StateIdx, u32> =
        To1::with_capacity(model.states().len());
    // Records the choices that are already known to have a branch into the complement of `result`,
    // so that a choice with several such branches decrements its state's counter only once.
    let mut choice_reaches_complement: To1<<M as ReadStateSpace>::ChoiceIdx, bool> =
        To1::with_entries(vec![false; model.choices().len()]);
    for state in model.states() {
        remaining_choices.add(model.choices_of_state(state).len() as u32);
        if model.is_atomic_proposition_set(state, goal) {
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
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
>(
    model: &M,
    goal: M::APIdx,
    s0_states: &To1<<M as ReadStateSpace>::StateIdx, bool>,
) -> To1<<M as ReadStateSpace>::StateIdx, bool> {
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
            if result[predecessor_state]
                && !model.is_atomic_proposition_set(predecessor_state, goal)
            {
                result[predecessor_state] = false;
                open_list.push(predecessor_state);
            }
        }
    }
    result
}

pub fn s0_max<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
>(
    model: &M,
    goal: M::APIdx,
) -> To1<<M as ReadStateSpace>::StateIdx, bool> {
    let mut open_list = Vec::new();
    let mut result = To1::with_entries(vec![true; model.states().len()]);
    for state in model.states() {
        if model.is_atomic_proposition_set(state, goal) {
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
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
>(
    model: &M,
    goal: M::APIdx,
) -> To1<<M as ReadStateSpace>::StateIdx, bool> {
    let mut result = To1::with_entries(vec![true; model.states().len()]);
    let mut inner_buffer = To1::with_entries(vec![false; model.states().len()]);
    let mut open_list = Vec::new();
    let mut stays_in_result = To1::with_entries(vec![true; model.choices().len()]);
    loop {
        open_list.clear();
        for state in model.states() {
            if model.is_atomic_proposition_set(state, goal) {
                inner_buffer[state] = true;
                open_list.push(state);
            } else {
                inner_buffer[state] = false;
            }
        }
        for choice in model.choices() {
            // If stays_in_result[choice] is already false, it still is in this iteration, because
            // result only ever shrinks.
            if stays_in_result[choice] {
                let mut all_inside = true;
                for branch in model.branches_of_choice(choice) {
                    if !result[model.branch_destination(branch)] {
                        all_inside = false;
                        break;
                    }
                }
                stays_in_result[choice] = all_inside;
            }
        }
        while let Some(state) = open_list.pop() {
            for predecessor in model.predecessors_of_state(state) {
                let predecessor_choice = model.choice_of_predecessor(predecessor);
                let predecessor_state = model.state_of_choice(predecessor_choice);
                if stays_in_result[predecessor_choice] {
                    if !inner_buffer[predecessor_state] {
                        inner_buffer[predecessor_state] = true;
                        open_list.push(predecessor_state);
                    }
                }
            }
        }

        let mut any_change = false;
        for (result, inner) in result.iter_mut().zip(inner_buffer.iter()) {
            if *result != *inner {
                any_change = true;
                *result = *inner;
            }
        }
        if !any_change {
            break;
        }
    }
    result
}
