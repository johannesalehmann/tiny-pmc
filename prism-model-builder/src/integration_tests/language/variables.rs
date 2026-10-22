use super::*;
use crate::expressions::VariableType;

#[test]
fn variable_types() {
    let model = build!(
        "mdp module m b: bool init true; z: [-2..1] init -1; u: int init 0; [] u<1 -> (b'=!b) & (z'=z+1) & (u'=u+1); endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    assert_eq!(valuations(&model), ["b=true,z=-1,u=0", "b=false,z=0,u=1"]);
}

#[test]
fn default_initial_values() {
    let model = build!(
        "mdp module m a: [-3..2]; b: [0..2]; c: [2..5]; d: bool; [] false -> true; endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    assert_eq!(valuations(&model), ["a=-3,b=0,c=2,d=false"]);

    let model = build!(
        "mdp module m a: [-3..2] init -1; b: [0..2] init 1; c: [2..5] init 4; d: bool init true; [] false -> true; endmodule",
        .with_mdp_model_type()
    )
    .unwrap();
    assert_eq!(valuations(&model), ["a=-1,b=1,c=4,d=true"]);
}

#[test]
fn build_and_return_variable_info() {
    let mut prism = prism_parser::parse_model(
        "mdp const int N = 2; module m x: [-1..N] init 0; b: bool init false; [] x<N -> (x'=x+1); endmodule",
    )
    .unwrap();
    let (model, info) = crate::ModelBuilder::new(&mut prism)
        .with_mdp_model_type()
        .build_and_return_variable_info()
        .unwrap();
    assert_eq!(
        valuations(&model),
        ["x=0,b=false", "x=1,b=false", "x=2,b=false"]
    );
    assert_eq!(info.details.details.len(), 2);

    let variables = &prism.variable_manager;
    let x = info
        .valuation_map
        .map_to_variable(variables.get_reference_by_str("x").unwrap().index)
        .unwrap();
    assert_eq!(info.details[x].bounds, Some((-1, 2)));
    assert!(matches!(info.details[x].variable_type, VariableType::Int));

    let b = info
        .valuation_map
        .map_to_variable(variables.get_reference_by_str("b").unwrap().index)
        .unwrap();
    assert!(matches!(info.details[b].variable_type, VariableType::Bool));

    let n = variables.get_reference_by_str("N").unwrap();
    assert!(info.valuation_map.map_to_variable(n.index).is_none());
    assert!(matches!(
        info.value_of_const(n),
        Some(crate::variables::ConstValuation::Int(2))
    ));
}
