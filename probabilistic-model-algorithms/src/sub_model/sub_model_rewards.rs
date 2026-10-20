use probabilistic_models::traits::ReadRewards;
use probabilistic_models::{ChoiceIndex, Ix, RewardIndex, StateIndex};

pub trait RewardsSource<I: Ix> {
    fn has_state_rewards(&self) -> bool;
    fn has_choice_rewards(&self) -> bool;

    fn state_reward(&self, state: StateIndex<I>) -> f64;
    fn choice_reward(&self, choice: ChoiceIndex<I>) -> f64;
}

impl<I: Ix> RewardsSource<I> for () {
    fn has_state_rewards(&self) -> bool {
        false
    }

    fn has_choice_rewards(&self) -> bool {
        false
    }

    fn state_reward(&self, _state: StateIndex<I>) -> f64 {
        panic!("The model does not have state rewards")
    }

    fn choice_reward(&self, _choice: ChoiceIndex<I>) -> f64 {
        panic!("The model does not have choice rewards")
    }
}

pub struct StateAndChoiceRewards<'a, M: ReadRewards> {
    model: &'a M,
    rewards_index: RewardIndex<M::Ix>,
}

impl<'a, M: ReadRewards> StateAndChoiceRewards<'a, M> {
    pub fn new(model: &'a M, rewards_index: RewardIndex<M::Ix>) -> Self {
        Self {
            model,
            rewards_index,
        }
    }
}

impl<'a, M: ReadRewards> RewardsSource<M::Ix> for StateAndChoiceRewards<'a, M> {
    fn has_state_rewards(&self) -> bool {
        self.model.has_state_rewards(self.rewards_index)
    }

    fn has_choice_rewards(&self) -> bool {
        self.model.has_choice_rewards(self.rewards_index)
    }

    fn state_reward(&self, state: StateIndex<M::Ix>) -> f64 {
        self.model.state_reward(self.rewards_index, state)
    }

    fn choice_reward(&self, choice: ChoiceIndex<M::Ix>) -> f64 {
        self.model.choice_reward(self.rewards_index, choice)
    }
}

pub struct UnitStateRewards;

impl<I: Ix> RewardsSource<I> for UnitStateRewards {
    fn has_state_rewards(&self) -> bool {
        true
    }

    fn has_choice_rewards(&self) -> bool {
        false
    }

    fn state_reward(&self, _state: StateIndex<I>) -> f64 {
        1.0
    }

    fn choice_reward(&self, _choice: ChoiceIndex<I>) -> f64 {
        panic!("The model does not have choice rewards")
    }
}
