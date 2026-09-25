use super::{PermuteStatesWithContext, StateOrdering};
use crate::annotations::{EntityRewards, RewardAnnotations, StateChoiceRewards};
use crate::traits::ReadStateSpace;
use typed_index_collections::{Index, To1};

impl<SI: Index, AI: Index, CI: Index, AEI: Index> PermuteStatesWithContext<SI, CI>
    for RewardAnnotations<AI, SI, CI, AEI>
{
    fn permute_states_with_context<Base: ReadStateSpace<StateIndex = SI, ChoiceIndex = CI>>(
        &self,
        ordering: &StateOrdering<SI>,
        old_base: &Base,
        new_base: &Base,
    ) -> Self {
        let mut rewards = RewardAnnotations::new();
        for (name, old_reward) in self {
            let new_state_rewards = match &old_reward.states {
                None => None,
                Some(old_state_rewards) => {
                    let mut new_state_rewards =
                        EntityRewards::with_identity_distribution_and_entries(To1::with_entries(
                            vec![0.0; old_state_rewards.values().len()],
                        ));
                    for (old, new) in ordering.old_to_new.enumerate() {
                        new_state_rewards[*new] = old_state_rewards[old]
                    }
                    Some(new_state_rewards)
                }
            };
            let new_choice_rewards = match &old_reward.choices {
                None => None,
                Some(old_choice_rewards) => {
                    let mut new_choice_rewards =
                        EntityRewards::with_identity_distribution_and_entries(To1::with_entries(
                            vec![0.0; old_choice_rewards.values().len()],
                        ));
                    for (old_state, &new_state) in ordering.old_to_new.enumerate() {
                        for (offset, old_choice) in
                            old_base.choices_of_state(old_state).into_iter().enumerate()
                        {
                            let new_choice = new_base.choices_of_state(new_state).index(offset);
                            new_choice_rewards[new_choice] = old_choice_rewards[old_choice];
                        }
                    }
                    Some(new_choice_rewards)
                }
            };
            let new_reward = StateChoiceRewards {
                states: new_state_rewards,
                choices: new_choice_rewards,
                branches: (),
            };

            rewards.add_entry(name.to_string(), new_reward);
        }
        rewards
    }
}
