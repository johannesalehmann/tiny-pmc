use crate::parsing::Inputs;
use crate::{CheckerOptions, OutputReturner, ProcessingOptions, StateValue};

#[test]
fn test_umb() {
    let inputs = Inputs::from_cli_args(&[
        "src/tests/files/umb/mdp-gzip.umb".to_string(),
        r#"Pmax=? [F "g"]; Pmin=? [F "g"]; "#.to_string(),
    ])
    .unwrap();
    let results = crate::build_and_check_model(
        inputs,
        [],
        &ProcessingOptions::default(),
        &CheckerOptions::default(),
        OutputReturner::new(),
    )
    .unwrap();
    // TODO: Make an epsilon comparison here, as value iteration is not guaranteed to always be exact
    let values = results
        .results
        .iter()
        .flat_map(|result| result.iter().map(|(_, value)| value))
        .collect::<Vec<_>>();
    assert_eq!(values, [StateValue::Float(0.8), StateValue::Float(0.0)]);
}
