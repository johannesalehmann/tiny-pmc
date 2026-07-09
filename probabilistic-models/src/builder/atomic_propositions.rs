use crate::annotations::{AtomicPropositions, Distribution, TypedAnnotation};
use crate::{AnnotationIndex, RawIndex, StateIndex};
use std::marker::PhantomData;

pub trait AtomicPropositionBuilder {
    type AtomicPropositions;
    type Index: RawIndex;

    fn stores_atomic_propositions() -> bool;
    fn register_atomic_proposition(&mut self, id: String) -> AnnotationIndex<Self::Index>;
    fn set_value(
        &mut self,
        index: AnnotationIndex<Self::Index>,
        state: StateIndex<Self::Index>,
        value: bool,
    );
}

#[derive(Default)]
pub struct UntrackedAtomicPropositionBuilder<I: RawIndex> {
    _phantom_data: PhantomData<I>,
}

impl<I: RawIndex> AtomicPropositionBuilder for UntrackedAtomicPropositionBuilder<I> {
    type AtomicPropositions = ();
    type Index = I;

    fn stores_atomic_propositions() -> bool {
        false
    }

    fn register_atomic_proposition(&mut self, id: String) -> AnnotationIndex<I> {
        panic!("Cannot register atomic propositions when using `UntrackedAtomicPropositionBuilder`")
    }

    fn set_value(&mut self, id: AnnotationIndex<I>, state: StateIndex<Self::Index>, value: bool) {
        panic!("Cannot store atomic propositions when using `UntrackedAtomicPropositionBuilder`")
    }
}

#[derive(Default)]
pub struct AtomicPropositionVectorsBuilder<I: RawIndex> {
    atomic_propositions: AtomicPropositions<I>,
}

impl<I: RawIndex> AtomicPropositionBuilder for AtomicPropositionVectorsBuilder<I> {
    type AtomicPropositions = AtomicPropositions<I>;
    type Index = I;

    fn stores_atomic_propositions() -> bool {
        true
    }

    fn register_atomic_proposition(&mut self, id: String) -> AnnotationIndex<I> {
        self.atomic_propositions
            .add_entry(id, TypedAnnotation::default())
    }

    fn set_value(&mut self, id: AnnotationIndex<I>, state: StateIndex<Self::Index>, value: bool) {
        self.atomic_propositions[id].add_value(state, value);
    }
}
