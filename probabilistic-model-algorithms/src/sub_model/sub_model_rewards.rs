use probabilistic_models::traits::ReadRewards;
use typed_index_collections::Index;

pub trait RewardsSource<StateIdx: Index, ChoiceIdx: Index> {
    fn has_state_rewards(&self) -> bool;
    fn has_choice_rewards(&self) -> bool;

    fn state_reward(&self, state: StateIdx) -> f64;
    fn choice_reward(&self, choice: ChoiceIdx) -> f64;
}

impl<StateIdx: Index, ChoiceIdx: Index> RewardsSource<StateIdx, ChoiceIdx> for () {
    fn has_state_rewards(&self) -> bool {
        false
    }

    fn has_choice_rewards(&self) -> bool {
        false
    }

    fn state_reward(&self, _state: StateIdx) -> f64 {
        panic!("The model does not have state rewards")
    }

    fn choice_reward(&self, _choice: ChoiceIdx) -> f64 {
        panic!("The model does not have choice rewards")
    }
}

pub struct StateAndChoiceRewards<'a, M: ReadRewards> {
    model: &'a M,
    rewards_index: M::RewardIdx,
}

impl<'a, M: ReadRewards> StateAndChoiceRewards<'a, M> {
    pub fn new(model: &'a M, rewards_index: M::RewardIdx) -> Self {
        Self {
            model,
            rewards_index,
        }
    }
}

impl<'a, M: ReadRewards> RewardsSource<M::StateIdx, M::ChoiceIdx> for StateAndChoiceRewards<'a, M> {
    fn has_state_rewards(&self) -> bool {
        self.model.has_state_rewards(self.rewards_index)
    }

    fn has_choice_rewards(&self) -> bool {
        self.model.has_choice_rewards(self.rewards_index)
    }

    fn state_reward(&self, state: M::StateIdx) -> f64 {
        self.model.state_reward(self.rewards_index, state)
    }

    fn choice_reward(&self, choice: M::ChoiceIdx) -> f64 {
        self.model.choice_reward(self.rewards_index, choice)
    }
}
