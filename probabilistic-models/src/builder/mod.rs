mod bases;
use bases::{BaseModelBuilder, MdpBuilder};

mod initial_states;
use initial_states::{
    InitialStatesBuilder, MultipleInitialStatesBuilder, SingleInitialStatesBuilder,
    UntrackedInitialStatesBuilder,
};

mod atomic_propositions;
use atomic_propositions::{
    AtomicPropositionBuilder, AtomicPropositionVectorsBuilder, UntrackedAtomicPropositionBuilder,
};

use crate::valuations::Valuations;
use crate::{Model, StateIndex};

pub struct ModelBuilderBuilder<
    Base: BaseModelBuilder,
    Ini: InitialStatesBuilder,
    APs: AtomicPropositionBuilder,
> {
    base: Base,
    initial_states: Ini,
    atomic_propositions: APs,
}

impl<Base: BaseModelBuilder, Ini: InitialStatesBuilder, APs: AtomicPropositionBuilder>
    ModelBuilderBuilder<Base, Ini, APs>
{
    pub fn new(
        base: Base,
    ) -> ModelBuilderBuilder<
        Base,
        UntrackedInitialStatesBuilder<Base::Index>,
        UntrackedAtomicPropositionBuilder<Base::Index>,
    > {
        ModelBuilderBuilder {
            base,
            initial_states: UntrackedInitialStatesBuilder::default(),
            atomic_propositions: UntrackedAtomicPropositionBuilder::default(),
        }
    }

    // TODO: Functions to add and remove initial states

    pub fn finish(self) -> ModelBuilder<Base, Ini, APs> {
        ModelBuilder {
            base: self.base,
            initial_states: self.initial_states,
            atomic_propositions: self.atomic_propositions,
        }
    }
}

pub struct ModelBuilder<
    Base: BaseModelBuilder,
    Ini: InitialStatesBuilder,
    APs: AtomicPropositionBuilder,
> {
    base: Base,
    initial_states: Ini,
    atomic_propositions: APs,
}

impl<Base: BaseModelBuilder, Ini: InitialStatesBuilder, APs: AtomicPropositionBuilder>
    ModelBuilder<Base, Ini, APs>
{
    pub fn finish() -> Model<
        Base::Index,
        Base::BaseModel,
        Ini::InitialStates,
        (),
        (),
        (),
        APs::AtomicPropositions,
        (),
        (),
        Valuations<Base::Index, StateIndex<Base::Index>>,
    > {
        todo!()
    }
}
