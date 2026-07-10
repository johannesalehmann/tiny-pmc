use prism_model::{Expression, Span, VariableReference};
use prism_model_builder::{AtomicPropositionIndex, To1};
use probabilistic_properties::Query;

pub fn prism_objectives_to_atomic_propositions<I, F, S: Span>(
    atomic_proposition: &mut To1<AtomicPropositionIndex<usize>, Expression<VariableReference, S>>,
    queries: Vec<probabilistic_properties::Query<I, F, Expression<VariableReference, S>>>,
) -> Vec<Query<I, F, AtomicPropositionIndex<usize>>> {
    let mut new_properties = Vec::new();
    for query in queries {
        new_properties.push(query.map_e(&mut |e| atomic_proposition.add(e)));
    }
    new_properties
}
