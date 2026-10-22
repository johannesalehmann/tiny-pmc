use super::*;
use probabilistic_models::base_model::Mdp;
use probabilistic_models::mdp;

mod constants_and_formulas;
mod modules;
mod updates;
mod variables;

#[test]
fn bfs_state_order() {
    let model = build!(
        "mdp module main x: [0..5] init 2; [] (true) -> 0.6: (x'=min(x+1, 5)) + 0.4: (x'=max(x-1, 0)); [] (x=4) -> 1.0: (x'=4); endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    let _: &Mdp<Ix> = &model.base;
    assert_eq!(
        valuations(&model),
        ["x=2", "x=3", "x=1", "x=4", "x=0", "x=5"]
    );
    mdp!(expected = {
        x2 -> 0.6: x3 & 0.4: x1,
        x3 -> 0.6: x4 & 0.4: x2,
        x1 -> 0.6: x2 & 0.4: x0,
        x4 -> 0.6: x5 & 0.4: x3,
        x4 -> 1.0: x4,
        x0 -> 0.6: x1 & 0.4: x0,
        x5 -> 0.6: x5 & 0.4: x4,
    });
    assert_eq!(transitions(&model), transitions(&expected));
}

#[test]
#[should_panic(expected = "Init constraints are not yet supported")]
fn init_endinit_panics() {
    let _ = build!(
        "mdp module m x: [0..1]; [] true -> true; endmodule init x=0 endinit",
        .with_mdp_model_type()
    );
}

#[test]
fn deadlock_fixing() {
    let model = build!(
        "mdp module m x: [0..1] init 0; [] x=0 -> (x'=1); endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    assert_eq!(valuations(&model), ["x=0", "x=1"]);
    mdp!(expected = { x0 -> 1.0: x1, x1 -> 1.0: x1 });
    assert_eq!(transitions(&model), transitions(&expected));
}
