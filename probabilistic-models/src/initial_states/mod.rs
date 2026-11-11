mod single;
pub use single::{NotSingleInitialStateError, SingleInitialState};

mod multiple;
pub use multiple::InitialStates;

mod initial_states_enum;
pub use initial_states_enum::InitialStatesEnum;

use crate::Model;
use crate::StateIndex;
use crate::traits::ReadInitialStates;
use crate::traits::ReadStateSpace;

impl<M, I, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
    Model<M, I, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
{
    pub fn map_initial_states<I2, F: FnOnce(I) -> I2>(
        self,
        map: F,
    ) -> Model<M, I2, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds> {
        Model {
            base: self.base,
            initial: map(self.initial),
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        }
    }

    pub fn map_initial_states_optional<I2, F: FnOnce(I) -> Option<I2>>(
        self,
        map: F,
    ) -> Option<Model<M, I2, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>> {
        Some(Model {
            base: self.base,
            initial: map(self.initial)?,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        })
    }
}
impl<M: ReadStateSpace, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
    Model<M, (), ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
{
    pub fn with_initial_state(
        self,
        initial: StateIndex<M::Ix>,
    ) -> Model<M, SingleInitialState<M::Ix>, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds> {
        self.map_initial_states(|_| SingleInitialState { index: initial })
    }
    pub fn with_initial_states(
        self,
        initial: InitialStates<M::Ix>,
    ) -> Model<M, InitialStates<M::Ix>, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds> {
        self.map_initial_states(|_| initial)
    }
}

impl<M, I: ReadInitialStates, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
    Model<M, I, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
{
    pub fn replace_initial_states<I2>(
        self,
        initial: I2,
    ) -> Model<M, I2, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds> {
        self.map_initial_states(|_| initial)
    }
    pub fn without_initial_states(
        self,
    ) -> Model<M, (), ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds> {
        self.map_initial_states(|_| ())
    }
}

impl<M: ReadStateSpace, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
    Model<M, SingleInitialState<M::Ix>, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
{
    /// Replaces the [`SingleInitialState`] by an equivalent [`InitialStates`] vector.
    pub fn convert_initial_state_to_multiple(
        self,
    ) -> Model<M, InitialStates<M::Ix>, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds> {
        let num_states = self.base.states().len();
        self.map_initial_states(|i| i.to_initial_states(num_states))
    }
}

impl<M, Ix: crate::ix::Ix, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
    Model<M, InitialStates<Ix>, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
{
    /// Replaces the [`InitialStates`] vector by an equivalent [`SingleInitialState`].
    ///
    /// If the `InitialStates` vector does not contain exactly a single initial state, then the
    /// unchanged model and the reason for failure is returned.
    pub fn try_convert_initial_states_to_single(
        self,
    ) -> Result<
        Model<M, SingleInitialState<Ix>, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>,
        (Self, NotSingleInitialStateError),
    > {
        match SingleInitialState::try_from_initial_states(&self.initial) {
            Ok(single) => Ok(self.map_initial_states(|_| single)),
            Err(e) => Err((self, e)),
        }
    }
}

impl<M: ReadStateSpace, Ini, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
{
    pub fn with_initial_states_enum<Single, Multiple>(
        self,
    ) -> Model<
        M,
        InitialStatesEnum<M::Ix, Single, Multiple>,
        ChLabel,
        BrLabel,
        Obs,
        APs,
        Rew,
        Anno,
        Val,
        Preds,
    >
    where
        Single: ReadInitialStates<Ix = M::Ix>,
        Multiple: ReadInitialStates<Ix = M::Ix>,
        Ini: Into<InitialStatesEnum<M::Ix, Single, Multiple>>,
    {
        self.map_initial_states(|i| i.into())
    }
}

impl<
    M: ReadStateSpace,
    Single: ReadInitialStates<Ix = M::Ix>,
    Multiple: ReadInitialStates<Ix = M::Ix>,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Anno,
    Val,
    Preds,
>
    Model<
        M,
        InitialStatesEnum<M::Ix, Single, Multiple>,
        ChLabel,
        BrLabel,
        Obs,
        APs,
        Rew,
        Anno,
        Val,
        Preds,
    >
{
    pub fn initial_states_is_single(&self) -> bool {
        self.initial.is_single()
    }

    pub fn initial_states_is_multiple(&self) -> bool {
        self.initial.is_multiple()
    }

    pub fn initial_states_into_single(
        self,
    ) -> Option<Model<M, Single, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>> {
        self.map_initial_states_optional(|i| i.into_single())
    }

    pub fn initial_states_into_multiple(
        self,
    ) -> Option<Model<M, Multiple, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>> {
        self.map_initial_states_optional(|i| i.into_multiple())
    }
}
