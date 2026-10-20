// If e.g. InitialStatesEnum::as_ref is renamed, the AsRefComponent implementation for it would
// not break and instead silently call itself. This ensures this does not happen.
#![deny(unconditional_recursion)]

use crate::Ix;
use crate::Model;
use crate::annotations::{AtomicPropositions, RewardAnnotations};
use crate::base_model::{
    BaseModel, BaseModelEnum, BranchGraph, ChoiceBranchGraph, ChoiceGraph, SinglePlayer, TurnBased,
};
use crate::initial_states::{InitialStates, InitialStatesEnum, SingleInitialState};
use crate::labels::Labels;
use crate::predecessors::Predecessors;
use crate::traits::ReadInitialStates;
use crate::valuations::Valuations;
use std::convert::Infallible;
use typed_index_collections::Index;

impl<
    M: AsRefComponent,
    Ini: AsRefComponent,
    ChLabel: AsRefComponent,
    BrLabel: AsRefComponent,
    Obs: AsRefComponent,
    APs: AsRefComponent,
    Rew: AsRefComponent,
    Ann: AsRefComponent,
    StateVals: AsRefComponent,
    Preds: AsRefComponent,
> Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    pub fn as_ref(
        &self,
    ) -> Model<
        M::Output<'_>,
        Ini::Output<'_>,
        ChLabel::Output<'_>,
        BrLabel::Output<'_>,
        Obs::Output<'_>,
        APs::Output<'_>,
        Rew::Output<'_>,
        Ann::Output<'_>,
        StateVals::Output<'_>,
        Preds::Output<'_>,
    > {
        Model {
            base: self.base.as_ref(),
            initial: self.initial.as_ref(),
            choice_labels: self.choice_labels.as_ref(),
            branch_labels: self.branch_labels.as_ref(),
            observations: self.observations.as_ref(),
            atomic_propositions: self.atomic_propositions.as_ref(),
            rewards: self.rewards.as_ref(),
            annotations: self.annotations.as_ref(),
            state_valuations: self.state_valuations.as_ref(),
            predecessors: self.predecessors.as_ref(),
        }
    }
}

pub trait AsRefComponent {
    type Output<'a>: Copy
    where
        Self: 'a;
    fn as_ref(&self) -> Self::Output<'_>;
}

impl AsRefComponent for () {
    type Output<'a> = ();

    fn as_ref(&self) -> Self::Output<'_> {}
}

impl AsRefComponent for Infallible {
    type Output<'a> = Infallible;

    fn as_ref(&self) -> Self::Output<'_> {
        unreachable!();
    }
}

impl<T: AsRefComponent> AsRefComponent for Option<T> {
    type Output<'a>
        = Option<T::Output<'a>>
    where
        T: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        Option::as_ref(self).map(T::as_ref)
    }
}

impl<I: Ix> AsRefComponent for AtomicPropositions<I> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        &self
    }
}

impl<G: AsRefComponent, P: AsRefComponent> BaseModel<G, P> {
    pub fn as_ref(&self) -> BaseModel<G::Output<'_>, P::Output<'_>> {
        BaseModel {
            graph: self.graph.as_ref(),
            players: self.players.as_ref(),
        }
    }
}

impl<G: AsRefComponent, P: AsRefComponent> AsRefComponent for BaseModel<G, P> {
    type Output<'a>
        = BaseModel<G::Output<'a>, P::Output<'a>>
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        BaseModel::as_ref(self)
    }
}

impl<
    TS: AsRefComponent,
    D: AsRefComponent,
    M: AsRefComponent,
    NG: AsRefComponent,
    SG: AsRefComponent,
> BaseModelEnum<TS, D, M, NG, SG>
{
    pub fn as_ref(
        &self,
    ) -> BaseModelEnum<TS::Output<'_>, D::Output<'_>, M::Output<'_>, NG::Output<'_>, SG::Output<'_>>
    {
        match self {
            BaseModelEnum::TransitionSystem(m) => BaseModelEnum::TransitionSystem(m.as_ref()),
            BaseModelEnum::Dtmc(m) => BaseModelEnum::Dtmc(m.as_ref()),
            BaseModelEnum::Mdp(m) => BaseModelEnum::Mdp(m.as_ref()),
            BaseModelEnum::NonstochasticGame(m) => BaseModelEnum::NonstochasticGame(m.as_ref()),
            BaseModelEnum::StochasticGame(m) => BaseModelEnum::StochasticGame(m.as_ref()),
        }
    }
}

impl<
    TS: AsRefComponent,
    D: AsRefComponent,
    M: AsRefComponent,
    NG: AsRefComponent,
    SG: AsRefComponent,
> AsRefComponent for BaseModelEnum<TS, D, M, NG, SG>
{
    type Output<'a>
        =
        BaseModelEnum<TS::Output<'a>, D::Output<'a>, M::Output<'a>, NG::Output<'a>, SG::Output<'a>>
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        BaseModelEnum::as_ref(self)
    }
}

impl<I: Ix> AsRefComponent for ChoiceBranchGraph<I> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<I: Ix> AsRefComponent for ChoiceGraph<I> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<I: Ix> AsRefComponent for BranchGraph<I> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

// As `SinglePlayer` is a unit struct, we return it by value. This also ensures
// impls for `BaseModel<G, SinglePlayer>` still apply
impl AsRefComponent for SinglePlayer {
    type Output<'a> = SinglePlayer;

    fn as_ref(&self) -> Self::Output<'_> {
        SinglePlayer
    }
}

impl<I: Ix, Owner> AsRefComponent for TurnBased<I, Owner> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<I: Ix> AsRefComponent for SingleInitialState<I> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<I: Ix> AsRefComponent for InitialStates<I> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<I: Ix, Single: ReadInitialStates<Ix = I>, Multiple: ReadInitialStates<Ix = I>> AsRefComponent
    for InitialStatesEnum<I, Single, Multiple>
{
    type Output<'a>
        = InitialStatesEnum<I, &'a Single, &'a Multiple>
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        InitialStatesEnum::as_ref(self)
    }
}

impl<EntityIdx: Index, ActionIdx: Index, E> AsRefComponent for Labels<EntityIdx, ActionIdx, E> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<I: Ix> AsRefComponent for RewardAnnotations<I> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<I: Ix, EntityIdx: Index> AsRefComponent for Valuations<I, EntityIdx> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<I: Ix> AsRefComponent for Predecessors<I> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}
