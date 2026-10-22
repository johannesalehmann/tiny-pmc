use super::*;
use crate::ModelBuildingError;
use probabilistic_models::base_model::{
    AnyBaseModel, BaseModelType, Dtmc, Mdp, NonstochasticGame, StochasticGame, TransitionSystem,
};
use probabilistic_models::{mdp, smg};

const TS: &str = "module m x: [0..2] init 0; [] x<2 -> (x'=x+1); endmodule";
const DTMC: &str = "module m x: [0..2] init 0; [] x<2 -> 0.5: (x'=x+1) + 0.5: true; endmodule";
const MDP: &str =
    "module m x: [0..2] init 0; [] x<2 -> 0.5: (x'=x+1) + 0.5: true; [] x=0 -> (x'=2); endmodule";

/// Player `p1` moves if `x=y`, player `p2` otherwise.
const STOCHASTIC_GAME: &str = r#"
    player p1 m1 endplayer
    player p2 m2 endplayer
    module m1 x: [0..1] init 0; [] x=y -> 0.5: (x'=1-x) + 0.5: true; endmodule
    module m2 y: [0..1] init 0; [] x!=y -> (y'=x); endmodule"#;

/// Like `STOCHASTIC_GAME`, but without probabilistic branching.
const NONSTOCHASTIC_GAME: &str = r#"
    player p1 m1 endplayer
    player p2 m2 endplayer
    module m1 x: [0..1] init 0; [] x=y -> (x'=1-x); endmodule
    module m2 y: [0..1] init 0; [] x!=y -> (y'=x); endmodule"#;

fn expected_ts() -> Mdp<Ix> {
    mdp!(m = { s0 -> 1.0: s1, s1 -> 1.0: s2, s2 -> 1.0: s2 });
    m
}

fn expected_dtmc() -> Mdp<Ix> {
    mdp!(m = {
        s0 -> 0.5: s1 & 0.5: s0,
        s1 -> 0.5: s2 & 0.5: s1,
        s2 -> 1.0: s2,
    });
    m
}

fn expected_mdp() -> Mdp<Ix> {
    mdp!(m = {
        s0 -> 0.5: s1 & 0.5: s0,
        s0 -> 1.0: s2,
        s1 -> 0.5: s2 & 0.5: s1,
        s2 -> 1.0: s2,
    });
    m
}

fn expected_stochastic_game() -> StochasticGame<Ix> {
    smg!(m = {
        players: [p1, p2],
        s00 (p1) -> 0.5: s10 & 0.5: s00,
        s10 (p2) -> 1.0: s11,
        s11 (p1) -> 0.5: s01 & 0.5: s11,
        s01 (p2) -> 1.0: s00,
    });
    m
}

fn expected_nonstochastic_game() -> StochasticGame<Ix> {
    smg!(m = {
        players: [p1, p2],
        s00 (p1) -> 1.0: s10,
        s10 (p2) -> 1.0: s11,
        s11 (p1) -> 1.0: s01,
        s01 (p2) -> 1.0: s00,
    });
    m
}

mod fixed {
    use super::*;

    #[test]
    fn fixed_dtmc() {
        let model = build!(DTMC, .with_dtmc_model_type()).unwrap();
        let _: &Dtmc<Ix> = &model.base;
        assert_eq!(transitions(&model), transitions(&expected_dtmc()));
    }

    #[test]
    fn fixed_transition_system() {
        let model = build!(TS, .with_transition_system_model_type()).unwrap();
        let _: &TransitionSystem<Ix> = &model.base;
        assert_eq!(transitions(&model), transitions(&expected_ts()));
    }

    #[test]
    fn fixed_mdp() {
        let model = build!(MDP, .with_mdp_model_type()).unwrap();
        let _: &Mdp<Ix> = &model.base;
        assert_eq!(transitions(&model), transitions(&expected_mdp()));
    }

    #[test]
    fn fixed_stochastic_game() {
        let model = build!(STOCHASTIC_GAME, .with_stochastic_game_model_type()).unwrap();
        let _: &StochasticGame<Ix> = &model.base;
        let expected = expected_stochastic_game();
        assert_eq!(transitions(&model), transitions(&expected));
        assert_eq!(model.base.players, expected.players);
    }

    #[test]
    fn fixed_nonstochastic_game() {
        let model = build!(NONSTOCHASTIC_GAME, .with_nonstochastic_game_model_type()).unwrap();
        let _: &NonstochasticGame<Ix> = &model.base;
        let expected = expected_nonstochastic_game();
        assert_eq!(transitions(&model), transitions(&expected));
        assert_eq!(model.base.players, expected.players);
    }

    #[test]
    fn fixed_type_mismatch() {
        assert!(matches!(
            build!(MDP, .with_dtmc_model_type()),
            Err(ModelBuildingError::ModelTypeMismatch {
                requested: BaseModelType::Dtmc,
                narrowest: BaseModelType::Mdp
            })
        ));
        assert!(matches!(
            build!(DTMC, .with_transition_system_model_type()),
            Err(ModelBuildingError::ModelTypeMismatch {
                requested: BaseModelType::TransitionSystem,
                narrowest: BaseModelType::Dtmc
            })
        ));
        assert!(matches!(
            build!(STOCHASTIC_GAME, .with_nonstochastic_game_model_type()),
            Err(ModelBuildingError::ModelTypeMismatch {
                requested: BaseModelType::NonstochasticGame,
                narrowest: BaseModelType::StochasticGame
            })
        ));
    }

    #[test]
    fn fixed_type_ignores_declared_type() {
        let model = build!(&format!("mdp {DTMC}"), .with_dtmc_model_type()).unwrap();
        let _: &Dtmc<Ix> = &model.base;
        assert_eq!(transitions(&model), transitions(&expected_dtmc()));
    }

    #[test]
    fn fixed_non_game_with_players() {
        assert!(matches!(
            build!(STOCHASTIC_GAME, .with_mdp_model_type()),
            Err(ModelBuildingError::ModelHasPlayers)
        ));
    }

    #[test]
    fn fixed_game_without_players() {
        assert!(matches!(
            build!(MDP, .with_stochastic_game_model_type()),
            Err(ModelBuildingError::ModelHasNoPlayers)
        ));
    }
}

mod auto {
    use super::*;

    #[test]
    fn auto_uses_declared_type() {
        let model = build!(&format!("dtmc {DTMC}")).unwrap();
        let _: &AnyBaseModel<Ix> = &model.base;
        let model = model.base_into_dtmc().unwrap();
        assert_eq!(transitions(&model), transitions(&expected_dtmc()));

        let model = build!(&format!("mdp {DTMC}")).unwrap();
        let model = model.base_into_mdp().unwrap();
        assert_eq!(transitions(&model), transitions(&expected_dtmc()));
    }

    #[test]
    fn auto_without_declared_type_narrows() {
        let _: &AnyBaseModel<Ix> = &build!(TS).unwrap().base;
        assert!(build!(TS).unwrap().base_is_transition_system());
        assert!(build!(DTMC).unwrap().base_is_dtmc());
        assert!(build!(MDP).unwrap().base_is_mdp());
        assert!(
            build!(NONSTOCHASTIC_GAME)
                .unwrap()
                .base_is_nonstochastic_game()
        );
        assert!(build!(STOCHASTIC_GAME).unwrap().base_is_stochastic_game());
    }

    #[test]
    fn auto_declared_type_mismatch() {
        assert!(matches!(
            build!(&format!("dtmc {MDP}")),
            Err(ModelBuildingError::ModelTypeMismatch {
                requested: BaseModelType::Dtmc,
                narrowest: BaseModelType::Mdp
            })
        ));
    }

    #[test]
    fn auto_declared_mdp_with_players() {
        // TODO: This error could be more helpful (the model has players, but the type is no game)
        assert!(matches!(
            build!(&format!("mdp {STOCHASTIC_GAME}")),
            Err(ModelBuildingError::ModelTypeMismatch {
                requested: BaseModelType::Mdp,
                narrowest: BaseModelType::StochasticGame
            })
        ));
    }

    #[test]
    fn auto_declared_smg_without_players() {
        assert!(matches!(
            build!(&format!("smg {MDP}")),
            Err(ModelBuildingError::ModelTypeMismatch {
                requested: BaseModelType::StochasticGame,
                narrowest: BaseModelType::Mdp
            })
        ));
    }
}

mod narrowest {
    use super::*;

    #[test]
    fn narrowest_ignores_declared_type() {
        let model = build!(&format!("mdp {DTMC}"), .with_narrowest_model_type()).unwrap();
        let _: &AnyBaseModel<Ix> = &model.base;
        let model = model.base_into_dtmc().unwrap();
        assert_eq!(transitions(&model), transitions(&expected_dtmc()));
    }

    #[test]
    fn ts_takes_priority_over_dtmc() {
        let model = build!(TS, .with_narrowest_model_type()).unwrap();
        let model = model.base_into_transition_system().unwrap();
        assert_eq!(transitions(&model), transitions(&expected_ts()));
    }
}

#[test]
fn indices_preserved_by_narrowing() {
    use probabilistic_models::traits::ReadRewards;
    const SOURCE: &str = r#"
        module m x: [0..2] init 0; [] x<2 -> 0.5: (x'=x+1) + 0.5: true; endmodule
        label "odd" = x=1;
        rewards "r" x=1: 2; x=2: 3; endrewards"#;
    fn state_rewards<M: ReadStateSpace + ReadRewards>(model: &M) -> Vec<f64> {
        let r = model.reward_structure(None).unwrap();
        model
            .states()
            .into_iter()
            .map(|s| model.state_reward(r, s))
            .collect()
    }
    fn ap_sets<M: ReadAtomicPropositions + ReadValuations>(
        model: &M,
    ) -> Vec<(String, Vec<String>)> {
        let names = model
            .atomic_propositions()
            .map(|(_, name)| name.to_string());
        names
            .map(|name| (name.clone(), states_with(model, &name)))
            .collect()
    }
    let mdp = build!(SOURCE, .with_mdp_model_type().with_all_labels()).unwrap();
    let dtmc = build!(SOURCE, .with_dtmc_model_type().with_all_labels()).unwrap();
    let _: &Dtmc<Ix> = &dtmc.base;
    assert_eq!(transitions(&mdp), transitions(&dtmc));
    assert_eq!(state_rewards(&mdp), state_rewards(&dtmc));
    assert_eq!(state_rewards(&dtmc), [0.0, 2.0, 3.0]);
    assert_eq!(ap_sets(&mdp), ap_sets(&dtmc));
    assert_eq!(states_with(&dtmc, "odd"), ["x=1"]);
    assert_eq!(mdp.initial, dtmc.initial);
}
