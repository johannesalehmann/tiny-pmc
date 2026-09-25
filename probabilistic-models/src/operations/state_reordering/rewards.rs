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
        let ordering_n = ordering.len();
        let old_n = old_base.states().len();
        let new_n = new_base.states().len();
        assert_eq!(
            old_n, ordering_n,
            "`old_base` model has {old_n} states, but `ordering` covers {ordering_n} states."
        );
        assert_eq!(
            new_n, ordering_n,
            "`new_base` has {new_n} states, but `ordering` covers {ordering_n} states."
        );
        for (old_state, &new_state) in ordering.old_to_new.enumerate() {
            let old_choices = old_base.choices_of_state(old_state).len();
            let new_choices = new_base.choices_of_state(new_state).len();
            assert_eq!(
                old_choices, new_choices,
                "State {old_state:?} has {old_choices} choices in the old base model, but its new state {new_state:?} has {new_choices} choices in the new base model."
            );
        }

        let mut rewards = RewardAnnotations::new();
        for (name, old_reward) in self {
            let new_state_rewards = match &old_reward.states {
                None => None,
                Some(old_state_rewards) => {
                    let n = old_state_rewards.values().len();
                    assert_eq!(
                        n, ordering_n,
                        "State rewards `{name}` cover {n} states, but the state ordering covers {ordering_n} states."
                    );
                    let mut new_state_rewards =
                        EntityRewards::with_identity_distribution_and_entries(To1::with_entries(
                            vec![0.0; n],
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
                    let n = old_choice_rewards.values().len();
                    let base_n = old_base.choices().len();
                    assert_eq!(
                        n, base_n,
                        "Choice rewards `{name}` cover {n} choices, but the base model has {base_n} choices."
                    );
                    let mut new_choice_rewards =
                        EntityRewards::with_identity_distribution_and_entries(To1::with_entries(
                            vec![0.0; n],
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

#[cfg(test)]
mod tests {
    use super::{PermuteStatesWithContext, StateOrdering};
    use crate::annotations::{EntityRewards, RewardAnnotations, StateChoiceRewards};
    use crate::base_model::Mdp;
    use crate::operations::state_reordering::PermuteStates;
    use crate::operations::state_reordering::test_utils::{state, state_ordering};
    use crate::{AnnotationEntryIndex, AnnotationIndex, BranchIndex, ChoiceIndex, StateIndex};
    use typed_index_collections::To1;

    type Base = Mdp<StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>>;
    type Rewards = RewardAnnotations<
        AnnotationIndex<usize>,
        StateIndex<usize>,
        ChoiceIndex<usize>,
        AnnotationEntryIndex<usize>,
    >;

    /// An MDP with the given number of choices per state. Every choice is a self-loop.
    fn base(choices_per_state: &[usize]) -> Base {
        let mut base = Mdp::with_default_types();
        for (index, &choices) in choices_per_state.iter().enumerate() {
            base.add_state(state(index));
            for _ in 0..choices {
                base.add_choice_from_slice(&[(1.0, state(index))]);
            }
        }
        base
    }

    fn entity_rewards<E: typed_index_collections::Index>(
        values: Option<&[f64]>,
    ) -> Option<EntityRewards<E, AnnotationEntryIndex<usize>>> {
        values.map(|values| {
            EntityRewards::with_identity_distribution_and_entries(To1::with_entries(
                values.to_vec(),
            ))
        })
    }

    // Reward models given as `(name, state rewards, choice rewards)`.
    fn rewards(entries: &[(&str, Option<&[f64]>, Option<&[f64]>)]) -> Rewards {
        let mut rewards = RewardAnnotations::new();
        for &(name, states, choices) in entries {
            rewards.add_entry(
                name.to_string(),
                StateChoiceRewards {
                    states: entity_rewards(states),
                    choices: entity_rewards(choices),
                    branches: (),
                },
            );
        }
        rewards
    }

    /// Permutes `rewards`, deriving the new base model by permuting `old_base`.
    fn permuted(
        rewards: &Rewards,
        old_base: &Base,
        ordering: &StateOrdering<StateIndex<usize>>,
    ) -> Rewards {
        let new_base = old_base.permute_states(ordering);
        rewards.permute_states_with_context(ordering, old_base, &new_base)
    }

    #[test]
    fn state_rewards_identity() {
        let input = rewards(&[("a", Some(&[1.0, 2.0, 3.0]), None)]);
        let ordering = state_ordering(vec![0, 1, 2]);
        assert_eq!(permuted(&input, &base(&[1, 1, 1]), &ordering), input);
    }

    #[test]
    fn state_rewards_cycle() {
        let input = rewards(&[("a", Some(&[1.0, 2.0, 3.0]), None)]);
        let ordering = state_ordering(vec![1, 2, 0]);
        let expected = rewards(&[("a", Some(&[3.0, 1.0, 2.0]), None)]);
        assert_eq!(permuted(&input, &base(&[1, 1, 1]), &ordering), expected);
    }

    #[test]
    fn choice_rewards_identity() {
        let input = rewards(&[("a", None, Some(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]))]);
        let ordering = state_ordering(vec![0, 1, 2]);
        assert_eq!(permuted(&input, &base(&[2, 1, 3]), &ordering), input);
    }

    #[test]
    fn choice_rewards_varying_choice_counts() {
        // old: s0 = {c0, c1}, s1 = {}, s2 = {c2}, s3 = {c3, c4, c5}
        // new: s0 = {}, s1 = {c0, c1, c2}, s2 = {c3, c4}, s3 = {c5}
        //  where the new choices are, in order, the old choices c3, c4, c5, c0, c1, c2.
        let input = rewards(&[("a", None, Some(&[10.0, 11.0, 12.0, 13.0, 14.0, 15.0]))]);
        let ordering = state_ordering(vec![2, 0, 3, 1]);
        let expected = rewards(&[("a", None, Some(&[13.0, 14.0, 15.0, 10.0, 11.0, 12.0]))]);
        assert_eq!(permuted(&input, &base(&[2, 0, 1, 3]), &ordering), expected);
    }

    #[test]
    fn choice_rewards_state_without_choices() {
        let input = rewards(&[("a", None, Some(&[5.0, 6.0]))]);
        let ordering = state_ordering(vec![2, 1, 0]);
        let expected = rewards(&[("a", None, Some(&[6.0, 5.0]))]);
        assert_eq!(permuted(&input, &base(&[1, 0, 1]), &ordering), expected);
    }

    #[test]
    fn multiple_reward_models() {
        // old: s0 = {c0, c1}, s1 = {c2}, s2 = {c3}
        // new choices are, in order, the old choices c3, c0, c1, c2
        let input = rewards(&[
            ("s", Some(&[1.0, 2.0, 3.0]), None),
            ("c", None, Some(&[10.0, 11.0, 12.0, 13.0])),
            (
                "sc",
                Some(&[4.0, 5.0, 6.0]),
                Some(&[20.0, 21.0, 22.0, 23.0]),
            ),
            ("none", None, None),
        ]);
        let ordering = state_ordering(vec![1, 2, 0]);
        let expected = rewards(&[
            ("s", Some(&[3.0, 1.0, 2.0]), None),
            ("c", None, Some(&[13.0, 10.0, 11.0, 12.0])),
            (
                "sc",
                Some(&[6.0, 4.0, 5.0]),
                Some(&[23.0, 20.0, 21.0, 22.0]),
            ),
            ("none", None, None),
        ]);
        let output = permuted(&input, &base(&[2, 1, 1]), &ordering);
        assert_eq!(output, expected);
        // Reward models keep their names and their indices.
        assert_eq!(output.names(), input.names());
    }

    #[test]
    fn no_reward_models() {
        let ordering = state_ordering(vec![2, 0, 1]);
        assert_eq!(
            permuted(&rewards(&[]), &base(&[1, 2, 0]), &ordering),
            rewards(&[])
        );
    }

    #[test]
    fn round_trip() {
        let old_base = base(&[2, 0, 1, 3]);
        let input = rewards(&[(
            "a",
            Some(&[1.0, 2.0, 3.0, 4.0]),
            Some(&[10.0, 11.0, 12.0, 13.0, 14.0, 15.0]),
        )]);
        let forward = state_ordering(vec![2, 0, 3, 1]);
        let backward = StateOrdering::new(forward.new_to_old.clone());
        let new_base = old_base.permute_states(&forward);
        let forward_permuted = permuted(&input, &old_base, &forward);
        assert_ne!(forward_permuted, input);
        assert_eq!(permuted(&forward_permuted, &new_base, &backward), input);
    }

    #[test]
    fn no_states() {
        let input = rewards(&[("a", Some(&[]), Some(&[]))]);
        let ordering = state_ordering(vec![]);
        assert_eq!(permuted(&input, &base(&[]), &ordering), input);
    }

    #[test]
    #[should_panic(expected = "State rewards `b` cover 2 states, but the state ordering covers 3")]
    fn too_few_state_rewards() {
        let input = rewards(&[
            ("a", Some(&[1.0, 2.0, 3.0]), None),
            ("b", Some(&[1.0, 2.0]), None),
        ]);
        permuted(&input, &base(&[1, 1, 1]), &state_ordering(vec![1, 2, 0]));
    }

    #[test]
    #[should_panic(expected = "State rewards `b` cover 4 states, but the state ordering covers 3")]
    fn too_many_state_rewards() {
        let input = rewards(&[
            ("a", Some(&[1.0, 2.0, 3.0]), None),
            ("b", Some(&[1.0, 2.0, 3.0, 4.0]), None),
        ]);
        permuted(&input, &base(&[1, 1, 1]), &state_ordering(vec![1, 2, 0]));
    }

    #[test]
    #[should_panic(expected = "Choice rewards `b` cover 3 choices, but the base model has 4")]
    fn too_few_choice_rewards() {
        let input = rewards(&[
            ("a", None, Some(&[1.0, 2.0, 3.0, 4.0])),
            ("b", None, Some(&[1.0, 2.0, 3.0])),
        ]);
        permuted(&input, &base(&[2, 1, 1]), &state_ordering(vec![1, 2, 0]));
    }

    #[test]
    #[should_panic(expected = "Choice rewards `b` cover 5 choices, but the base model has 4")]
    fn too_many_choice_rewards() {
        let input = rewards(&[
            ("a", None, Some(&[1.0, 2.0, 3.0, 4.0])),
            ("b", None, Some(&[1.0, 2.0, 3.0, 4.0, 5.0])),
        ]);
        permuted(&input, &base(&[2, 1, 1]), &state_ordering(vec![1, 2, 0]));
    }

    #[test]
    #[should_panic(expected = "`old_base` model has 2 states, but `ordering` covers 3")]
    fn old_base_wrong_state_count() {
        let ordering = state_ordering(vec![1, 2, 0]);
        rewards(&[]).permute_states_with_context(&ordering, &base(&[1, 1]), &base(&[1, 1, 1]));
    }

    #[test]
    #[should_panic(expected = "`new_base` has 4 states, but `ordering` covers 3")]
    fn new_base_wrong_state_count() {
        let ordering = state_ordering(vec![1, 2, 0]);
        rewards(&[]).permute_states_with_context(
            &ordering,
            &base(&[1, 1, 1]),
            &base(&[1, 1, 1, 1]),
        );
    }

    #[test]
    #[should_panic(
        expected = "State StateIndex (0) has 2 choices in the old base model, but its new state StateIndex (1) has 1 choices"
    )]
    fn new_base_wrong_choice_structure() {
        // Swapping the states would need the new base model to have the choice counts [1, 2].
        let ordering = state_ordering(vec![1, 0]);
        rewards(&[]).permute_states_with_context(&ordering, &base(&[2, 1]), &base(&[2, 1]));
    }
}
