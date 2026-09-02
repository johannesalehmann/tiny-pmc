mod min_max;
use crate::buffer::ZeroedBuffer;
use crate::sccs::{ExclusionList, SccEntryIndex, SccIndex, Sccs};
use min_max::*;
use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadOwners, ReadPredecessors, ReadStateSpace,
};
use probabilistic_models::typed_index_collections::To1;

pub fn value_iteration_max<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<StateIdx = <M as ReadStateSpace>::StateIdx>,
>(
    model: &M,
    goal: <M as ReadAtomicPropositions>::AnnotationIdx,
    eps: f64,
) -> To1<<M as ReadStateSpace>::StateIdx, f64> {
    value_iteration_internal(model, goal, eps, Maximiser::default())
}
pub fn value_iteration_min<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<StateIdx = <M as ReadStateSpace>::StateIdx>,
>(
    model: &M,
    goal: <M as ReadAtomicPropositions>::AnnotationIdx,
    eps: f64,
) -> To1<<M as ReadStateSpace>::StateIdx, f64> {
    // TODO: Collapse MECs!
    value_iteration_internal(model, goal, eps, Minimiser::default())
}
pub fn value_iteration_game<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadOwners<StateIdx = <M as ReadStateSpace>::StateIdx, OwnerType = TwoPlayer>,
>(
    model: &M,
    goal: <M as ReadAtomicPropositions>::AnnotationIdx,
    eps: f64,
) -> To1<<M as ReadStateSpace>::StateIdx, f64> {
    value_iteration_internal(
        model,
        goal,
        eps,
        PlayerOneMaximisesPlayerTwoMinimises::default(),
    )
}

fn value_iteration_internal<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<StateIdx = <M as ReadStateSpace>::StateIdx>,
    MinMax: ValueComparator<Model = M>,
>(
    model: &M,
    goal: <M as ReadAtomicPropositions>::AnnotationIdx,
    eps: f64,
    min_max: MinMax,
) -> To1<<M as ReadStateSpace>::StateIdx, f64> {
    let buffer = ZeroedBuffer::new(model.states().len());
    let mut values = buffer.into_values();
    let mut target_states = Vec::new();
    for state in model.states() {
        if model.is_atomic_proposition_set(state, goal) {
            target_states.push(state);
            values[state] = 1.0;
        } else {
            values[state] = min_max.initial_value(state, model);
        }
    }

    let excluded = ExclusionList::new(&target_states);

    // TODO: Adapt these types to those used for state indices in the model
    let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> = Sccs::compute(model, &excluded);

    for scc_index in sccs.reverse_topological_ordering() {
        loop {
            let mut largest_change = 0.0;
            for entry in sccs.entries(scc_index) {
                let state = sccs.entry_to_state(entry);

                let mut best_value = min_max.initial_value(state, model);
                for choice in model.choices_of_state(state) {
                    let mut value = 0.0;
                    for branch in model.branches_of_choice(choice) {
                        value += model.branch_probability(branch)
                            * values[model.branch_destination(branch)];
                    }
                    if min_max.is_better(state, model, best_value, value) {
                        best_value = value;
                    }
                }

                let absolute_error = best_value - values[state];
                let relative_error = absolute_error / best_value;
                if relative_error > largest_change {
                    largest_change = relative_error;
                }
                values[state] = best_value;
            }
            if largest_change < eps {
                break;
            }
        }
    }

    values
}
