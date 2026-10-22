use super::*;
use crate::ModelBuilder;
use crate::choice_labels::{CommandIndexComponent, CommandIndices};
use crate::queries::{ModelAndQuery, ProcessedQuery};
use probabilistic_models::AtomicPropositionIndex;
use probabilistic_models::annotations::{AtomicPropositions, RewardAnnotations};
use probabilistic_models::base_model::{Dtmc, ReadOwners, ReadPlayers, StochasticGame};
use probabilistic_models::initial_states::InitialStates;
use probabilistic_models::labels::ChoiceLabels;
use probabilistic_models::traits::{ReadChoiceLabels, ReadInitialStates, ReadRewards};
use probabilistic_models::{mdp, smg};

/// Parses `$source` and `$property`, and builds the model with every component enabled.
macro_rules! build_everything {
    ($source:expr, $property:expr, $($model_type:tt)*) => {{
        let parsed = prism_parser::parse_model_and_prop($source, $property);
        let mut prism = parsed.model.unwrap();
        ModelBuilder::new(&mut prism)
            $($model_type)*
            .with_all_labels()
            .label_choices_with_command_indices()
            .with_initial_state_vector()
            .with_query(parsed.property.unwrap().query)
            .build()
            .unwrap()
    }};
}

/// Command-index label, state reward of the source state and choice reward of every choice.
fn choice_summary<M>(model: &M) -> Vec<(CommandIndices, f64, f64)>
where
    M: ReadStateSpace + ReadRewards + ReadChoiceLabels<E = CommandIndices>,
{
    let r = model.reward_structure(None).unwrap();
    model
        .states()
        .into_iter()
        .flat_map(|s| model.choices_of_state(s).into_iter().map(move |c| (s, c)))
        .map(|(s, c)| {
            let label = model.choice_label(c).clone();
            (label, model.state_reward(r, s), model.choice_reward(r, c))
        })
        .collect()
}

fn unsynchronised(module_index: usize, command_index: usize) -> CommandIndices {
    CommandIndices::Unsynchronised {
        module_index,
        command_index,
    }
}

fn synchronised(module_index: usize, command_index: usize) -> CommandIndices {
    CommandIndices::Synchronised {
        components: vec![CommandIndexComponent {
            module_index,
            command_index,
        }],
    }
}

#[test]
fn everything_enabled_dtmc() {
    const SOURCE: &str = r#"
        dtmc
        module m
            x : [0..2] init 0;
            [] x=0 -> 0.5: (x'=1) + 0.5: (x'=2);
            [step] x>0 -> (x'=0);
        endmodule
        label "goal" = x=2;
        label "start" = x=0;
        rewards
            x=1 : 2;
            [step] true : 1;
            [] true : 0.5;
        endrewards"#;
    let ModelAndQuery { model, query } =
        build_everything!(SOURCE, r#"P=? [F "goal"]"#, .with_dtmc_model_type());
    let _: &Dtmc<Ix> = &model.base;
    let _: &InitialStates<Ix> = &model.initial;
    let _: &ChoiceLabels<Ix, CommandIndices> = &model.choice_labels;
    let _: &AtomicPropositions<Ix> = &model.atomic_propositions;
    let _: &RewardAnnotations<Ix> = &model.rewards;
    let _: &ProcessedQuery<AtomicPropositionIndex<Ix>> = &query;

    mdp!(expected = { s0 -> 0.5: s1 & 0.5: s2, s1 -> 1.0: s0, s2 -> 1.0: s0 });
    assert_eq!(valuations(&model), ["x=0", "x=1", "x=2"]);
    assert_eq!(transitions(&model), transitions(&expected));
    assert!(model.is_initial(state(&model, "x=0")));
    assert!(!model.is_initial(state(&model, "x=1")));
    assert_eq!(states_with(&model, "goal"), ["x=2"]);
    assert_eq!(states_with(&model, "start"), ["x=0"]);
    assert_eq!(
        choice_summary(&model),
        [
            (unsynchronised(0, 0), 0.0, 0.5),
            (synchronised(0, 1), 2.0, 1.0),
            (synchronised(0, 1), 0.0, 1.0),
        ]
    );
}

#[test]
fn everything_enabled_game() {
    const SOURCE: &str = r#"
        smg
        player p1 m1 endplayer
        player p2 m2 endplayer
        module m1
            x : [0..1] init 0;
            [] x=0 -> 0.5: (x'=1) + 0.5: true;
            [] x=0 -> (x'=1);
        endmodule
        module m2
            y : [0..1] init 0;
            [sync] x=1 & y=0 -> (y'=1);
        endmodule
        label "done" = y=1;
        rewards
            y=1 : 4;
            [] true : 1;
            [sync] true : 2;
        endrewards"#;
    let ModelAndQuery { model, query } = build_everything!(
        SOURCE,
        r#"Pmax=? [F "done"]"#,
        .with_stochastic_game_model_type()
    );
    let _: &StochasticGame<Ix> = &model.base;
    let _: &InitialStates<Ix> = &model.initial;
    let _: &ChoiceLabels<Ix, CommandIndices> = &model.choice_labels;
    let _: &AtomicPropositions<Ix> = &model.atomic_propositions;
    let _: &RewardAnnotations<Ix> = &model.rewards;
    let _: &ProcessedQuery<AtomicPropositionIndex<Ix>> = &query;

    smg!(expected = {
        players: [p1, p2],
        s0 (p1) -> 0.5: s1 & 0.5: s0,
        s0 (p1) -> 1.0: s1,
        s1 (p2) -> 1.0: s2,
        s2 (p1) -> 1.0: s2,
    });
    assert_eq!(valuations(&model), ["x=0,y=0", "x=1,y=0", "x=1,y=1"]);
    assert_eq!(transitions(&model), transitions(&expected));
    let owners: Vec<_> = model
        .states()
        .into_iter()
        .map(|s| model.base.player_name(model.base.state_owner(s)))
        .collect();
    assert_eq!(owners, ["p1", "p2", "p1"]);
    assert!(model.is_initial(state(&model, "x=0,y=0")));
    assert!(!model.is_initial(state(&model, "x=1,y=0")));
    assert_eq!(states_with(&model, "done"), ["x=1,y=1"]);
    assert_eq!(
        choice_summary(&model),
        [
            (unsynchronised(0, 0), 0.0, 1.0),
            (unsynchronised(0, 1), 0.0, 1.0),
            (synchronised(1, 0), 0.0, 2.0),
            (CommandIndices::DeadlockFix, 4.0, 0.0),
        ]
    );
}
