use crate::Model;

// TODO: Once the other components also have map_[component]_optional, use that instead of manually
//  rebuilding the model.

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    Model<M, Option<Ini>, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    pub fn initial_states_is_some(&self) -> bool {
        self.initial.is_some()
    }

    pub fn initial_states_unwrapped(
        self,
    ) -> Option<Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>> {
        self.map_initial_states_optional(|i| i)
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    Model<M, Ini, Option<ChLabel>, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    pub fn choice_labels_is_some(&self) -> bool {
        self.choice_labels.is_some()
    }

    pub fn choice_labels_unwrapped(
        self,
    ) -> Option<Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels?,
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

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    Model<M, Ini, ChLabel, Option<BrLabel>, Obs, APs, Rew, Ann, StateVals, Preds>
{
    pub fn branch_labels_is_some(&self) -> bool {
        self.branch_labels.is_some()
    }

    pub fn branch_labels_unwrapped(
        self,
    ) -> Option<Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels?,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        })
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    Model<M, Ini, ChLabel, BrLabel, Option<Obs>, APs, Rew, Ann, StateVals, Preds>
{
    pub fn observations_is_some(&self) -> bool {
        self.observations.is_some()
    }

    pub fn observations_unwrapped(
        self,
    ) -> Option<Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations?,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        })
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    Model<M, Ini, ChLabel, BrLabel, Obs, Option<APs>, Rew, Ann, StateVals, Preds>
{
    pub fn atomic_propositions_is_some(&self) -> bool {
        self.atomic_propositions.is_some()
    }

    pub fn atomic_propositions_unwrapped(
        self,
    ) -> Option<Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions?,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        })
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Option<Rew>, Ann, StateVals, Preds>
{
    pub fn rewards_is_some(&self) -> bool {
        self.rewards.is_some()
    }

    pub fn rewards_unwrapped(
        self,
    ) -> Option<Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards?,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        })
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Option<Ann>, StateVals, Preds>
{
    pub fn annotations_is_some(&self) -> bool {
        self.annotations.is_some()
    }

    pub fn annotations_unwrapped(
        self,
    ) -> Option<Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations?,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        })
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Option<StateVals>, Preds>
{
    pub fn state_valuations_is_some(&self) -> bool {
        self.state_valuations.is_some()
    }

    pub fn state_valuations_unwrapped(
        self,
    ) -> Option<Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations?,
            predecessors: self.predecessors,
        })
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Option<Preds>>
{
    pub fn predecessors_is_some(&self) -> bool {
        self.predecessors.is_some()
    }

    pub fn predecessors_unwrapped(
        self,
    ) -> Option<Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors?,
        })
    }
}

pub trait IntoOptionalComponent<Output> {
    fn into_optional(self) -> Option<Output>;
}

impl<Output> IntoOptionalComponent<Output> for () {
    fn into_optional(self) -> Option<Output> {
        None
    }
}

impl<Output> IntoOptionalComponent<Output> for Option<Output> {
    fn into_optional(self) -> Option<Output> {
        self
    }
}

impl<T: crate::Component> IntoOptionalComponent<T> for T {
    fn into_optional(self) -> Option<T> {
        Some(self)
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    pub fn into_optional<Ini2, ChLabel2, BrLabel2, Obs2, APs2, Rew2, Ann2, StateVals2, Preds2>(
        self,
    ) -> Model<
        M,
        Option<Ini2>,
        Option<ChLabel2>,
        Option<BrLabel2>,
        Option<Obs2>,
        Option<APs2>,
        Option<Rew2>,
        Option<Ann2>,
        Option<StateVals2>,
        Option<Preds2>,
    >
    where
        Ini: IntoOptionalComponent<Ini2>,
        ChLabel: IntoOptionalComponent<ChLabel2>,
        BrLabel: IntoOptionalComponent<BrLabel2>,
        Obs: IntoOptionalComponent<Obs2>,
        APs: IntoOptionalComponent<APs2>,
        Rew: IntoOptionalComponent<Rew2>,
        Ann: IntoOptionalComponent<Ann2>,
        StateVals: IntoOptionalComponent<StateVals2>,
        Preds: IntoOptionalComponent<Preds2>,
    {
        Model {
            base: self.base,
            initial: self.initial.into_optional(),
            choice_labels: self.choice_labels.into_optional(),
            branch_labels: self.branch_labels.into_optional(),
            observations: self.observations.into_optional(),
            atomic_propositions: self.atomic_propositions.into_optional(),
            rewards: self.rewards.into_optional(),
            annotations: self.annotations.into_optional(),
            state_valuations: self.state_valuations.into_optional(),
            predecessors: self.predecessors.into_optional(),
        }
    }
}
