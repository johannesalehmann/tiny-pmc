use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadStateSpace, ReadValuations, Transition,
};
use probabilistic_models::{StateIndex, Uniform};

type Ix = Uniform<usize>;

/// Parses `$source` and builds it with the default builder, after applying the given builder
/// methods, e.g. `build!(SOURCE, .with_dtmc_model_type().without_rewards())`.
macro_rules! build {
    ($source:expr $(, $($methods:tt)*)?) => {{
        let mut prism = prism_parser::parse_model($source).unwrap();
        crate::ModelBuilder::new(&mut prism) $($($methods)*)? .build()
    }};
}

mod choice_labels;
mod combined;
mod labels;
mod language;
mod model_types;
mod players;
mod queries;
mod rewards;
mod state_space;

fn transitions<M: ReadStateSpace>(model: &M) -> Vec<Transition<M::Ix>> {
    model.transitions().collect()
}

fn valuations<M: ReadValuations>(model: &M) -> Vec<String> {
    model
        .state_valuations()
        .map(|(_, v)| v.to_string())
        .collect()
}

// TODO: Parts of this could be generalised into a lookup by valuation in probabilistic-models.
fn state<M: ReadValuations>(model: &M, valuation: &str) -> StateIndex<M::Ix> {
    model
        .state_valuations()
        .find(|(_, v)| v.to_string() == valuation)
        .unwrap_or_else(|| panic!("No state with valuation `{valuation}`"))
        .0
}

/// Valuations of all states in which the atomic proposition `name` is set.
fn states_with<M: ReadAtomicPropositions + ReadValuations>(model: &M, name: &str) -> Vec<String> {
    let ap = model.atomic_proposition_by_name(name).unwrap();
    model
        .states_with_atomic_proposition(ap)
        .map(|s| model.state_valuation(s).to_string())
        .collect()
}
