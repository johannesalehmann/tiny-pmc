use crate::Model;
use crate::annotations::RewardAnnotations;
use crate::{ChoiceIndex, HasIx, Ix, RewardIndex, StateIndex};
use typed_index_collections::{Index, RawIndex};

pub trait ReadRewards: HasIx {
    /// Returns the reward structure with the given name. If `name` is `None`, returns the first
    /// reward structure.
    fn reward_structure(&self, name: Option<&str>) -> Option<RewardIndex<Self::Ix>>;
    fn has_state_rewards(&self, rewards: RewardIndex<Self::Ix>) -> bool;
    fn has_choice_rewards(&self, rewards: RewardIndex<Self::Ix>) -> bool;
    fn state_reward(&self, rewards: RewardIndex<Self::Ix>, state: StateIndex<Self::Ix>) -> f64;
    fn choice_reward(&self, rewards: RewardIndex<Self::Ix>, choice: ChoiceIndex<Self::Ix>) -> f64;
}

macro_rules! derive_read_rewards {
    ($subcomponent:ident) => {
        fn reward_structure(&self, name: Option<&str>) -> Option<$crate::RewardIndex<Self::Ix>> {
            self.$subcomponent.reward_structure(name)
        }

        fn has_state_rewards(&self, rewards: $crate::RewardIndex<Self::Ix>) -> bool {
            self.$subcomponent.has_state_rewards(rewards)
        }

        fn has_choice_rewards(&self, rewards: $crate::RewardIndex<Self::Ix>) -> bool {
            self.$subcomponent.has_choice_rewards(rewards)
        }

        fn state_reward(
            &self,
            rewards: $crate::RewardIndex<Self::Ix>,
            state: $crate::StateIndex<Self::Ix>,
        ) -> f64 {
            self.$subcomponent.state_reward(rewards, state)
        }

        fn choice_reward(
            &self,
            rewards: $crate::RewardIndex<Self::Ix>,
            choice: $crate::ChoiceIndex<Self::Ix>,
        ) -> f64 {
            self.$subcomponent.choice_reward(rewards, choice)
        }
    };
}
pub(crate) use derive_read_rewards;

impl<I: Ix> ReadRewards for RewardAnnotations<I> {
    fn reward_structure(&self, name: Option<&str>) -> Option<RewardIndex<Self::Ix>> {
        match name {
            Some(name) => self.index_by_name(name),
            None if self.len() > 0 => Some(RewardIndex::from_raw(I::Meta::zero())),
            None => None,
        }
    }

    fn has_state_rewards(&self, rewards: RewardIndex<Self::Ix>) -> bool {
        self.entries()[rewards].states.is_some()
    }

    fn has_choice_rewards(&self, rewards: RewardIndex<Self::Ix>) -> bool {
        self.entries()[rewards].choices.is_some()
    }

    fn state_reward(&self, rewards: RewardIndex<Self::Ix>, state: StateIndex<Self::Ix>) -> f64 {
        match &self.entries()[rewards].states {
            Some(values) => values[state],
            None => 0.0,
        }
    }

    fn choice_reward(&self, rewards: RewardIndex<Self::Ix>, choice: ChoiceIndex<Self::Ix>) -> f64 {
        match &self.entries()[rewards].choices {
            Some(values) => values[choice],
            None => 0.0,
        }
    }
}

impl<M: HasIx, Ini, ChLabel, BrLabel, Obs, APs, Rew: ReadRewards<Ix = M::Ix>, Ann, StateVals, Preds>
    ReadRewards for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    derive_read_rewards!(rewards);
}

impl<T: ReadRewards> ReadRewards for &T {
    fn reward_structure(&self, name: Option<&str>) -> Option<RewardIndex<Self::Ix>> {
        (**self).reward_structure(name)
    }
    fn has_state_rewards(&self, rewards: RewardIndex<Self::Ix>) -> bool {
        (**self).has_state_rewards(rewards)
    }
    fn has_choice_rewards(&self, rewards: RewardIndex<Self::Ix>) -> bool {
        (**self).has_choice_rewards(rewards)
    }
    fn state_reward(&self, rewards: RewardIndex<Self::Ix>, state: StateIndex<Self::Ix>) -> f64 {
        (**self).state_reward(rewards, state)
    }
    fn choice_reward(&self, rewards: RewardIndex<Self::Ix>, choice: ChoiceIndex<Self::Ix>) -> f64 {
        (**self).choice_reward(rewards, choice)
    }
}

impl<T: ReadRewards> ReadRewards for &mut T {
    fn reward_structure(&self, name: Option<&str>) -> Option<RewardIndex<Self::Ix>> {
        (**self).reward_structure(name)
    }
    fn has_state_rewards(&self, rewards: RewardIndex<Self::Ix>) -> bool {
        (**self).has_state_rewards(rewards)
    }
    fn has_choice_rewards(&self, rewards: RewardIndex<Self::Ix>) -> bool {
        (**self).has_choice_rewards(rewards)
    }
    fn state_reward(&self, rewards: RewardIndex<Self::Ix>, state: StateIndex<Self::Ix>) -> f64 {
        (**self).state_reward(rewards, state)
    }
    fn choice_reward(&self, rewards: RewardIndex<Self::Ix>, choice: ChoiceIndex<Self::Ix>) -> f64 {
        (**self).choice_reward(rewards, choice)
    }
}

pub trait ReadRewardsMaybe: HasIx {
    type WithRewards: ReadRewards<Ix = Self::Ix>;

    fn has_rewards(&self) -> bool;
    fn try_with_rewards(self) -> Option<Self::WithRewards>;
}

impl<T: ReadRewards> ReadRewardsMaybe for T {
    type WithRewards = T;

    fn has_rewards(&self) -> bool {
        true
    }

    fn try_with_rewards(self) -> Option<Self::WithRewards> {
        Some(self)
    }
}

impl<T: ReadRewards> ReadRewardsMaybe for Option<T> {
    type WithRewards = T;

    fn has_rewards(&self) -> bool {
        self.is_some()
    }

    fn try_with_rewards(self) -> Option<Self::WithRewards> {
        self
    }
}

impl<M: HasIx, Ini, ChLabel, BrLabel, Obs, APs, Rew: ReadRewards<Ix = M::Ix>, Ann, StateVals, Preds>
    ReadRewardsMaybe
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Option<Rew>, Ann, StateVals, Preds>
{
    type WithRewards = Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>;

    fn has_rewards(&self) -> bool {
        self.rewards.is_some()
    }

    fn try_with_rewards(self) -> Option<Self::WithRewards> {
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
