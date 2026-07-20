use crate::ModelBuilder;
use prism_model::Span;
use probabilistic_models::annotations::{AtomicPropositions, TypedAnnotation};
use std::marker::PhantomData;
use typed_index_collections::Index;

// TODO: It might not make sense to keep this configurable. Instead, the correct kind of
//  AtomicPropositionBuilder can be derived from the Labels field directly.

pub trait AtomicPropositionBuilder {
    type AtomicPropositions;
    type APIdx: Index;
    type StateIdx: Index;

    fn stores_atomic_propositions() -> bool;
    fn register_atomic_proposition(&mut self, id: String) -> Self::APIdx;
    fn set_value(&mut self, index: Self::APIdx, state: Self::StateIdx, value: bool);
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
    type APIdx = AnnotationIdx;
    type StateIdx = StateIdx;

    fn stores_atomic_propositions() -> bool {
        false
    }

    fn register_atomic_proposition(&mut self, _id: String) -> Self::APIdx {
        panic!("Cannot register atomic propositions when using `UntrackedAtomicPropositionBuilder`")
    }

    fn set_value(&mut self, _id: Self::APIdx, _state: Self::StateIdx, _value: bool) {
        panic!("Cannot store atomic propositions when using `UntrackedAtomicPropositionBuilder`")
    }

    fn into_atomic_propositions(self) -> Self::AtomicPropositions {
        ()
    }
}
impl<
    'a,
    API: Index,
    S: Span,
    Q: crate::queries::QueryCollection,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    B: crate::bases::BaseModelBuilder,
    IB: crate::initial_states_builder::InitialStatesBuilder<StateIdx = B::StateIdx>,
> ModelBuilder<'a, S, Q, L, IS, B, IB, UntrackedAtomicPropositionBuilder<API, B::StateIdx>>
{
    pub fn with_atomic_proposition_vector<APEI: Index>(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, AtomicPropositionVectorsBuilder<API, B::StateIdx, APEI>>
    {
        self.map_atomic_propositions(AtomicPropositionVectorsBuilder::default())
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
    type APIdx = AnnotationIdx;
    type StateIdx = StateIdx;

    fn stores_atomic_propositions() -> bool {
        true
    }

    fn register_atomic_proposition(&mut self, id: String) -> Self::APIdx {
        self.atomic_propositions
            .add_entry(id, TypedAnnotation::default())
    }

    fn set_value(&mut self, id: Self::APIdx, state: Self::StateIdx, value: bool) {
        self.atomic_propositions[id].add_value(state, value);
    }

    fn into_atomic_propositions(self) -> Self::AtomicPropositions {
        self.atomic_propositions
    }
}
impl<
    'a,
    API: Index,
    APEI: Index,
    S: Span,
    Q: crate::queries::QueryCollection,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    B: crate::bases::BaseModelBuilder,
    IB: crate::initial_states_builder::InitialStatesBuilder<StateIdx = B::StateIdx>,
> ModelBuilder<'a, S, Q, L, IS, B, IB, AtomicPropositionVectorsBuilder<API, B::StateIdx, APEI>>
{
    pub fn without_atomic_proposition_vector(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, UntrackedAtomicPropositionBuilder<API, B::StateIdx>>
    {
        self.map_atomic_propositions(UntrackedAtomicPropositionBuilder::default())
    }
}
