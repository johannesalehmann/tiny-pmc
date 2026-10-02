mod single;
pub use single::SingleInitialState;

mod multiple;
pub use multiple::InitialStates;

mod initial_states_enum;
pub use initial_states_enum::InitialStatesEnum;

use crate::Model;
use crate::base_model::BaseModel;
use crate::traits::{ReadInitialStates, ReadStateSpace, StateSet};

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
impl<M: BaseModel, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
    Model<M, (), ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
{
    pub fn with_initial_state(
        self,
        initial: M::StateIndex,
    ) -> Model<M, SingleInitialState<M::StateIndex>, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
    {
        self.map_initial_states(|_| SingleInitialState { index: initial })
    }
    pub fn with_initial_states(
        self,
        initial: InitialStates<M::StateIndex>,
    ) -> Model<M, InitialStates<M::StateIndex>, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
    {
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

impl<M: BaseModel, Ini, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
{
    pub fn with_initial_states_enum<Single, Multiple>(
        self,
    ) -> Model<
        M,
        InitialStatesEnum<M::StateIndex, Single, Multiple>,
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
        Single: ReadInitialStates<StateIdx = M::StateIndex>,
        Multiple: ReadInitialStates<StateIdx = M::StateIndex>,
        Ini: Into<InitialStatesEnum<M::StateIndex, Single, Multiple>>,
    {
        self.map_initial_states(|i| i.into())
    }
}

impl<
    M: BaseModel,
    Single: ReadInitialStates<StateIdx = M::StateIndex>,
    Multiple: ReadInitialStates<StateIdx = M::StateIndex>,
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
        InitialStatesEnum<M::StateIndex, Single, Multiple>,
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
