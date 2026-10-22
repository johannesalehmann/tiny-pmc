use super::*;
use crate::ModelBuilder;
use prism_model::Expression;
use probabilistic_models::base_model::Mdp;
use probabilistic_models::initial_states::{InitialStates, SingleInitialState};
use probabilistic_models::mdp;
use probabilistic_models::traits::ReadInitialStates;
use typed_index_collections::Index;

const COUNTDOWN: &str = "mdp module m x: [0..2] init 1; [] x>0 -> (x'=x-1); endmodule";

#[test]
fn single_initial_state() {
    let model = build!(COUNTDOWN, .with_mdp_model_type()).unwrap();
    let _: &SingleInitialState<Ix> = &model.initial;
    assert_eq!(valuations(&model), ["x=1", "x=0"]);
    assert_eq!(model.initial.index, StateIndex::from_raw(0));
    assert!(model.is_initial(StateIndex::from_raw(0)));
    assert!(!model.is_initial(StateIndex::from_raw(1)));
}

#[test]
fn initial_state_vector() {
    let model = build!(COUNTDOWN, .with_mdp_model_type().with_initial_state_vector()).unwrap();
    let _: &InitialStates<Ix> = &model.initial;
    assert_eq!(valuations(&model), ["x=1", "x=0"]);
    assert_eq!(model.initial.entries(), [true, false]);
}

#[test]
fn without_initial_states() {
    let model = build!(COUNTDOWN, .with_mdp_model_type().without_initial_states()).unwrap();
    let _: &() = &model.initial;
    assert_eq!(valuations(&model), ["x=1", "x=0"]);
}

mod full_state_space {
    use super::*;

    const IMPLICIT_X_INIT: &str =
        "mdp module m x: [-1..0]; y: [1..2] init 2; [] x<0 -> (x'=x+1); endmodule";
    const IMPLICIT_Y_INIT: &str =
        "mdp module m x: [-1..0] init 0; y: [1..2]; [] x<0 -> (x'=x+1); endmodule";

    fn expected() -> Mdp<Ix> {
        mdp!(expected = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s1,
            s2 -> 1.0: s3,
            s3 -> 1.0: s3,
        });
        expected
    }

    #[test]
    fn full_state_space() {
        let model =
            build!(IMPLICIT_X_INIT, .with_mdp_model_type().with_full_state_space()).unwrap();
        let _: &SingleInitialState<Ix> = &model.initial;
        assert_eq!(
            valuations(&model),
            ["x=-1,y=1", "x=0,y=1", "x=-1,y=2", "x=0,y=2"]
        );
        assert_eq!(transitions(&model), transitions(&expected()));
        assert_eq!(model.initial.index, state(&model, "x=-1,y=2"));
    }

    #[test]
    fn full_state_space_with_initial_state_vector() {
        let model = build!(
            IMPLICIT_Y_INIT,
            .with_mdp_model_type().with_full_state_space().with_initial_state_vector()
        )
        .unwrap();
        let _: &InitialStates<Ix> = &model.initial;
        assert_eq!(
            valuations(&model),
            ["x=-1,y=1", "x=0,y=1", "x=-1,y=2", "x=0,y=2"]
        );
        assert_eq!(transitions(&model), transitions(&expected()));
        assert_eq!(model.initial.entries(), [false, true, false, false]);
    }

    #[test]
    fn full_state_space_round_trip() {
        let round_trip = build!(
            IMPLICIT_X_INIT,
            .with_mdp_model_type().with_full_state_space().with_reachable_state_space()
        )
        .unwrap();
        let default = build!(IMPLICIT_X_INIT, .with_mdp_model_type()).unwrap();
        assert_eq!(valuations(&round_trip), valuations(&default));
        assert_eq!(transitions(&round_trip), transitions(&default));
        assert_eq!(round_trip.initial.index, default.initial.index);
    }

    #[test]
    #[should_panic(
        expected = "Cannot build entire model state space if the model contains unbounded variables"
    )]
    fn full_state_space_unbounded_int_panics() {
        let _ = build!(
            "mdp module m x: int init 0; [] x<2 -> (x'=x+1); endmodule",
            .with_mdp_model_type().with_full_state_space()
        );
    }
}

#[test]
fn state_space_restriction_keeps_frontier() {
    let mut prism =
        prism_parser::parse_model("mdp module m x: [0..4] init 0; [] x<4 -> (x'=x+1); endmodule")
            .unwrap();
    let x = prism.variable_manager.get_reference_by_str("x").unwrap();
    let restriction = Expression::var_or_const(x).less_than(Expression::int(2));
    let model = ModelBuilder::new(&mut prism)
        .with_mdp_model_type()
        .with_state_space_restriction(restriction)
        .build()
        .unwrap();
    mdp!(expected = {
        s0 -> 1.0: s1,
        s1 -> 1.0: s2,
        s2 -> 1.0: s2,
    });
    assert_eq!(valuations(&model), ["x=0", "x=1", "x=2"]);
    assert_eq!(transitions(&model), transitions(&expected));
}

fn state_space_restriction_excludes_initial_state() {
    let mut prism =
        prism_parser::parse_model("mdp module m x: [0..4] init 0; [] x<4 -> (x'=x+1); endmodule")
            .unwrap();
    let x = prism.variable_manager.get_reference_by_str("x").unwrap();
    let restriction = Expression::var_or_const(x).greater_or_equal(Expression::int(1));
    let model = ModelBuilder::new(&mut prism)
        .with_mdp_model_type()
        .with_state_space_restriction(restriction)
        .build()
        .unwrap();
    mdp!(expected = {
        s0 -> 1.0: s0,
    });
    assert_eq!(valuations(&model), ["x=0"]);
    assert_eq!(transitions(&model), transitions(&expected));
}
