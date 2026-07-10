use crate::annotations::{AtomicPropositions, Distribution, TypedAnnotation};
use crate::{AnnotationIndex, RawIndex, StateIndex};
use std::marker::PhantomData;
use typed_index_collections::Index;

pub trait AtomicPropositionBuilder {
    type AtomicPropositions;
    type AnnotationIdx: Index;
    type StateIdx: Index;

    fn stores_atomic_propositions() -> bool;
    fn register_atomic_proposition(&mut self, id: String) -> Self::AnnotationIdx;
    fn set_value(&mut self, index: Self::AnnotationIdx, state: Self::StateIdx, value: bool);
    fn into_atomic_propositions(self) -> Self::AtomicPropositions;
}

#[derive(Default)]
pub struct UntrackedAtomicPropositionBuilder<AnnotationIdx: Index, StateIdx: Index> {
    _phantom_data: PhantomData<(AnnotationIdx, StateIdx)>,
}

impl<AnnotationIdx: Index, StateIdx: Index> AtomicPropositionBuilder
    for UntrackedAtomicPropositionBuilder<AnnotationIdx, StateIdx>
{
    type AtomicPropositions = ();
    type AnnotationIdx = AnnotationIdx;
    type StateIdx = StateIdx;

    fn stores_atomic_propositions() -> bool {
        false
    }

    fn register_atomic_proposition(&mut self, id: String) -> Self::AnnotationIdx {
        panic!("Cannot register atomic propositions when using `UntrackedAtomicPropositionBuilder`")
    }

    fn set_value(&mut self, id: Self::AnnotationIdx, state: Self::StateIdx, value: bool) {
        panic!("Cannot store atomic propositions when using `UntrackedAtomicPropositionBuilder`")
    }

    fn into_atomic_propositions(self) -> Self::AtomicPropositions {
        ()
    }
}

#[derive(Default)]
pub struct AtomicPropositionVectorsBuilder<
    AnnotationIdx: Index,
    StateIdx: Index,
    AnnotationEntryIdx: Index,
> {
    atomic_propositions: AtomicPropositions<AnnotationIdx, StateIdx, AnnotationEntryIdx>,
}

impl<AnnotationIdx: Index, StateIdx: Index, AnnotationEntryIdx: Index> AtomicPropositionBuilder
    for AtomicPropositionVectorsBuilder<AnnotationIdx, StateIdx, AnnotationEntryIdx>
{
    type AtomicPropositions = AtomicPropositions<AnnotationIdx, StateIdx, AnnotationEntryIdx>;
    type AnnotationIdx = AnnotationIdx;
    type StateIdx = StateIdx;

    fn stores_atomic_propositions() -> bool {
        true
    }

    fn register_atomic_proposition(&mut self, id: String) -> Self::AnnotationIdx {
        self.atomic_propositions
            .add_entry(id, TypedAnnotation::default())
    }

    fn set_value(&mut self, id: Self::AnnotationIdx, state: Self::StateIdx, value: bool) {
        self.atomic_propositions[id].add_value(state, value);
    }

    fn into_atomic_propositions(self) -> Self::AtomicPropositions {
        self.atomic_propositions
    }
}
