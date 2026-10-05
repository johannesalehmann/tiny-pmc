use crate::Model;

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
