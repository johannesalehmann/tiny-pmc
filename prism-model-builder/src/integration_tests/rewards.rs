use super::*;
use probabilistic_models::RewardIndex;
use probabilistic_models::annotations::RewardAnnotations;
use probabilistic_models::base_model::{Dtmc, ReadOwners, StochasticGame, TwoPlayer};
use probabilistic_models::traits::{ReadChoiceLabels, ReadRewards};

/// State rewards of all states, in state order.
fn rewards_of_states<M: ReadStateSpace + ReadRewards>(
    model: &M,
    r: RewardIndex<M::Ix>,
) -> Vec<f64> {
    model
        .states()
        .into_iter()
        .map(|s| model.state_reward(r, s))
        .collect()
}

/// Action name and reward of all choices, in choice order.
fn choice_rewards<M>(model: &M, r: RewardIndex<M::Ix>) -> Vec<(Option<&str>, f64)>
where
    M: ReadStateSpace + ReadRewards + ReadChoiceLabels<E = Option<String>>,
{
    model
        .choices()
        .into_iter()
        .map(|c| (model.choice_label(c).as_deref(), model.choice_reward(r, c)))
        .collect()
}

#[test]
fn state_rewards() {
    let model = build!(
        "dtmc module m x : [0..2] init 0; [] x<2 -> (x'=x+1); endmodule \
         rewards true : 1; x>=1 : 2; x=2 : 0.5; endrewards",
        .with_dtmc_model_type()
    )
    .unwrap();
    let _: &RewardAnnotations<Ix> = &model.rewards;
    let r = model.reward_structure(None).unwrap();
    assert_eq!(valuations(&model), ["x=0", "x=1", "x=2"]);
    assert_eq!(rewards_of_states(&model, r), [1.0, 3.0, 3.5]);
}

#[test]
fn unlabelled_transition_rewards() {
    const SOURCE: &str = r#"
        mdp
        module m
            x : [0..2] init 0;
            [] x=0 -> (x'=1);
            [a] x=0 -> (x'=2);
            [] x=1 -> (x'=2);
            [a] x=2 -> true;
        endmodule
        rewards
            [] true : 1;
            [] x=0 : 2;
        endrewards"#;
    let model = build!(SOURCE, .with_mdp_model_type()).unwrap();
    let r = model.reward_structure(None).unwrap();
    assert_eq!(valuations(&model), ["x=0", "x=1", "x=2"]);
    assert_eq!(
        choice_rewards(&model, r),
        [(None, 3.0), (Some("a"), 0.0), (None, 1.0), (Some("a"), 0.0)]
    );
}

#[test]
fn action_transition_rewards() {
    const SOURCE: &str = r#"
        mdp
        module m1
            x : [0..2] init 0;
            [] x=0 -> (x'=1);
            [a] x=0 -> (x'=1);
            [b] x=0 -> (x'=2);
            [a] x=1 -> (x'=2);
            [b] x=2 -> true;
        endmodule
        module m2
            [a] true -> true;
        endmodule
        rewards
            [a] true : 1;
            [a] x=0 : 2;
            [b] true : 4;
        endrewards"#;
    let model = build!(SOURCE, .with_mdp_model_type()).unwrap();
    let r = model.reward_structure(None).unwrap();
    assert_eq!(valuations(&model), ["x=0", "x=1", "x=2"]);
    assert_eq!(
        choice_rewards(&model, r),
        [
            (None, 0.0),
            (Some("a"), 3.0),
            (Some("b"), 4.0),
            (Some("a"), 1.0),
            (Some("b"), 4.0)
        ]
    );
}

#[test]
fn multiple_named_and_unnamed_structures() {
    const SOURCE: &str = r#"
        dtmc
        module m x : bool init false; [] true -> true; endmodule
        rewards "first" true : 1; endrewards
        rewards true : 2; endrewards
        rewards "second" true : 3; endrewards
        rewards [] true : 4; endrewards"#;
    let model = build!(SOURCE, .with_dtmc_model_type()).unwrap();
    let names: Vec<_> = model.reward_structures().map(|(_, name)| name).collect();
    assert_eq!(names, ["first", "#1", "second", "#3"]);
    assert_eq!(
        model.reward_structure(None),
        model.reward_structure(Some("first"))
    );
    assert_eq!(model.reward_structure(Some("third")), None);

    let s = state(&model, "x=false");
    let by_name = |name| model.reward_structure(Some(name)).unwrap();
    assert_eq!(model.state_reward(by_name("first"), s), 1.0);
    assert_eq!(model.state_reward(by_name("#1"), s), 2.0);
    assert_eq!(model.state_reward(by_name("second"), s), 3.0);
    assert_eq!(choice_rewards(&model, by_name("#3")), [(None, 4.0)]);
}

#[test]
fn has_state_vs_choice_rewards() {
    const SOURCE: &str = r#"
        mdp
        module m x : bool init false; [a] true -> true; endmodule
        rewards "state" true : 1; endrewards
        rewards "unlabelled" [] true : 1; endrewards
        rewards "action" [a] true : 1; endrewards
        rewards "both" true : 1; [a] true : 1; endrewards"#;
    let model = build!(SOURCE, .with_mdp_model_type()).unwrap();
    let flags: Vec<_> = model
        .reward_structures()
        .map(|(r, name)| {
            (
                name,
                model.has_state_rewards(r),
                model.has_choice_rewards(r),
            )
        })
        .collect();
    assert_eq!(
        flags,
        [
            ("state", true, false),
            ("unlabelled", false, true),
            ("action", false, true),
            ("both", true, true),
        ]
    );
}

#[test]
fn deadlock_fix_has_zero_transition_reward() {
    let model = build!(
        "mdp module m x : [0..1] init 0; [] x=0 -> (x'=1); endmodule rewards [] true : 1; endrewards",
        .with_mdp_model_type()
    )
    .unwrap();
    let r = model.reward_structure(None).unwrap();
    assert_eq!(valuations(&model), ["x=0", "x=1"]);
    assert_eq!(choice_rewards(&model, r), [(None, 1.0), (None, 0.0)]);
}

#[test]
fn rewards_on_dtmc_and_game() {
    let dtmc = build!(
        "dtmc module m x : [0..2] init 0; [] x<2 -> 0.5: (x'=x+1) + 0.5: (x'=2); \
         [] x=2 -> true; endmodule rewards [] true : x+1; endrewards",
        .with_dtmc_model_type()
    )
    .unwrap();
    let _: &Dtmc<Ix> = &dtmc.base;
    let r = dtmc.reward_structure(None).unwrap();
    assert_eq!(valuations(&dtmc), ["x=0", "x=1", "x=2"]);
    assert_eq!(
        choice_rewards(&dtmc, r),
        [(None, 1.0), (None, 2.0), (None, 3.0)]
    );

    const GAME: &str = r#"
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
            [] x=1 & y=0 -> (y'=1);
        endmodule
        rewards
            [] x=0 : 1;
            [] y=0 : 2;
        endrewards"#;
    let game = build!(GAME, .with_stochastic_game_model_type()).unwrap();
    let _: &StochasticGame<Ix> = &game.base;
    let r = game.reward_structure(None).unwrap();
    let owners: Vec<_> = game
        .states()
        .into_iter()
        .map(|s| game.base.state_owner(s))
        .collect();
    assert_eq!(owners, [TwoPlayer::Eve, TwoPlayer::Adam, TwoPlayer::Eve]);
    assert_eq!(
        choice_rewards(&game, r),
        [(None, 3.0), (None, 3.0), (None, 2.0), (None, 0.0)]
    );
}

#[test]
fn without_rewards() {
    let model = build!(
        "dtmc module m x : bool init false; endmodule rewards true : 1; endrewards",
        .without_rewards()
    )
    .unwrap();
    let _: &() = &model.rewards;
}

#[test]
#[should_panic(expected = "must be finite and non-negative")]
fn negative_reward_panics() {
    let _ = build!(
        "mdp module m x : bool init false; [] true -> true; endmodule rewards [] true : -0.5; endrewards"
    );
}
