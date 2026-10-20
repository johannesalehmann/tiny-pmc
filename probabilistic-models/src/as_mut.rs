// If e.g. InitialStatesEnum::as_mut is renamed, the AsMutComponent implementation for it would
// not break and instead silently call itself. This ensures this does not happen.
#![deny(unconditional_recursion)]

use crate::Model;
use crate::annotations::{AtomicPropositions, RewardAnnotations};
use crate::base_model::{
    BaseModel, BranchGraph, ChoiceBranchGraph, ChoiceGraph, SinglePlayer, TurnBased,
};
use crate::initial_states::{InitialStates, InitialStatesEnum, SingleInitialState};
use crate::labels::Labels;
use crate::predecessors::Predecessors;
use crate::traits::ReadInitialStates;
use crate::valuations::Valuations;
use typed_index_collections::Index;

impl<
    M: AsMutComponent,
    Ini: AsMutComponent,
    ChLabel: AsMutComponent,
    BrLabel: AsMutComponent,
    Obs: AsMutComponent,
    APs: AsMutComponent,
    Rew: AsMutComponent,
    Ann: AsMutComponent,
    StateVals: AsMutComponent,
    Preds: AsMutComponent,
> Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    pub fn as_mut(
        &mut self,
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
            base: self.base.as_mut(),
            initial: self.initial.as_mut(),
            choice_labels: self.choice_labels.as_mut(),
            branch_labels: self.branch_labels.as_mut(),
            observations: self.observations.as_mut(),
            atomic_propositions: self.atomic_propositions.as_mut(),
            rewards: self.rewards.as_mut(),
            annotations: self.annotations.as_mut(),
            state_valuations: self.state_valuations.as_mut(),
            predecessors: self.predecessors.as_mut(),
        }
    }
}

pub trait AsMutComponent {
    type Output<'a>
    where
        Self: 'a;
    fn as_mut(&mut self) -> Self::Output<'_>;
}

impl AsMutComponent for () {
    type Output<'a> = ();

    fn as_mut(&mut self) -> Self::Output<'_> {}
}

impl<T: AsMutComponent> AsMutComponent for Option<T> {
    type Output<'a>
        = Option<T::Output<'a>>
    where
        T: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        Option::as_mut(self).map(T::as_mut)
    }
}

impl<AI: Index, SI: Index, AEI: Index> AsMutComponent for AtomicPropositions<AI, SI, AEI> {
    type Output<'a>
        = &'a mut Self
    where
        Self: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        self
    }
}

impl<G: AsMutComponent, P: AsMutComponent> BaseModel<G, P> {
    pub fn as_mut(&mut self) -> BaseModel<G::Output<'_>, P::Output<'_>> {
        BaseModel {
            graph: self.graph.as_mut(),
            players: self.players.as_mut(),
        }
    }
}

impl<G: AsMutComponent, P: AsMutComponent> AsMutComponent for BaseModel<G, P> {
    type Output<'a>
        = BaseModel<G::Output<'a>, P::Output<'a>>
    where
        Self: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        BaseModel::as_mut(self)
    }
}

impl<SI: Index, CI: Index, BI: Index> AsMutComponent for ChoiceBranchGraph<SI, CI, BI> {
    type Output<'a>
        = &'a mut Self
    where
        Self: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        self
    }
}

impl<SI: Index, CI: Index> AsMutComponent for ChoiceGraph<SI, CI> {
    type Output<'a>
        = &'a mut Self
    where
        Self: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        self
    }
}

impl<SI: Index, BI: Index> AsMutComponent for BranchGraph<SI, BI> {
    type Output<'a>
        = &'a mut Self
    where
        Self: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        self
    }
}

// As `SinglePlayer` is a unit struct, we return it by value. This also ensures
// impls for `BaseModel<G, SinglePlayer>` still apply
impl AsMutComponent for SinglePlayer {
    type Output<'a> = SinglePlayer;

    fn as_mut(&mut self) -> Self::Output<'_> {
        SinglePlayer
    }
}

impl<SI: Index, Owner> AsMutComponent for TurnBased<SI, Owner> {
    type Output<'a>
        = &'a mut Self
    where
        Self: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        self
    }
}

impl<SI: Index> AsMutComponent for SingleInitialState<SI> {
    type Output<'a>
        = &'a mut Self
    where
        Self: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        self
    }
}

impl<SI: Index> AsMutComponent for InitialStates<SI> {
    type Output<'a>
        = &'a mut Self
    where
        Self: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        self
    }
}

impl<
    SI: Index,
    Single: ReadInitialStates<StateIdx = SI>,
    Multiple: ReadInitialStates<StateIdx = SI>,
> AsMutComponent for InitialStatesEnum<SI, Single, Multiple>
{
    type Output<'a>
        = InitialStatesEnum<SI, &'a mut Single, &'a mut Multiple>
    where
        Self: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        InitialStatesEnum::as_mut(self)
    }
}

impl<EntityIdx: Index, ActionIdx: Index, E> AsMutComponent for Labels<EntityIdx, ActionIdx, E> {
    type Output<'a>
        = &'a mut Self
    where
        Self: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        self
    }
}

impl<RI: Index, SI: Index, CI: Index, AEI: Index> AsMutComponent
    for RewardAnnotations<RI, SI, CI, AEI>
{
    type Output<'a>
        = &'a mut Self
    where
        Self: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        self
    }
}

impl<EntityIdx: Index, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index> AsMutComponent
    for Valuations<EntityIdx, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    type Output<'a>
        = &'a mut Self
    where
        Self: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        self
    }
}

impl<SI: Index, CI: Index, BI: Index, PI: Index> AsMutComponent for Predecessors<SI, CI, BI, PI> {
    type Output<'a>
        = &'a mut Self
    where
        Self: 'a;

    fn as_mut(&mut self) -> Self::Output<'_> {
        self
    }
}
