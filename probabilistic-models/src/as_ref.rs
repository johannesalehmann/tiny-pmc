// If e.g. InitialStatesEnum::as_ref is renamed, the AsRefComponent implementation for it would
// not break and instead silently call itself. This ensures this does not happen.
#![deny(unconditional_recursion)]

use crate::Model;
use crate::annotations::{AtomicPropositions, RewardAnnotations};
use crate::base_model::{Mdp, NonstochasticGame, TransitionSystem, TwoPlayerTurnBasedGame};
use crate::initial_states::{InitialStates, InitialStatesEnum, SingleInitialState};
use crate::labels::Labels;
use crate::predecessors::Predecessors;
use crate::traits::ReadInitialStates;
use crate::valuations::Valuations;
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
    type Output<'a>
    where
        Self: 'a;
    fn as_ref(&self) -> Self::Output<'_>;
}

impl AsRefComponent for () {
    type Output<'a> = ();

    fn as_ref(&self) -> Self::Output<'_> {}
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

impl<AI: Index, SI: Index, AEI: Index> AsRefComponent for AtomicPropositions<AI, SI, AEI> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        &self
    }
}

impl<SI: Index, CI: Index, BI: Index> AsRefComponent for Mdp<SI, CI, BI> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<SI: Index, CI: Index> AsRefComponent for TransitionSystem<SI, CI> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<SI: Index, CI: Index> AsRefComponent for NonstochasticGame<SI, CI> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<SI: Index, CI: Index, BI: Index> AsRefComponent for TwoPlayerTurnBasedGame<SI, CI, BI> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<SI: Index> AsRefComponent for SingleInitialState<SI> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<SI: Index> AsRefComponent for InitialStates<SI> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<
    SI: Index,
    Single: ReadInitialStates<StateIdx = SI>,
    Multiple: ReadInitialStates<StateIdx = SI>,
> AsRefComponent for InitialStatesEnum<SI, Single, Multiple>
{
    type Output<'a>
        = InitialStatesEnum<SI, &'a Single, &'a Multiple>
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

impl<RI: Index, SI: Index, CI: Index, AEI: Index> AsRefComponent
    for RewardAnnotations<RI, SI, CI, AEI>
{
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<EntityIdx: Index, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index> AsRefComponent
    for Valuations<EntityIdx, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}

impl<SI: Index, CI: Index, BI: Index, PI: Index> AsRefComponent for Predecessors<SI, CI, BI, PI> {
    type Output<'a>
        = &'a Self
    where
        Self: 'a;

    fn as_ref(&self) -> Self::Output<'_> {
        self
    }
}
