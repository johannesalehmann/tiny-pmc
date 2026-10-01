use crate::{Error, ErrorSource, ValidationError};
use prism_model::Displayable;
use probabilistic_properties::{PathFormula, Query, StateFormula};

/// Asserts that `$query` (a `NamedQuery`) has the given name and that its query matches `$pattern`.
macro_rules! assert_query {
    ($query: expr, $name: expr, $pattern: pat) => {
        assert_eq!($query.name.as_deref(), $name);
        assert!(matches!($query.query, $pattern));
    };
}

fn duplicate_name(name: &str, previous_index: Option<usize>) -> Error<'static> {
    ValidationError::DuplicateQueryName {
        name: name.to_string(),
        previous_index,
    }
    .into()
}

const MODEL: &str = r#"
dtmc

label "goal" = x=3;

module main
    x: [0..5] init 0;
    [] x<5 -> (x'=x+1);
endmodule"#;

#[test]
fn two_unnamed_in_one_file() {
    let properties = &["P=? [F x=5]; R=? [F x=6];"];
    let parse = crate::parse_unprocessed_props(properties);

    let file_0 = parse.result_for_input_file(0).unwrap();
    assert_eq!(file_0.len(), 2);
    assert_query!(file_0[0], None, Query::ProbabilityValue { .. });
    assert_query!(file_0[1], None, Query::RewardValue { .. });

    let Ok(queries) = parse.all_ok() else {
        panic!()
    };
    assert_eq!(queries.len(), 2);
    assert_query!(queries[0], None, Query::ProbabilityValue { .. });
    assert_query!(queries[1], None, Query::RewardValue { .. });
}

#[test]
fn three_in_one_file() {
    let properties = &["P=? [F x=5]; \"p2\": R>=1 [F x=6]; R=? [F x=7];"];
    let parse = crate::parse_unprocessed_props(properties);

    let file_0 = parse.result_for_input_file(0).unwrap();
    assert_eq!(file_0.len(), 3);
    assert_query!(file_0[0], None, Query::ProbabilityValue { .. });
    assert_query!(file_0[1], Some("p2"), Query::RewardBound { .. });
    assert_query!(file_0[2], None, Query::RewardValue { .. });

    let Ok(queries) = parse.all_ok() else {
        panic!()
    };
    assert_eq!(queries.len(), 3);
    assert_query!(queries[0], None, Query::ProbabilityValue { .. });
    assert_query!(queries[1], Some("p2"), Query::RewardBound { .. });
    assert_query!(queries[2], None, Query::RewardValue { .. });
}

#[test]
fn three_in_two_files() {
    let properties = &["P=? [F x=5];", "\"p2\": R>=1 [F x=6]; R=? [F x=7];"];
    let parse = crate::parse_unprocessed_props(properties);

    let file_0 = parse.result_for_input_file(0).unwrap();
    assert_eq!(file_0.len(), 1);
    assert_query!(file_0[0], None, Query::ProbabilityValue { .. });
    let file_1 = parse.result_for_input_file(1).unwrap();
    assert_eq!(file_1.len(), 2);
    assert_query!(file_1[0], Some("p2"), Query::RewardBound { .. });
    assert_query!(file_1[1], None, Query::RewardValue { .. });

    let Ok(queries) = parse.all_ok() else {
        panic!()
    };
    assert_eq!(queries.len(), 3);
    assert_query!(queries[0], None, Query::ProbabilityValue { .. });
    assert_query!(queries[1], Some("p2"), Query::RewardBound { .. });
    assert_query!(queries[2], None, Query::RewardValue { .. });
}
#[test]
fn some_files_empty() {
    let properties = &[
        "P=? [F x=5];",
        "",
        "",
        "\"p2\": R>=1 [F x=6]; R=? [F x=7];",
        "",
    ];
    let parse = crate::parse_unprocessed_props(properties);

    let file_0 = parse.result_for_input_file(0).unwrap();
    assert_eq!(file_0.len(), 1);
    assert_query!(file_0[0], None, Query::ProbabilityValue { .. });
    let file_3 = parse.result_for_input_file(3).unwrap();
    assert_eq!(file_3.len(), 2);
    assert_query!(file_3[0], Some("p2"), Query::RewardBound { .. });
    assert_query!(file_3[1], None, Query::RewardValue { .. });

    for i in [1, 2, 4] {
        assert!(parse.result_for_input_file(i).unwrap().is_empty());
    }

    let Ok(queries) = parse.all_ok() else {
        panic!()
    };
    assert_eq!(queries.len(), 3);
    assert_query!(queries[0], None, Query::ProbabilityValue { .. });
    assert_query!(queries[1], Some("p2"), Query::RewardBound { .. });
    assert_query!(queries[2], None, Query::RewardValue { .. });
}
#[test]
fn some_files_malformed() {
    let properties = &[
        "P=? [F x=5];",
        "Ceci n’est pas une propriété",
        "",
        "\"p2\": R>=1 [F x=6]; R=? [F x=7];",
        "And neither is this",
    ];
    let parse = crate::parse_unprocessed_props(properties);

    let file_0 = parse.result_for_input_file(0).unwrap();
    assert_eq!(file_0.len(), 1);
    assert_query!(file_0[0], None, Query::ProbabilityValue { .. });
    for i in [1, 4] {
        let Err(e) = parse.result_for_input_file(i) else {
            panic!()
        };
        matches!(e, &[Error::ExpectedFound { .. }]);
    }
    let file_3 = parse.result_for_input_file(3).unwrap();
    assert_eq!(file_3.len(), 2);
    assert_query!(file_3[0], Some("p2"), Query::RewardBound { .. });
    assert_query!(file_3[1], None, Query::RewardValue { .. });

    assert!(parse.result_for_input_file(2).unwrap().is_empty());

    let Err(errs) = parse.all_ok() else { panic!() };
    assert_eq!(errs.len(), 2);
    for (i, property_file_index) in [(0, 1), (1, 4)] {
        assert_eq!(
            errs[i].source,
            ErrorSource::Property {
                property_file_index
            }
        );
        matches!(errs[i].error, Error::ExpectedFound { .. });
    }
}

#[test]
fn single_with_and_without_semicolon() {
    for source in ["P=? [F x=5];", "P=? [F x=5]"] {
        let properties = &[source];
        let parse = crate::parse_unprocessed_props(properties);

        let file_0 = parse.result_for_input_file(0).unwrap();
        assert_eq!(file_0.len(), 1);
        assert_query!(file_0[0], None, Query::ProbabilityValue { .. });

        let Ok(queries) = parse.all_ok() else {
            panic!()
        };
        assert_eq!(queries.len(), 1);
        assert_query!(queries[0], None, Query::ProbabilityValue { .. });
    }
}

#[test]
fn duplicate_name_in_one_file() {
    let properties = &["\"a\": P=? [F x=5]; \"a\": R=? [F x=6];"];
    let parse = crate::parse_unprocessed_props(properties);

    let Err(errors_0) = parse.result_for_input_file(0) else {
        panic!()
    };
    assert_eq!(errors_0, &[duplicate_name("a", Some(0))]);
    // The first occurrence is kept, the duplicate is dropped
    assert_eq!(parse.properties.len(), 1);
    assert_query!(
        parse.properties[0],
        Some("a"),
        Query::ProbabilityValue { .. }
    );

    let Err(errors) = parse.all_ok() else {
        panic!()
    };
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].source,
        ErrorSource::Property {
            property_file_index: 0
        }
    ));
    assert_eq!(errors[0].error, duplicate_name("a", Some(0)));
}

#[test]
fn two_duplicate_names_in_one_file() {
    let properties =
        &["\"a\": P=? [F x=5]; \"b\": R=? [F x=6]; \"b\": P=? [F x=7]; \"a\": R=? [F x=8];"];
    let parse = crate::parse_unprocessed_props(properties);

    let Err(errors_0) = parse.result_for_input_file(0) else {
        panic!()
    };
    assert_eq!(
        errors_0,
        &[duplicate_name("b", Some(1)), duplicate_name("a", Some(0))]
    );
    assert_eq!(parse.properties.len(), 2);
    assert_query!(
        parse.properties[0],
        Some("a"),
        Query::ProbabilityValue { .. }
    );
    assert_query!(parse.properties[1], Some("b"), Query::RewardValue { .. });

    let Err(errors) = parse.all_ok() else {
        panic!()
    };
    assert_eq!(errors.len(), 2);
    assert!(matches!(
        errors[0].source,
        ErrorSource::Property {
            property_file_index: 0
        }
    ));
    assert_eq!(errors[0].error, duplicate_name("b", Some(1)));
    assert!(matches!(
        errors[1].source,
        ErrorSource::Property {
            property_file_index: 0
        }
    ));
    assert_eq!(errors[1].error, duplicate_name("a", Some(0)));
}

#[test]
fn duplicate_name_in_two_files() {
    let properties = &[
        "P=? [F x=5]; \"a\": P=? [F x=6];",
        "\"b\": R=? [F x=7]; \"a\": R=? [F x=8];",
    ];
    let parse = crate::parse_unprocessed_props(properties);

    let file_0 = parse.result_for_input_file(0).unwrap();
    assert_eq!(file_0.len(), 2);
    assert_query!(file_0[0], None, Query::ProbabilityValue { .. });
    assert_query!(file_0[1], Some("a"), Query::ProbabilityValue { .. });
    let Err(errors_1) = parse.result_for_input_file(1) else {
        panic!()
    };
    assert_eq!(errors_1, &[duplicate_name("a", Some(1))]);
    assert_eq!(parse.properties.len(), 3);
    assert_query!(parse.properties[2], Some("b"), Query::RewardValue { .. });

    let Err(errors) = parse.all_ok() else {
        panic!()
    };
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].source,
        ErrorSource::Property {
            property_file_index: 1
        }
    ));
    assert_eq!(errors[0].error, duplicate_name("a", Some(1)));
}

#[test]
fn labels_are_substituted() {
    let properties = &["P=? [F x=2]", "P=? [F \"goal\"]"];
    let parse = crate::parse_model_and_props(MODEL, properties);
    let model = parse.model.as_ref().unwrap();

    // Returns the condition of a query `P=? [F condition]` as a string
    let condition = |query: &crate::Query| {
        let Query::ProbabilityValue {
            path: PathFormula::Eventually { condition },
            ..
        } = &query.query
        else {
            panic!()
        };
        let StateFormula::Expression(condition) = condition.as_ref() else {
            panic!()
        };
        condition.displayable(&model.variable_manager).to_string()
    };

    let file_0 = parse.properties.result_for_input_file(0).unwrap();
    assert_eq!(file_0.len(), 1);
    assert_eq!(condition(&file_0[0]), "x=2");
    let file_1 = parse.properties.result_for_input_file(1).unwrap();
    assert_eq!(file_1.len(), 1);
    assert_eq!(condition(&file_1[0]), "x=3");
}

#[test]
fn empty_files_with_processing() {
    let properties = &["P=? [F x=2]", ""];
    let parse = crate::parse_model_and_props(MODEL, properties);
    assert_eq!(parse.properties.result_for_input_file(0).unwrap().len(), 1);
    assert_eq!(parse.properties.result_for_input_file(1).unwrap().len(), 0);
}

#[test]
fn duplicate_names_with_model() {
    let properties = &[
        // `y` does not exist in the model, so "bad" is dropped during processing
        "\"bad\": P=? [F y=1]; \"good\": P=? [F x=1];",
        "\"good\": P=? [F x=2]; \"bad\": P=? [F x=3]; \"ok\": R=? [F x=4];",
    ];
    let parse = crate::parse_model_and_props(MODEL, properties);
    assert!(parse.model.is_ok());
    let props = &parse.properties;

    // Only "good" (from the first file) and "ok" remain
    assert_eq!(props.properties.len(), 2);
    assert_query!(
        props.properties[0],
        Some("good"),
        Query::ProbabilityValue { .. }
    );
    assert_query!(props.properties[1], Some("ok"), Query::RewardValue { .. });
    assert_eq!(props.input_source_to_properties, vec![0..1, 1..2]);

    let Err(errors_0) = props.result_for_input_file(0) else {
        panic!()
    };
    assert_eq!(errors_0.len(), 1);
    assert!(matches!(
        &errors_0[0],
        Error::Validation(ValidationError::UnknownVariable { identifier }) if identifier.name == "y"
    ));

    // "good" was moved from index 1 to index 0, "bad" no longer exists
    let Err(errors_1) = props.result_for_input_file(1) else {
        panic!()
    };
    assert_eq!(
        errors_1,
        &[duplicate_name("good", Some(0)), duplicate_name("bad", None)]
    );
}
