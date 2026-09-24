use crate::ModelBuilder;
use crate::synchronised_actions::SynchronisedActions;
use prism_model::{Identifier, Model, RewardsTarget, Span, VariableReference};
use probabilistic_models::annotations::{RewardAnnotations, Rewards, TypedAnnotation};
use std::marker::PhantomData;
use typed_index_collections::Index;

pub trait RewardsBuilder {
    type Rewards;
    type RewardIdx: Index;
    type StateIdx: Index;
    type ChoiceIdx: Index;

    fn stores_rewards() -> bool;
    fn register_rewards(
        &mut self,
        name: String,
        has_state_rewards: bool,
        has_choice_rewards: bool,
    ) -> Self::RewardIdx;
    fn add_state_reward(&mut self, rewards: Self::RewardIdx, state: Self::StateIdx, value: f64);
    fn add_choice_reward(&mut self, rewards: Self::RewardIdx, choice: Self::ChoiceIdx, value: f64);
    fn into_rewards(self) -> Self::Rewards;
}

#[derive(Default)]
pub struct NoRewards<RewardIdx: Index, StateIdx: Index, ChoiceIdx: Index> {
    _phantom_data: PhantomData<(RewardIdx, StateIdx, ChoiceIdx)>,
}

impl<RewardIdx: Index, StateIdx: Index, ChoiceIdx: Index> RewardsBuilder
    for NoRewards<RewardIdx, StateIdx, ChoiceIdx>
{
    type Rewards = ();
    type RewardIdx = RewardIdx;
    type StateIdx = StateIdx;
    type ChoiceIdx = ChoiceIdx;

    fn stores_rewards() -> bool {
        false
    }

    fn register_rewards(
        &mut self,
        _name: String,
        _has_state_rewards: bool,
        _has_choice_rewards: bool,
    ) -> Self::RewardIdx {
        panic!("Cannot register rewards when using `NoRewards`")
    }

    fn add_state_reward(&mut self, _rewards: RewardIdx, _state: StateIdx, _value: f64) {
        panic!("Cannot store rewards when using `NoRewards`")
    }

    fn add_choice_reward(&mut self, _rewards: RewardIdx, _choice: ChoiceIdx, _value: f64) {
        panic!("Cannot store rewards when using `NoRewards`")
    }

    fn into_rewards(self) -> Self::Rewards {
        ()
    }
}

impl<
    'a,
    RewardIdx: Index,
    S: Span,
    Q: crate::queries::QueryCollection,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    B: crate::bases::BaseModelBuilder,
    IB: crate::initial_states_builder::InitialStatesBuilder<StateIdx = B::StateIdx>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = B::StateIdx>,
    CL: crate::choice_labels::ChoiceLabelBuilder<ChoiceIdx = B::ChoiceIdx>,
> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, CL, NoRewards<RewardIdx, B::StateIdx, B::ChoiceIdx>>
{
    pub fn with_all_rewards<AnnotationEntryIdx: Index>(
        self,
    ) -> ModelBuilder<
        'a,
        S,
        Q,
        L,
        IS,
        B,
        IB,
        APs,
        CL,
        RewardVectorsBuilder<RewardIdx, B::StateIdx, B::ChoiceIdx, AnnotationEntryIdx>,
    > {
        self.map_rewards(RewardVectorsBuilder::default())
    }
}

#[derive(Default)]
pub struct RewardVectorsBuilder<
    RewardIdx: Index,
    StateIdx: Index,
    ChoiceIdx: Index,
    AnnotationEntryIdx: Index,
> {
    rewards: RewardAnnotations<RewardIdx, StateIdx, ChoiceIdx, AnnotationEntryIdx>,
}

impl<RewardIdx: Index, StateIdx: Index, ChoiceIdx: Index, AnnotationEntryIdx: Index> RewardsBuilder
    for RewardVectorsBuilder<RewardIdx, StateIdx, ChoiceIdx, AnnotationEntryIdx>
{
    type Rewards = RewardAnnotations<RewardIdx, StateIdx, ChoiceIdx, AnnotationEntryIdx>;
    type RewardIdx = RewardIdx;
    type StateIdx = StateIdx;
    type ChoiceIdx = ChoiceIdx;

    fn stores_rewards() -> bool {
        true
    }

    fn register_rewards(
        &mut self,
        name: String,
        has_state_rewards: bool,
        has_choice_rewards: bool,
    ) -> Self::RewardIdx {
        let rewards = Rewards {
            states: has_state_rewards.then(TypedAnnotation::default),
            choices: has_choice_rewards.then(TypedAnnotation::default),
            branches: (),
        };
        self.rewards.add_entry(name, rewards)
    }

    fn add_state_reward(&mut self, rewards: RewardIdx, state: StateIdx, value: f64) {
        let rewards = self.rewards.get_mut(rewards).unwrap();
        rewards.states.as_mut().unwrap().add_value(state, value);
    }

    fn add_choice_reward(&mut self, rewards: RewardIdx, choice: ChoiceIdx, value: f64) {
        let rewards = self.rewards.get_mut(rewards).unwrap();
        rewards.choices.as_mut().unwrap().add_value(choice, value);
    }

    fn into_rewards(self) -> Self::Rewards {
        self.rewards
    }
}

impl<
    'a,
    RewardIdx: Index,
    AnnotationEntryIdx: Index,
    S: Span,
    Q: crate::queries::QueryCollection,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    B: crate::bases::BaseModelBuilder,
    IB: crate::initial_states_builder::InitialStatesBuilder<StateIdx = B::StateIdx>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = B::StateIdx>,
    CL: crate::choice_labels::ChoiceLabelBuilder<ChoiceIdx = B::ChoiceIdx>,
>
    ModelBuilder<
        'a,
        S,
        Q,
        L,
        IS,
        B,
        IB,
        APs,
        CL,
        RewardVectorsBuilder<RewardIdx, B::StateIdx, B::ChoiceIdx, AnnotationEntryIdx>,
    >
{
    pub fn without_rewards(
        self,
    ) -> ModelBuilder<
        'a,
        S,
        Q,
        L,
        IS,
        B,
        IB,
        APs,
        CL,
        NoRewards<RewardIdx, B::StateIdx, B::ChoiceIdx>,
    > {
        self.map_rewards(NoRewards::default())
    }
}

pub(crate) struct RewardEntry<'m, E> {
    pub condition: &'m E,
    pub value: &'m E,
}

pub(crate) struct RewardStructure<'m, RewardIdx, E> {
    pub index: RewardIdx,
    pub name: String,
    pub state_entries: Vec<RewardEntry<'m, E>>,
    pub unlabelled_entries: Vec<RewardEntry<'m, E>>,
    pub synchronised_entries: Vec<Vec<RewardEntry<'m, E>>>,
    pub has_choice_rewards: bool,
}

pub(crate) struct RewardStructures<'m, RewardIdx, E> {
    pub structures: Vec<RewardStructure<'m, RewardIdx, E>>,
}

impl<'m, RewardIdx: Index, E> RewardStructures<'m, RewardIdx, E> {
    pub fn new<S: Span, Rew: RewardsBuilder<RewardIdx = RewardIdx>>(
        model: &'m Model<VariableReference, S, E, Identifier<S>>,
        synchronised_actions: &SynchronisedActions,
        rewards_builder: &mut Rew,
    ) -> Self {
        if !Rew::stores_rewards() {
            return Self {
                structures: Vec::new(),
            };
        }
        let mut structures = Vec::new();
        for (index, rewards) in model.rewards.iter().enumerate() {
            let name = match &rewards.name {
                Some(name) => name.name.clone(),
                None => format!("#{index}"),
            };
            let mut state_entries = Vec::new();
            let mut unlabelled_entries = Vec::new();
            let mut synchronised_entries = (0..synchronised_actions.len())
                .map(|_| Vec::new())
                .collect::<Vec<_>>();
            let mut has_choice_rewards = false;
            for element in &rewards.entries {
                let entry = RewardEntry {
                    condition: &element.condition,
                    value: &element.value,
                };
                match &element.target {
                    RewardsTarget::State => state_entries.push(entry),
                    RewardsTarget::Action(None) => {
                        has_choice_rewards = true;
                        unlabelled_entries.push(entry);
                    }
                    RewardsTarget::Action(Some(action)) => {
                        has_choice_rewards = true;
                        if let Some(index) = synchronised_actions.index_of(&action.name) {
                            synchronised_entries[index].push(entry);
                        }
                    }
                }
            }
            let index = rewards_builder.register_rewards(
                name.clone(),
                !state_entries.is_empty(),
                has_choice_rewards,
            );
            structures.push(RewardStructure {
                index,
                name,
                state_entries,
                unlabelled_entries,
                synchronised_entries,
                has_choice_rewards,
            });
        }
        Self { structures }
    }
}

#[cfg(test)]
mod tests {
    use crate::ModelBuilder;
    use probabilistic_models::labels::ReadLabels;
    use probabilistic_models::traits::{ReadRewards, ReadStateSpace};
    use probabilistic_models::{Index, StateIndex};

    fn state(index: u32) -> StateIndex<u32> {
        StateIndex::from_raw(index)
    }

    #[test]
    fn state_and_unlabelled_rewards() {
        let mut prism = prism_parser::parse_model(
            r#"mdp
            module m
                x : [0..3] init 0;
                [] x<3 -> (x'=x+1);
            endmodule
            rewards "steps"
                true : 1;
                x>=2 : x;
            endrewards
            rewards
                x=1 : 0.5;
            endrewards
            rewards
                [] true : 2;
            endrewards"#,
        )
        .unwrap();
        let model = ModelBuilder::new_mdp_builder(&mut prism).build();

        let steps = model.reward_structure(Some("steps")).unwrap();
        let unnamed_state = model.reward_structure(Some("#1")).unwrap();
        let unnamed_choice = model.reward_structure(Some("#2")).unwrap();
        assert_eq!(model.reward_structure(None), Some(steps));
        assert!(model.has_state_rewards(steps));
        assert!(!model.has_choice_rewards(steps));
        assert!(!model.has_state_rewards(unnamed_choice));
        assert!(model.has_choice_rewards(unnamed_choice));

        let expected_steps = [1.0, 1.0, 3.0, 4.0];
        let expected_unnamed = [0.0, 0.5, 0.0, 0.0];
        for s in 0..4 {
            assert_eq!(
                model.state_reward(steps, state(s)),
                expected_steps[s as usize]
            );
            assert_eq!(
                model.state_reward(unnamed_state, state(s)),
                expected_unnamed[s as usize]
            );
        }

        // State 3 is a deadlock, the added self-loop receives no transition reward
        let expected_choice = [2.0, 2.0, 2.0, 0.0];
        for s in 0..4 {
            let choices = model.choices_of_state(state(s));
            assert_eq!(choices.len(), 1);
            for choice in choices {
                assert_eq!(
                    model.choice_reward(unnamed_choice, choice),
                    expected_choice[s as usize]
                );
            }
        }
    }

    #[test]
    fn synchronised_rewards() {
        let mut prism = prism_parser::parse_model(
            r#"mdp
            module m1
                x : [0..2] init 0;
                [a] x=0 -> (x'=1);
                [a] x=0 -> (x'=2);
                [b] x=0 -> (x'=2);
                [] x=1 -> (x'=2);
            endmodule
            module m2
                y : [0..1] init 0;
                [a] true -> true;
                [c] true -> true;
            endmodule
            rewards
                [a] true : 1;
                [a] x=0 : 2;
                [b] true : 5;
                [] true : 7;
                [d] true : 100;
            endrewards"#,
        )
        .unwrap();
        let model = ModelBuilder::new_mdp_builder(&mut prism).build();
        let rewards = model.reward_structure(None).unwrap();
        assert!(!model.has_state_rewards(rewards));

        let mut a_choices = 0;
        for choice in model.choices() {
            let expected = match model.choice_labels.label(choice).as_deref() {
                Some("a") => {
                    a_choices += 1;
                    3.0
                }
                Some("b") => 5.0,
                Some("c") => 0.0,
                None => 7.0,
                Some(other) => panic!("Unexpected action {other}"),
            };
            assert_eq!(model.choice_reward(rewards, choice), expected);
        }
        assert_eq!(a_choices, 2);
    }

    #[test]
    #[should_panic(expected = "invalid value -1")]
    fn negative_rewards() {
        let mut prism = prism_parser::parse_model(
            r#"mdp
            module m
                x : [0..1] init 0;
                [] x<1 -> (x'=x+1);
            endmodule
            rewards
                x=1 : -1;
            endrewards"#,
        )
        .unwrap();
        ModelBuilder::new_mdp_builder(&mut prism).build();
    }
}
