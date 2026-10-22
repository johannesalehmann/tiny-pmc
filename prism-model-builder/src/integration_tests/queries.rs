use super::labels::ap_names;
use super::*;
use crate::ModelBuilder;
use crate::queries::{ModelAndQueries, ModelAndQuery, ProcessedQuery, UnprocessedQuery};
use prism_parser::ParserSpan;
use probabilistic_models::AtomicPropositionIndex;
use probabilistic_properties::Query;
use std::collections::HashSet;
use typed_index_collections::Index;

type Ap = AtomicPropositionIndex<Ix>;

const MODEL: &str = r#"
mdp
const int N = 2;
module m
    x: [0..N] init 0;
    [] x<N -> 0.5: (x'=x+1) + 0.5: true;
endmodule
label "goal" = x=N;
label "start" = x=0;
"#;

const A: &str = "P=? [F x=1]";
const B: &str = "Pmax=? [F x=2]";
const C: &str = "Pmin=? [F x=0]";

/// Parses `source` and the properties `props`.
pub(super) fn parse(
    source: &str,
    props: &[&str],
) -> (prism_parser::Model, Vec<UnprocessedQuery<ParserSpan>>) {
    let parsed = prism_parser::parse_model_and_props(source, props)
        .all_ok()
        .unwrap_or_else(|_| panic!("Could not parse model or properties"));
    (parsed.model, parsed.properties.into())
}

/// Displays `query` with atomic propositions shown as their raw index.
pub(super) fn show(query: &ProcessedQuery<Ap>) -> String {
    query.clone().map_e(&mut |ap| ap.raw()).to_string()
}

#[test]
fn with_query() {
    let (mut prism, mut queries) = parse(MODEL, &[r#"P=? [F "start"]"#]);
    let out: ModelAndQuery<_, Ap> = ModelBuilder::new(&mut prism)
        .with_mdp_model_type()
        .with_all_labels()
        .with_query(queries.pop().unwrap())
        .build()
        .unwrap();
    assert_eq!(ap_names(&out.model), ["goal", "start"]);
    assert_eq!(show(&out.query), "P=? [F 1]");
}

mod query_order {
    use super::*;

    const EXPECTED: [&str; 3] = ["P=? [F 0]", "Pmax=? [F 1]", "Pmin=? [F 2]"];

    #[test]
    fn with_queries() {
        let (mut prism, mut queries) = parse(MODEL, &[A, B, C]);
        let c = queries.pop().unwrap();
        let out: ModelAndQueries<_, Ap> = ModelBuilder::new(&mut prism)
            .with_mdp_model_type()
            .with_queries(queries)
            .and_with_query(c)
            .build()
            .unwrap();
        assert_eq!(out.queries.iter().map(show).collect::<Vec<_>>(), EXPECTED);
    }

    #[test]
    fn and_with_query() {
        let (mut prism, queries) = parse(MODEL, &[A, B, C]);
        let [a, b, c]: [_; 3] = queries.try_into().ok().unwrap();
        let out: ModelAndQueries<_, Ap> = ModelBuilder::new(&mut prism)
            .with_mdp_model_type()
            .with_query(a)
            .and_with_query(b)
            .and_with_queries(vec![c])
            .build()
            .unwrap();
        assert_eq!(out.queries.iter().map(show).collect::<Vec<_>>(), EXPECTED);
    }

    #[test]
    fn and_with_queries() {
        let (mut prism, mut queries) = parse(MODEL, &[A, B, C]);
        let a = queries.remove(0);
        let out: ModelAndQueries<_, Ap> = ModelBuilder::new(&mut prism)
            .with_mdp_model_type()
            .with_query(a)
            .and_with_queries(queries)
            .build()
            .unwrap();
        assert_eq!(out.queries.iter().map(show).collect::<Vec<_>>(), EXPECTED);
    }
}

mod query_expression_becomes_ap {
    use super::*;

    #[test]
    fn single_query() {
        let (mut prism, mut queries) = parse(MODEL, &[A]);
        let out = ModelBuilder::new(&mut prism)
            .with_mdp_model_type()
            .with_query(queries.pop().unwrap())
            .build()
            .unwrap();
        assert_eq!(ap_names(&out.model), ["query_ap"]);
        assert_eq!(states_with(&out.model, "query_ap"), ["x=1"]);
    }

    #[test]
    fn query_vector() {
        let (mut prism, queries) = parse(MODEL, &[A, B]);
        let out = ModelBuilder::new(&mut prism)
            .with_mdp_model_type()
            .with_queries(queries)
            .build()
            .unwrap();
        assert_eq!(ap_names(&out.model), ["query_ap_0", "query_ap_1"]);
        assert_eq!(states_with(&out.model, "query_ap_0"), ["x=1"]);
        assert_eq!(states_with(&out.model, "query_ap_1"), ["x=2"]);
    }
}

#[test]
fn query_ap_name_collision() {
    const COLLIDING: &str = r#"mdp module m x: [0..2] init 0; [] x<2 -> (x'=x+1); endmodule
        label "query_ap" = x=0;"#;
    let (mut prism, mut queries) = parse(COLLIDING, &[A]);
    let out = ModelBuilder::new(&mut prism)
        .with_mdp_model_type()
        .with_all_labels()
        .with_query(queries.pop().unwrap())
        .build()
        .unwrap();
    assert_eq!(ap_names(&out.model), ["query_ap", "query_ap_0"]);
    assert_eq!(show(&out.query), "P=? [F 1]");
    assert_eq!(states_with(&out.model, "query_ap_0"), ["x=1"]);
}

#[test]
fn shared_label_between_queries() {
    let (mut prism, queries) = parse(MODEL, &[r#"P=? [F "goal"]"#, r#"Pmax=? [F "goal"]"#]);
    let out = ModelBuilder::new(&mut prism)
        .with_mdp_model_type()
        .with_queries(queries)
        .build()
        .unwrap();
    assert_eq!(ap_names(&out.model), ["goal"]);
    assert_eq!(
        out.queries.iter().map(show).collect::<Vec<_>>(),
        ["P=? [F 0]", "Pmax=? [F 0]"]
    );
}

#[test]
fn query_constants_evaluated() {
    const CONSTS: &str = r#"mdp const int K = 2; const double T = 0.25;
        module m x: [0..2] init 0; [] x<2 -> (x'=x+1); endmodule"#;
    let (mut prism, mut queries) = parse(CONSTS, &["P>=T [F<=K x=2]"]);
    let out = ModelBuilder::new(&mut prism)
        .with_mdp_model_type()
        .with_query(queries.pop().unwrap())
        .build()
        .unwrap();
    let _: &Query<i64, f64, Ap> = &out.query;
    assert_eq!(show(&out.query), "P>=0.25 [ F<=2 0 ]");
}

mod query_needs_unlisted_label_panics {
    use super::*;

    #[test]
    #[should_panic(expected = "The property requires labels")]
    fn with_given_labels() {
        let (mut prism, mut queries) = parse(MODEL, &[r#"P=? [F "goal"]"#]);
        let _ = ModelBuilder::new(&mut prism)
            .with_mdp_model_type()
            .with_given_labels(HashSet::from(["start".to_string()]))
            .with_query(queries.pop().unwrap())
            .build();
    }

    #[test]
    #[should_panic(expected = "The property requires labels")]
    fn without_labels() {
        let (mut prism, mut queries) = parse(MODEL, &[r#"P=? [F "goal"]"#]);
        let _ = ModelBuilder::new(&mut prism)
            .with_mdp_model_type()
            .without_labels()
            .with_query(queries.pop().unwrap())
            .build();
    }
}
