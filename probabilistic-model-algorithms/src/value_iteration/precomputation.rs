use crate::value_iteration::min_max::{AttractorBehaviour, ValueComparator};
use probabilistic_models::traits::{ReadAtomicPropositions, ReadPredecessors, ReadStateSpace};
use typed_index_collections::{Index, To1, To1BoolValues};

// TODO: The pre-computation currently only computes sure reachability, not almost-sure
//  reachability. For the p1 case, this leads to a significant amount of states being missed.
//  "Automated Verification Techniques for Probabilistic Systems" describes algorithms to handle
//  almost-sure reachability in Algorithms 1 to 4 (Section 4.1).

// TODO: We could save some memory and allocations by merging these two vectors
pub struct P0P1States<StateIdx: Index> {
    p_non_zero_states: To1<StateIdx, bool>,
    p1_states: To1<StateIdx, bool>,
}

impl<StateIdx: Index> P0P1States<StateIdx> {
    pub fn is_state_p0(&self, state: StateIdx) -> bool {
        !self.p_non_zero_states[state]
    }
    pub fn is_state_p1(&self, state: StateIdx) -> bool {
        self.p1_states[state]
    }
    pub fn p0_states(&self) -> To1BoolValues<StateIdx, &To1<StateIdx, bool>> {
        self.p_non_zero_states.false_values()
    }
    pub fn p1_states(&self) -> To1BoolValues<StateIdx, &To1<StateIdx, bool>> {
        self.p1_states.true_values()
    }
}

pub fn compute_p_0_and_p1_states<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
    MinMax: ValueComparator<Model = M>,
>(
    model: &M,
    min_max: MinMax,
    goal: M::APIdx,
) -> P0P1States<<M as ReadStateSpace>::StateIdx> {
    let mut p_non_zero_choice_counts: To1<<M as ReadStateSpace>::ChoiceIdx, usize> =
        To1::with_capacity(model.choices().len());
    let mut p1_choice_counts: To1<<M as ReadStateSpace>::ChoiceIdx, usize> =
        To1::with_capacity(model.choices().len());

    let mut p_non_zero_state_counts: To1<<M as ReadStateSpace>::StateIdx, usize> =
        To1::with_capacity(model.states().len());
    let mut p1_state_counts: To1<<M as ReadStateSpace>::StateIdx, usize> =
        To1::with_capacity(model.states().len());

    let mut open_state_list = Vec::new();

    for state in model.states() {
        if model.is_atomic_proposition_set(state, goal) {
            open_state_list.push(state);
            p_non_zero_state_counts.add_checked(state, 0);
            p1_state_counts.add_checked(state, 0);
        } else {
            let choices = model.choices_of_state(state);
            let attractor_count = match min_max.attractor_behaviour(state, model) {
                AttractorBehaviour::TakesHighestValueChoice => 1,
                AttractorBehaviour::TakesLowestValueChoice => choices.len(),
            };
            p_non_zero_state_counts.add_checked(state, attractor_count);
            p1_state_counts.add_checked(state, attractor_count);
        }
    }

    for choice in model.choices() {
        p_non_zero_choice_counts.add_checked(choice, 1);
        p1_choice_counts.add_checked(choice, model.branches_of_choice(choice).len());
    }

    let p_non_zero_states = attract(
        model,
        &mut p_non_zero_state_counts,
        &mut p_non_zero_choice_counts,
        open_state_list.clone(),
    );
    let p1_states = attract(
        model,
        &mut p1_state_counts,
        &mut p1_choice_counts,
        open_state_list,
    );
    P0P1States {
        p_non_zero_states,
        p1_states,
    }
}

fn attract<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
>(
    model: &M,
    state_counts: &mut To1<<M as ReadStateSpace>::StateIdx, usize>,
    choice_counts: &mut To1<<M as ReadStateSpace>::ChoiceIdx, usize>,
    mut open_state_list: Vec<<M as ReadStateSpace>::StateIdx>,
) -> To1<<M as ReadStateSpace>::StateIdx, bool> {
    let mut result = To1::with_entries(vec![false; model.states().len()]);
    while let Some(state) = open_state_list.pop() {
        result[state] = true;
        for predecessor in model.predecessors_of_state(state) {
            let choice = model.choice_of_branch(model.branch_of_predecessor(predecessor));
            if choice_counts[choice] > 0 {
                choice_counts[choice] -= 1;
                if choice_counts[choice] == 0 {
                    let predecessor_state = model.state_of_choice(choice);
                    if state_counts[predecessor_state] > 0 {
                        state_counts[predecessor_state] -= 1;
                        if state_counts[predecessor_state] == 0 {
                            open_state_list.push(predecessor_state);
                        }
                    }
                }
            }
        }
    }
    result
}
