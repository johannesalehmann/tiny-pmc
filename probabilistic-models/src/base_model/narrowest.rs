use crate::base_model::{
    AnyBaseModel, BaseModelEnum, BaseModelType, Dtmc, Mdp, NonstochasticGame, StochasticGame,
    TransitionSystem,
};
use crate::{HasIx, Ix, Model};

// The narrowest type is the most specific type that fits the model. Transition systems take
// priority over DTMCs, so a model whose choices all have one branch is a transition system even if
// every state also has one choice. Games stay games, as narrowing never removes owners.

pub trait Narrowable: HasIx + Sized {
    fn narrowest_type(&self) -> BaseModelType;

    // Converts the model into exactly the target type. Returns the model unchanged if it does not
    // fit the target.
    fn try_convert_to(self, target: BaseModelType) -> Result<AnyBaseModel<Self::Ix>, Self>;

    fn narrow(self) -> AnyBaseModel<Self::Ix> {
        let target = self.narrowest_type();
        self.try_convert_to(target)
            .unwrap_or_else(|_| unreachable!("the narrowest type always fits the model"))
    }
}

impl<I: Ix> Narrowable for TransitionSystem<I> {
    fn narrowest_type(&self) -> BaseModelType {
        BaseModelType::TransitionSystem
    }

    fn try_convert_to(self, target: BaseModelType) -> Result<AnyBaseModel<I>, Self> {
        match target {
            BaseModelType::TransitionSystem => Ok(self.into()),
            BaseModelType::Dtmc => self.try_into_dtmc().map(Into::into),
            BaseModelType::Mdp => Ok(self.into_mdp().into()),
            BaseModelType::NonstochasticGame | BaseModelType::StochasticGame => Err(self),
        }
    }
}

impl<I: Ix> Narrowable for Dtmc<I> {
    fn narrowest_type(&self) -> BaseModelType {
        if self.graph.every_state_has_one_branch() {
            BaseModelType::TransitionSystem
        } else {
            BaseModelType::Dtmc
        }
    }

    fn try_convert_to(self, target: BaseModelType) -> Result<AnyBaseModel<I>, Self> {
        match target {
            BaseModelType::TransitionSystem => self.try_into_transition_system().map(Into::into),
            BaseModelType::Dtmc => Ok(self.into()),
            BaseModelType::Mdp => Ok(self.into_mdp().into()),
            BaseModelType::NonstochasticGame | BaseModelType::StochasticGame => Err(self),
        }
    }
}

impl<I: Ix> Narrowable for Mdp<I> {
    fn narrowest_type(&self) -> BaseModelType {
        if self.graph.every_choice_has_one_branch() {
            BaseModelType::TransitionSystem
        } else if self.graph.every_state_has_one_choice() {
            BaseModelType::Dtmc
        } else {
            BaseModelType::Mdp
        }
    }

    fn try_convert_to(self, target: BaseModelType) -> Result<AnyBaseModel<I>, Self> {
        match target {
            BaseModelType::TransitionSystem => self.try_into_transition_system().map(Into::into),
            BaseModelType::Dtmc => self.try_into_dtmc().map(Into::into),
            BaseModelType::Mdp => Ok(self.into()),
            BaseModelType::NonstochasticGame | BaseModelType::StochasticGame => Err(self),
        }
    }
}

impl<I: Ix> Narrowable for NonstochasticGame<I> {
    fn narrowest_type(&self) -> BaseModelType {
        BaseModelType::NonstochasticGame
    }

    fn try_convert_to(self, target: BaseModelType) -> Result<AnyBaseModel<I>, Self> {
        match target {
            BaseModelType::NonstochasticGame => Ok(self.into()),
            BaseModelType::StochasticGame => Ok(self.into_stochastic_game().into()),
            BaseModelType::TransitionSystem | BaseModelType::Dtmc | BaseModelType::Mdp => Err(self),
        }
    }
}

impl<I: Ix> Narrowable for StochasticGame<I> {
    fn narrowest_type(&self) -> BaseModelType {
        if self.graph.every_choice_has_one_branch() {
            BaseModelType::NonstochasticGame
        } else {
            BaseModelType::StochasticGame
        }
    }

    fn try_convert_to(self, target: BaseModelType) -> Result<AnyBaseModel<I>, Self> {
        match target {
            BaseModelType::NonstochasticGame => self.try_into_nonstochastic_game().map(Into::into),
            BaseModelType::StochasticGame => Ok(self.into()),
            BaseModelType::TransitionSystem | BaseModelType::Dtmc | BaseModelType::Mdp => Err(self),
        }
    }
}

impl<TS, D, M, NG, SG> Narrowable for BaseModelEnum<TS, D, M, NG, SG>
where
    TS: Narrowable,
    D: Narrowable<Ix = TS::Ix>,
    M: Narrowable<Ix = TS::Ix>,
    NG: Narrowable<Ix = TS::Ix>,
    SG: Narrowable<Ix = TS::Ix>,
{
    fn narrowest_type(&self) -> BaseModelType {
        match self {
            BaseModelEnum::TransitionSystem(m) => m.narrowest_type(),
            BaseModelEnum::Dtmc(m) => m.narrowest_type(),
            BaseModelEnum::Mdp(m) => m.narrowest_type(),
            BaseModelEnum::NonstochasticGame(m) => m.narrowest_type(),
            BaseModelEnum::StochasticGame(m) => m.narrowest_type(),
        }
    }

    fn try_convert_to(self, target: BaseModelType) -> Result<AnyBaseModel<TS::Ix>, Self> {
        match self {
            BaseModelEnum::TransitionSystem(m) => m
                .try_convert_to(target)
                .map_err(BaseModelEnum::TransitionSystem),
            BaseModelEnum::Dtmc(m) => m.try_convert_to(target).map_err(BaseModelEnum::Dtmc),
            BaseModelEnum::Mdp(m) => m.try_convert_to(target).map_err(BaseModelEnum::Mdp),
            BaseModelEnum::NonstochasticGame(m) => m
                .try_convert_to(target)
                .map_err(BaseModelEnum::NonstochasticGame),
            BaseModelEnum::StochasticGame(m) => m
                .try_convert_to(target)
                .map_err(BaseModelEnum::StochasticGame),
        }
    }
}

impl<M: Narrowable, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
{
    /// Narrows the base model, see [`Narrowable`]. All other components are kept unchanged.
    pub fn narrow_base(
        self,
    ) -> Model<AnyBaseModel<M::Ix>, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds> {
        self.map_base(Narrowable::narrow)
    }

    /// Converts the base model into exactly the target type, see [`Narrowable::try_convert_to`].
    /// All other components are kept unchanged. Returns the model unchanged if it does not fit.
    pub fn try_convert_base_to(
        self,
        target: BaseModelType,
    ) -> Result<
        Model<AnyBaseModel<M::Ix>, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>,
        Self,
    > {
        self.try_map_base(|base| base.try_convert_to(target))
    }
}
