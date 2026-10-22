use super::*;

#[test]
fn empty_update_and_true() {
    let model = build!(
        "mdp module m x: [0..1] init 0; [] x=0 -> true; [] x=0 -> (x'=x); [] x=0 -> (x'=1); endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    assert_eq!(valuations(&model), ["x=0", "x=1"]);
    mdp!(expected = {
        x0 -> 1.0: x0,
        x0 -> 1.0: x0,
        x0 -> 1.0: x1,
        x1 -> 1.0: x1,
    });
    assert_eq!(transitions(&model), transitions(&expected));
}

#[test]
fn multiple_assignments_per_update() {
    let model = build!(
        "mdp module m x: [0..1] init 0; y: [0..1] init 1; [] true -> (x'=y) & (y'=x); endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    assert_eq!(valuations(&model), ["x=0,y=1", "x=1,y=0"]);
    mdp!(expected = { s01 -> 1.0: s10, s10 -> 1.0: s01 });
    assert_eq!(transitions(&model), transitions(&expected));
}

#[test]
fn expressions_in_guards_and_updates() {
    const SOURCE: &str = r"
        mdp
        module m
            x: [0..30] init 0;
            b: bool init false;
            [] x=0 & (b ? false : 1.5 < 2) ->
                (x'=(true ? max(3, 2) : 0) + min(4, 5) + floor(2.5) + ceil(2.5) + pow(2, 3) + mod(7, 3))
                & (b'=(false => false) & (true <=> !false) & (1.5 + 1 > 2));
        endmodule
    ";
    let model = build!(SOURCE, .with_mdp_model_type()).unwrap();
    // 3 + 4 + 2 + 3 + 8 + 1 = 21
    assert_eq!(valuations(&model), ["x=0,b=false", "x=21,b=true"]);
}

#[test]
fn probability_expressions() {
    let model = build!(
        "mdp const double p = 0.5; module m x: [0..2] init 0; [] x=0 -> p/2: (x'=1) + p/2: (x'=1) + 1-p: (x'=2); endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    assert_eq!(valuations(&model), ["x=0", "x=1", "x=2"]);
    // The two branches to `x=1` are not merged.
    mdp!(expected = {
        x0 -> 0.25: x1 & 0.25: x1 & 0.5: x2,
        x1 -> 1.0: x1,
        x2 -> 1.0: x2,
    });
    assert_eq!(transitions(&model), transitions(&expected));
}

#[test]
#[should_panic(expected = "exceeds variable bounds")]
fn update_out_of_bounds_panics() {
    let _ = build!(
        "mdp module m x: [0..1] init 0; [] true -> (x'=x+1); endmodule",
        .with_mdp_model_type()
    );
}
