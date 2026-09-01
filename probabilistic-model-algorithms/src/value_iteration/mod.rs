use crate::buffer::ZeroedBuffer;
use crate::sccs::{ExclusionList, SccEntryIndex, SccIndex, Sccs};
use probabilistic_models::traits::{ReadAtomicPropositions, ReadPredecessors, ReadStateSpace};
use probabilistic_models::typed_index_collections::To1;

pub fn value_iteration<
    M: ReadStateSpace
        + ReadAtomicPropositions<StateIdx = <M as ReadStateSpace>::StateIdx>
        + ReadPredecessors<StateIdx = <M as ReadStateSpace>::StateIdx>,
>(
    model: &M,
    goal: <M as ReadAtomicPropositions>::AnnotationIdx,
    eps: f64,
) -> To1<<M as ReadStateSpace>::StateIdx, f64> {
    let buffer = ZeroedBuffer::new(model.states().len());
    let mut values = buffer.into_values();

    let target_states = model
        .states()
        .into_iter()
        .filter(|s| model.is_atomic_proposition_set(*s, goal))
        .collect::<Vec<_>>();

    for &target_state in &target_states {
        values[target_state] = 1.0;
    }
    let excluded = ExclusionList::new(&target_states);

    // TODO: Adapt these types to those used for state indices in the model
    let sccs: Sccs<SccIndex<usize>, SccEntryIndex<usize>, _> = Sccs::compute(model, &excluded);

    for scc_index in sccs.reverse_topological_ordering() {
        loop {
            let mut largest_change = 0.0;
            for entry in sccs.entries(scc_index) {
                let state = sccs.entry_to_state(entry);

                let mut best_value = 0.0;
                for choice in model.choices_of_state(state) {
                    let mut value = 0.0;
                    for branch in model.branches_of_choice(choice) {
                        value += model.branch_probability(branch)
                            * values[model.branch_destination(branch)];
                    }
                    if value >= best_value {
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
