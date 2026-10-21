use crate::ModelBuilder;
use prism_model::Span;
use probabilistic_models::annotations::{AtomicPropositions, TypedAnnotation};
use probabilistic_models::{AtomicPropositionIndex, HasIx, Ix, StateIndex};
use std::marker::PhantomData;

// TODO: It might not make sense to keep this configurable. Instead, the correct kind of
//  AtomicPropositionBuilder can be derived from the Labels field directly.

pub trait AtomicPropositionBuilder: HasIx {
    type AtomicPropositions;

    fn stores_atomic_propositions() -> bool;
    fn register_atomic_proposition(&mut self, id: String) -> AtomicPropositionIndex<Self::Ix>;
    fn set_value(
        &mut self,
        index: AtomicPropositionIndex<Self::Ix>,
        state: StateIndex<Self::Ix>,
        value: bool,
    );
    fn into_atomic_propositions(self) -> Self::AtomicPropositions;
}

#[derive(Default)]
pub struct UntrackedAtomicPropositionBuilder<I: Ix> {
    _phantom_data: PhantomData<I>,
}

impl<I: Ix> HasIx for UntrackedAtomicPropositionBuilder<I> {
    type Ix = I;
}

impl<I: Ix> AtomicPropositionBuilder for UntrackedAtomicPropositionBuilder<I> {
    type AtomicPropositions = ();

    fn stores_atomic_propositions() -> bool {
        false
    }

    fn register_atomic_proposition(&mut self, _id: String) -> AtomicPropositionIndex<Self::Ix> {
        panic!("Cannot register atomic propositions when using `UntrackedAtomicPropositionBuilder`")
    }

    fn set_value(
        &mut self,
        _id: AtomicPropositionIndex<Self::Ix>,
        _state: StateIndex<Self::Ix>,
        _value: bool,
    ) {
        panic!("Cannot store atomic propositions when using `UntrackedAtomicPropositionBuilder`")
    }

    fn into_atomic_propositions(self) -> Self::AtomicPropositions {
        ()
    }
}
impl<
    'a,
    S: Span,
    Q: crate::queries::QueryCollection,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    Sel: crate::model_type_selection::ModelTypeSelection,
    IB: crate::initial_states_builder::InitialStatesBuilder<Ix = Sel::Ix>,
    CL: crate::choice_labels::ChoiceLabelBuilder<Ix = Sel::Ix>,
    Rew: crate::rewards_builder::RewardsBuilder<Ix = Sel::Ix>,
> ModelBuilder<'a, S, Q, L, IS, Sel, IB, UntrackedAtomicPropositionBuilder<Sel::Ix>, CL, Rew>
{
    pub fn with_atomic_proposition_vector(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, Sel, IB, AtomicPropositionVectorsBuilder<Sel::Ix>, CL, Rew>
    {
        self.map_atomic_propositions(AtomicPropositionVectorsBuilder::default())
    }
}

#[derive(Default)]
pub struct AtomicPropositionVectorsBuilder<I: Ix> {
    atomic_propositions: AtomicPropositions<I>,
}

impl<I: Ix> HasIx for AtomicPropositionVectorsBuilder<I> {
    type Ix = I;
}

impl<I: Ix> AtomicPropositionBuilder for AtomicPropositionVectorsBuilder<I> {
    type AtomicPropositions = AtomicPropositions<I>;

    fn stores_atomic_propositions() -> bool {
        true
    }

    fn register_atomic_proposition(&mut self, id: String) -> AtomicPropositionIndex<Self::Ix> {
        self.atomic_propositions
            .add_entry(id, TypedAnnotation::default())
    }

    fn set_value(
        &mut self,
        id: AtomicPropositionIndex<Self::Ix>,
        state: StateIndex<Self::Ix>,
        value: bool,
    ) {
        self.atomic_propositions[id].add_value(state, value);
    }

    fn into_atomic_propositions(self) -> Self::AtomicPropositions {
        self.atomic_propositions
    }
}
impl<
    'a,
    S: Span,
    Q: crate::queries::QueryCollection,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    Sel: crate::model_type_selection::ModelTypeSelection,
    IB: crate::initial_states_builder::InitialStatesBuilder<Ix = Sel::Ix>,
    CL: crate::choice_labels::ChoiceLabelBuilder<Ix = Sel::Ix>,
    Rew: crate::rewards_builder::RewardsBuilder<Ix = Sel::Ix>,
> ModelBuilder<'a, S, Q, L, IS, Sel, IB, AtomicPropositionVectorsBuilder<Sel::Ix>, CL, Rew>
{
    pub fn without_atomic_proposition_vector(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, Sel, IB, UntrackedAtomicPropositionBuilder<Sel::Ix>, CL, Rew>
    {
        self.map_atomic_propositions(UntrackedAtomicPropositionBuilder::default())
    }
}
