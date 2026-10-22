use super::queries::{parse, show};
use super::*;
use crate::ModelBuilder;
use probabilistic_models::annotations::AtomicPropositions;
use std::collections::HashSet;

const MODEL: &str = r#"
mdp
const int N = 2;
formula finished = x=N;
module m
    x: [0..N] init 0;
    [] x<N -> 0.5: (x'=x+1) + 0.5: true;
endmodule
label "start" = x=0;
label "done" = finished;
label "small" = x<N;
"#;

/// Names of all atomic propositions of `model`, in index order.
pub(super) fn ap_names<M: ReadAtomicPropositions>(model: &M) -> Vec<String> {
    model
        .atomic_propositions()
        .map(|(_, name)| name.to_string())
        .collect()
}

fn names(names: &[&str]) -> HashSet<String> {
    names.iter().map(|name| name.to_string()).collect()
}

#[test]
fn default_builds_no_labels() {
    let model = build!(MODEL, .with_mdp_model_type()).unwrap();
    let _: &AtomicPropositions<Ix> = &model.atomic_propositions;
    assert!(ap_names(&model).is_empty());
}

#[test]
fn with_all_labels() {
    let model = build!(MODEL, .with_mdp_model_type().with_all_labels()).unwrap();
    assert_eq!(ap_names(&model), ["start", "done", "small"]);
    assert_eq!(states_with(&model, "start"), ["x=0"]);
    assert_eq!(states_with(&model, "done"), ["x=2"]);
    assert_eq!(states_with(&model, "small"), ["x=0", "x=1"]);
}

#[test]
fn with_given_labels() {
    let model = build!(MODEL, .with_mdp_model_type().with_given_labels(names(&["small"]))).unwrap();
    assert_eq!(ap_names(&model), ["small"]);
    assert_eq!(states_with(&model, "small"), ["x=0", "x=1"]);
}

#[test]
fn with_listed_plus_necessary_labels() {
    let (mut prism, mut queries) = parse(MODEL, &[r#"P=? [F "done"]"#]);
    let out = ModelBuilder::new(&mut prism)
        .with_mdp_model_type()
        .with_listed_plus_necessary_labels(names(&["start"]))
        .with_query(queries.pop().unwrap())
        .build()
        .unwrap();
    assert_eq!(ap_names(&out.model), ["start", "done"]);
    assert_eq!(states_with(&out.model, "done"), ["x=2"]);
}

#[test]
fn without_atomic_proposition_vector() {
    let model = build!(MODEL, .with_mdp_model_type().without_atomic_proposition_vector()).unwrap();
    let _: &() = &model.atomic_propositions;
}

#[test]
fn ap_index_order() {
    let (mut prism, mut queries) = parse(MODEL, &["P=? [F x=1]"]);
    let out = ModelBuilder::new(&mut prism)
        .with_mdp_model_type()
        .with_all_labels()
        .with_query(queries.pop().unwrap())
        .build()
        .unwrap();
    assert_eq!(ap_names(&out.model), ["start", "done", "small", "query_ap"]);
    assert_eq!(show(&out.query), "P=? [F 3]");
    assert_eq!(states_with(&out.model, "query_ap"), ["x=1"]);
}
