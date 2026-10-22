use super::*;
use crate::{ModelBuilder, ModelBuildingError};
use prism_model::Expression;
use probabilistic_models::base_model::StochasticGame;
use probabilistic_models::smg;

/// Player `p1` moves if `x=y`, player `p2` otherwise.
const GAME: &str = r#"
    smg
    player p1 m1 endplayer
    player p2 m2 endplayer
    module m1 x: [0..1] init 0; [] x=y -> 0.5: (x'=1-x) + 0.5: true; endmodule
    module m2 y: [0..1] init 0; [] x!=y -> (y'=x); endmodule"#;

#[test]
fn owners_by_module() {
    let model = build!(GAME, .with_stochastic_game_model_type()).unwrap();
    let _: &StochasticGame<Ix> = &model.base;
    let order = ["x=0,y=0", "x=1,y=0", "x=1,y=1", "x=0,y=1"];
    assert_eq!(valuations(&model), order);
    smg!(expected = {
        s00 (Eve) -> 0.5: s10 & 0.5: s00,
        s10 (Adam) -> 1.0: s11,
        s11 (Eve) -> 0.5: s01 & 0.5: s11,
        s01 (Adam) -> 1.0: s00,
    });
    assert_eq!(transitions(&model), transitions(&expected));
    assert_eq!(model.base.players, expected.players);
}

#[test]
fn owners_by_action() {
    const SOURCE: &str = r#"
        smg
        player p1 [a] endplayer
        player p2 [b] endplayer
        module m1 x: [0..1] init 0; [a] x=0 -> 0.5: (x'=1) + 0.5: true; [b] x=1 -> (x'=0); endmodule
        module m2 y: bool init false; [a] true -> true; [b] true -> true; endmodule"#;
    let model = build!(SOURCE, .with_stochastic_game_model_type()).unwrap();
    let _: &StochasticGame<Ix> = &model.base;
    smg!(expected = {
        s0 (Eve) -> 0.5: s1 & 0.5: s0,
        s1 (Adam) -> 1.0: s0,
    });
    assert_eq!(transitions(&model), transitions(&expected));
    assert_eq!(model.base.players, expected.players);
}

#[test]
fn action_internal_to_owned_module() {
    const SOURCE: &str = r#"
        smg
        player p1 m1 endplayer
        player p2 m2 endplayer
        module m1 x: [0..1] init 0; [go] x=y -> (x'=1-x); endmodule
        module m2 y: [0..1] init 0; [back] x!=y -> (y'=x); endmodule"#;
    let model = build!(SOURCE, .with_stochastic_game_model_type()).unwrap();
    let _: &StochasticGame<Ix> = &model.base;
    smg!(expected = {
        s00 (Eve) -> 1.0: s10,
        s10 (Adam) -> 1.0: s11,
        s11 (Eve) -> 1.0: s01,
        s01 (Adam) -> 1.0: s00,
    });
    assert_eq!(transitions(&model), transitions(&expected));
    assert_eq!(model.base.players, expected.players);
}

#[test]
fn deadlock_state_owned_by_first_player() {
    const SOURCE: &str = r#"
        smg
        player p1 m1 endplayer
        player p2 m2 endplayer
        module m1 x: [0..1] init 0; [] x=0 -> (x'=1); endmodule
        module m2 y: [0..1] init 0; [] x=1 & y=0 -> (y'=1); endmodule"#;
    let model = build!(SOURCE, .with_stochastic_game_model_type()).unwrap();
    let _: &StochasticGame<Ix> = &model.base;
    smg!(expected = {
        s0 (Eve) -> 1.0: s1,
        s1 (Adam) -> 1.0: s2,
        s2 (Eve) -> 1.0: s2,
    });
    assert_eq!(transitions(&model), transitions(&expected));
    assert_eq!(model.base.players, expected.players);
}

#[test]
fn state_space_restriction_with_game() {
    let mut prism = prism_parser::parse_model(GAME).unwrap();
    let x = prism.variable_manager.get_reference_by_str("x").unwrap();
    let restriction = Expression::var_or_const(x).less_than(Expression::int(1));
    let model = ModelBuilder::new(&mut prism)
        .with_stochastic_game_model_type()
        .with_state_space_restriction(restriction)
        .build()
        .unwrap();
    // `x=1,y=0` is excluded, so it is owned by the first player instead of `p2`
    smg!(expected = {
        s00 (Eve) -> 0.5: s10 & 0.5: s00,
        s10 (Eve) -> 1.0: s10,
    });
    assert_eq!(transitions(&model), transitions(&expected));
    assert_eq!(model.base.players, expected.players);
}

mod errors {
    use super::*;

    macro_rules! error_test {
        ($name:ident, $error:ident, $source:expr) => {
            #[test]
            fn $name() {
                let result = build!(concat!("smg ", $source));
                assert!(matches!(result, Err(ModelBuildingError::$error { .. })));
            }
        };
    }

    error_test!(
        unknown_player_module,
        UnknownPlayerModule,
        "player p1 m2 endplayer module m1 x: [0..1] init 0; [] x=0 -> (x'=1); endmodule"
    );
    error_test!(
        unknown_player_action,
        UnknownPlayerAction,
        "player p1 [b] endplayer module m1 x: [0..1] init 0; [a] x=0 -> (x'=1); endmodule"
    );
    error_test!(
        module_controlled_by_multiple_players,
        ModuleControlledByMultiplePlayers,
        "player p1 m1 endplayer player p2 m1 endplayer
         module m1 x: [0..1] init 0; [] x=0 -> (x'=1); endmodule"
    );
    error_test!(
        action_controlled_by_multiple_players,
        ActionControlledByMultiplePlayers,
        "player p1 [a] endplayer player p2 [a] endplayer
         module m1 x: [0..1] init 0; [a] x=0 -> (x'=1); endmodule"
    );
    error_test!(
        state_controlled_by_multiple_players,
        StateControlledByMultiplePlayers,
        "player p1 m1 endplayer player p2 m2 endplayer
         module m1 x: [0..1] init 0; [] x=0 -> (x'=1); endmodule
         module m2 y: [0..1] init 0; [] y=0 -> (y'=1); endmodule"
    );
    error_test!(
        state_without_owner,
        StateWithoutOwner,
        "player p1 m1 endplayer
         module m1 x: [0..1] init 0; [] x=1 -> (x'=0); endmodule
         module m2 y: [0..1] init 0; [] y=0 -> (y'=1); endmodule"
    );
}
