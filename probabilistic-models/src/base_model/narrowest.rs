use crate::base_model::{
    AnyBaseModel, BaseModelEnum, BaseModelType, Dtmc, Mdp, NonstochasticGame, StochasticGame,
    TransitionSystem,
};
use crate::{HasIx, Ix, Model};

// The narrowest type is the most specific type that fits the model. Transition systems take
// priority over DTMCs, so a model whose choices all have one branch is a transition system even if
// every state also has one choice. Games stay games, as narrowing never removes owners.

pub trait Narrowable: HasIx {
    fn narrowest_type(&self) -> BaseModelType;

    fn narrow(self) -> AnyBaseModel<Self::Ix>;
}

impl<I: Ix> Narrowable for TransitionSystem<I> {
    fn narrowest_type(&self) -> BaseModelType {
        BaseModelType::TransitionSystem
    }

    fn narrow(self) -> AnyBaseModel<I> {
        self.into()
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

    fn narrow(self) -> AnyBaseModel<I> {
        let target = self.narrowest_type();
        self.try_narrow_to(target)
            .unwrap_or_else(|_| unreachable!("the narrowest type always fits the model"))
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

    fn narrow(self) -> AnyBaseModel<I> {
        let target = self.narrowest_type();
        self.try_narrow_to(target)
            .unwrap_or_else(|_| unreachable!("the narrowest type always fits the model"))
    }
}

impl<I: Ix> Narrowable for NonstochasticGame<I> {
    fn narrowest_type(&self) -> BaseModelType {
        BaseModelType::NonstochasticGame
    }

    fn narrow(self) -> AnyBaseModel<I> {
        self.into()
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

    fn narrow(self) -> AnyBaseModel<I> {
        let target = self.narrowest_type();
        self.try_narrow_to(target)
            .unwrap_or_else(|_| unreachable!("the narrowest type always fits the game"))
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

    fn narrow(self) -> AnyBaseModel<TS::Ix> {
        match self {
            BaseModelEnum::TransitionSystem(m) => m.narrow(),
            BaseModelEnum::Dtmc(m) => m.narrow(),
            BaseModelEnum::Mdp(m) => m.narrow(),
            BaseModelEnum::NonstochasticGame(m) => m.narrow(),
            BaseModelEnum::StochasticGame(m) => m.narrow(),
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
}
