use crate::Model;
use crate::annotations::RewardAnnotations;
use typed_index_collections::{Index, RawIndex};

pub trait ReadRewards {
    type StateIdx: Index;
    type ChoiceIdx: Index;
    type RewardIdx: Index;

    /// Returns the reward structure with the given name. If `name` is `None`, returns the first
    /// reward structure.
    fn reward_structure(&self, name: Option<&str>) -> Option<Self::RewardIdx>;
    fn has_state_rewards(&self, rewards: Self::RewardIdx) -> bool;
    fn has_choice_rewards(&self, rewards: Self::RewardIdx) -> bool;
    fn state_reward(&self, rewards: Self::RewardIdx, state: Self::StateIdx) -> f64;
    fn choice_reward(&self, rewards: Self::RewardIdx, choice: Self::ChoiceIdx) -> f64;
}

macro_rules! derive_read_rewards {
    ($subcomponent:ident) => {
        fn reward_structure(&self, name: Option<&str>) -> Option<Self::RewardIdx> {
            self.$subcomponent.reward_structure(name)
        }

        fn has_state_rewards(&self, rewards: Self::RewardIdx) -> bool {
            self.$subcomponent.has_state_rewards(rewards)
        }

        fn has_choice_rewards(&self, rewards: Self::RewardIdx) -> bool {
            self.$subcomponent.has_choice_rewards(rewards)
        }

        fn state_reward(&self, rewards: Self::RewardIdx, state: Self::StateIdx) -> f64 {
            self.$subcomponent.state_reward(rewards, state)
        }

        fn choice_reward(&self, rewards: Self::RewardIdx, choice: Self::ChoiceIdx) -> f64 {
            self.$subcomponent.choice_reward(rewards, choice)
        }
    };
}
pub(crate) use derive_read_rewards;

impl<RI: Index, SI: Index, CI: Index, AEI: Index> ReadRewards
    for RewardAnnotations<RI, SI, CI, AEI>
{
    type StateIdx = SI;
    type ChoiceIdx = CI;
    type RewardIdx = RI;

    fn reward_structure(&self, name: Option<&str>) -> Option<Self::RewardIdx> {
        match name {
            Some(name) => self.index_by_name(name),
            None if self.len() > 0 => Some(RI::from_raw(RI::RawType::from_usize(0))),
            None => None,
        }
    }

    fn has_state_rewards(&self, rewards: Self::RewardIdx) -> bool {
        self.entries()[rewards].states.is_some()
    }

    fn has_choice_rewards(&self, rewards: Self::RewardIdx) -> bool {
        self.entries()[rewards].choices.is_some()
    }

    fn state_reward(&self, rewards: Self::RewardIdx, state: Self::StateIdx) -> f64 {
        match &self.entries()[rewards].states {
            Some(values) => values[state],
            None => 0.0,
        }
    }

    fn choice_reward(&self, rewards: Self::RewardIdx, choice: Self::ChoiceIdx) -> f64 {
        match &self.entries()[rewards].choices {
            Some(values) => values[choice],
            None => 0.0,
        }
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew: ReadRewards, Ann, StateVals, Preds> ReadRewards
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type StateIdx = Rew::StateIdx;
    type ChoiceIdx = Rew::ChoiceIdx;
    type RewardIdx = Rew::RewardIdx;

    derive_read_rewards!(rewards);
}
