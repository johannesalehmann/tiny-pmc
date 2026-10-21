use crate::Ix;
use crate::base_model::{AnyBaseModel, BaseModelType, Mdp, StochasticGame};

// The narrowest type is the most specific type that fits the model. Transition systems take
// priority over DTMCs, so a model whose choices all have one branch is a transition system even if
// every state also has one choice. Games stay games, as narrowing never removes owners.

impl<I: Ix> Mdp<I> {
    pub fn narrowest_type(&self) -> BaseModelType {
        if self.graph.every_choice_has_one_branch() {
            BaseModelType::TransitionSystem
        } else if self.graph.every_state_has_one_choice() {
            BaseModelType::Dtmc
        } else {
            BaseModelType::Mdp
        }
    }

    pub fn narrow(self) -> AnyBaseModel<I> {
        let target = self.narrowest_type();
        self.try_narrow_to(target)
            .unwrap_or_else(|_| unreachable!("the narrowest type always fits the model"))
    }
}

impl<I: Ix> StochasticGame<I> {
    pub fn narrowest_type(&self) -> BaseModelType {
        if self.graph.every_choice_has_one_branch() {
            BaseModelType::NonstochasticGame
        } else {
            BaseModelType::StochasticGame
        }
    }

    pub fn narrow(self) -> AnyBaseModel<I> {
        let target = self.narrowest_type();
        self.try_narrow_to(target)
            .unwrap_or_else(|_| unreachable!("the narrowest type always fits the game"))
    }
}
