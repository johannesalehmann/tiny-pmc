use prism_model::{Expression, Span, VariableReference};

pub fn prism_objectives_to_atomic_propositions<I, F, S: Span>(
    atomic_proposition: &mut Vec<Expression<VariableReference, S>>,
    queries: Vec<probabilistic_properties::Query<I, F, Expression<VariableReference, S>>>,
) -> Vec<probabilistic_properties::Query<I, F, probabilistic_models::AtomicProposition>> {
    let mut new_properties = Vec::new();
    for query in queries {
        new_properties.push(query.map_e(&mut |e| {
            let res = probabilistic_models::AtomicProposition::new(atomic_proposition.len());
            atomic_proposition.push(e);
            res
        }));
    }
    new_properties
}
