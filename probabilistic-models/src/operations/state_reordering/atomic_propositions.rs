use super::{PermuteStates, StateOrdering};
use crate::annotations::{AtomicPropositions, TypedAnnotation};
use typed_index_collections::{Index, To1};

impl<SI: Index, AI: Index, AEI: Index> PermuteStates for AtomicPropositions<AI, SI, AEI> {
    type StateIndex = SI;

    fn permute_states(&self, ordering: &StateOrdering<SI>) -> Self {
        let mut atomic_propositions = AtomicPropositions::new();
        for (name, old_annotations) in self {
            let mut new_annotations =
                TypedAnnotation::with_identity_distribution_and_entries(To1::with_entries(vec![
                    false;
                    old_annotations.values().len()
                ]));
            for (old, new) in ordering.old_to_new.enumerate() {
                new_annotations[*new] = old_annotations[old];
            }

            atomic_propositions.add_entry(name.to_string(), new_annotations);
        }
        atomic_propositions
    }
}
