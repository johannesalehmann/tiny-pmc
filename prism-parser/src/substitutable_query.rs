use prism_model::{
    CyclicDependency, Expression, FormulaManager, Identifier, LabelManager, Span,
    UnknownVariableError, VariableManager, VariableReference,
};
use probabilistic_properties::Query;

pub trait SubstitutableQuery<S: Span> {
    fn substitute_labels(
        &mut self,
        default_span: S,
        labels: &LabelManager<S, Expression<Identifier<S>, S>>,
    );

    fn substitute_formulas(
        &mut self,
        default_span: S,
        formulas: &FormulaManager<S, Expression<Identifier<S>, S>>,
    ) -> Result<(), CyclicDependency<S>>;

    fn replace_identifiers_by_variable_indices<R>(
        self,
        variable_manager: &VariableManager<S, R>,
    ) -> Result<
        Query<
            Expression<VariableReference, S>,
            Expression<VariableReference, S>,
            Expression<VariableReference, S>,
        >,
        Vec<UnknownVariableError<S>>,
    >;
}
impl<S: Span> SubstitutableQuery<S>
    for Query<
        Expression<Identifier<S>, S>,
        Expression<Identifier<S>, S>,
        Expression<Identifier<S>, S>,
    >
{
    fn substitute_labels(
        &mut self,
        default_span: S,
        labels: &LabelManager<S, Expression<Identifier<S>, S>>,
    ) {
        self.as_mut().map_e(&mut |ex| {
            ex.substitute_labels(default_span.clone(), labels);
        });
        self.as_mut().map_f(&mut |ex| {
            ex.substitute_labels(default_span.clone(), labels);
        });
        self.as_mut().map_i(&mut |ex| {
            ex.substitute_labels(default_span.clone(), labels);
        });
    }
    fn substitute_formulas(
        &mut self,
        default_span: S,
        formulas: &FormulaManager<S, Expression<Identifier<S>, S>>,
    ) -> Result<(), CyclicDependency<S>> {
        self.as_mut().try_map_e(&mut |ex| {
            ex.substitute_formulas(default_span.clone(), formulas)?;
            Ok(())
        })?;
        self.as_mut().try_map_f(&mut |ex| {
            ex.substitute_formulas(default_span.clone(), formulas)?;
            Ok(())
        })?;
        self.as_mut().try_map_i(&mut |ex| {
            ex.substitute_formulas(default_span.clone(), formulas)?;
            Ok(())
        })?;

        Ok(())
    }

    fn replace_identifiers_by_variable_indices<R>(
        self,
        variable_manager: &VariableManager<S, R>,
    ) -> Result<
        Query<
            Expression<VariableReference, S>,
            Expression<VariableReference, S>,
            Expression<VariableReference, S>,
        >,
        Vec<UnknownVariableError<S>>,
    > {
        self.try_map_i(&mut |ex| ex.replace_identifiers_by_variable_indices(variable_manager))?
            .try_map_f(&mut |ex| ex.replace_identifiers_by_variable_indices(variable_manager))?
            .try_map_e(&mut |ex| ex.replace_identifiers_by_variable_indices(variable_manager))
    }
}
