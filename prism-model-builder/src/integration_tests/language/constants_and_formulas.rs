use super::*;
use crate::UserProvidedConstValue;

#[test]
fn constants_in_model() {
    const SOURCE: &str = r"
        mdp
        const int N = 3;
        const int M = N - 1;
        const double p = 0.25;
        const bool B = true;
        module m
            x: [0..N] init M;
            [] B & x<N -> p: (x'=x+1) + 1-p: (x'=0);
        endmodule
    ";
    let model = build!(SOURCE, .with_mdp_model_type()).unwrap();
    assert_eq!(valuations(&model), ["x=2", "x=3", "x=0", "x=1"]);
    mdp!(expected = {
        x2 -> 0.25: x3 & 0.75: x0,
        x3 -> 1.0: x3,
        x0 -> 0.25: x1 & 0.75: x0,
        x1 -> 0.25: x2 & 0.75: x0,
    });
    assert_eq!(transitions(&model), transitions(&expected));
}

const UNDEFINED: &str = r"
    mdp
    const int N;
    const bool B;
    const double p;
    module m
        x: [0..N] init 0;
        [] B & x<N -> p: (x'=x+1) + 1-p: (x'=0);
    endmodule
";

#[test]
fn undefined_constants_from_builder() {
    let model = build!(
        UNDEFINED,
        .with_mdp_model_type()
            .with_int_constant("N".to_string(), 1)
            .with_bool_constant("B".to_string(), true)
            .with_float_constant("p".to_string(), 0.5)
    )
    .unwrap();
    assert_eq!(valuations(&model), ["x=0", "x=1"]);
    mdp!(expected = { x0 -> 0.5: x1 & 0.5: x0, x1 -> 1.0: x1 });
    assert_eq!(transitions(&model), transitions(&expected));

    let model = build!(
        UNDEFINED,
        .with_mdp_model_type().with_constants([
            ("N".to_string(), UserProvidedConstValue::Int(2)),
            ("B".to_string(), UserProvidedConstValue::Bool(false)),
            ("p".to_string(), UserProvidedConstValue::Float(0.5)),
        ])
    )
    .unwrap();
    assert_eq!(valuations(&model), ["x=0"]);
    mdp!(expected = { x0 -> 1.0: x0 });
    assert_eq!(transitions(&model), transitions(&expected));
}

#[test]
#[should_panic(expected = "Incompatible value assigned to constant")]
fn constant_type_mismatch_panics() {
    let _ = build!(
        "mdp const bool B; module m x: [0..1] init 0; [] B -> (x'=1); endmodule",
        .with_mdp_model_type().with_int_constant("B".to_string(), 1)
    );
}

#[test]
fn formulas() {
    const SOURCE: &str = r#"
        mdp
        formula f = x + 1;
        formula g = f * 2;
        label "big" = g > 4;
        module m
            x: [0..3] init 0;
            [] f <= 3 -> (x'=f);
        endmodule
    "#;
    let model = build!(SOURCE, .with_mdp_model_type().with_all_labels()).unwrap();
    assert_eq!(valuations(&model), ["x=0", "x=1", "x=2", "x=3"]);
    mdp!(expected = { x0 -> 1.0: x1, x1 -> 1.0: x2, x2 -> 1.0: x3, x3 -> 1.0: x3 });
    assert_eq!(transitions(&model), transitions(&expected));
    assert_eq!(states_with(&model, "big"), ["x=2", "x=3"]);
}
