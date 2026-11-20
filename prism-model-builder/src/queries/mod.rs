use crate::expression_context::ExpressionContext;
use crate::expressions::ValuationSource;
use crate::labels::Labels;
use crate::{ModelBuilder, ModelBuildingError};
use prism_model::{Expression, LabelManager, Span, VariableReference};
use probabilistic_models::{AtomicPropositionIndex, Ix, PlayerIndex};
use probabilistic_properties::{IndexBasedCoalition, QueryEnum};
use std::marker::PhantomData;
use typed_index_collections::NamedTo1;

pub trait QueryCollection {
    type Span: Span;
    type ProcessedType<I: Ix>;
    type OutputType<Model, I: Ix>;
    fn process_queries<
        I: Ix,
        EC: ExpressionContext<Expression<VariableReference, Self::Span>>,
        V: ValuationSource,
    >(
        &self,
        labels: &LabelManager<Self::Span, Expression<VariableReference, Self::Span>>,
        selected_labels: &mut Labels<
            AtomicPropositionIndex<I>,
            Expression<VariableReference, Self::Span>,
        >,
        players: &NamedTo1<PlayerIndex<I>, ()>,
        context: &mut EC,
        valuations: &V,
    ) -> Result<Self::ProcessedType<I>, ModelBuildingError>;
    fn output<I: Ix, Model>(
        model: Model,
        processed: Self::ProcessedType<I>,
    ) -> Self::OutputType<Model, I>;
}

pub type UnprocessedQuery<S> = QueryEnum<
    Expression<VariableReference, S>,
    Expression<VariableReference, S>,
    Expression<VariableReference, S>,
>;
pub type ProcessedQuery<I> =
    QueryEnum<i64, f64, AtomicPropositionIndex<I>, IndexBasedCoalition<PlayerIndex<I>>>;

pub struct ModelOnly<S: Span> {
    _phantom_data: PhantomData<S>,
}

impl<S: Span> Default for ModelOnly<S> {
    fn default() -> Self {
        Self {
            _phantom_data: PhantomData,
        }
    }
}

impl<S: Span> QueryCollection for ModelOnly<S> {
    type Span = S;
    type ProcessedType<I: Ix> = ();
    type OutputType<Model, I: Ix> = Model;

    fn process_queries<
        I: Ix,
        EC: ExpressionContext<Expression<VariableReference, S>>,
        V: ValuationSource,
    >(
        &self,
        _labels: &LabelManager<S, Expression<VariableReference, S>>,
        _selected_labels: &mut Labels<AtomicPropositionIndex<I>, Expression<VariableReference, S>>,
        _players: &NamedTo1<PlayerIndex<I>, ()>,
        _context: &mut EC,
        _valuations: &V,
    ) -> Result<Self::ProcessedType<I>, ModelBuildingError> {
        Ok(())
    }

    fn output<I: Ix, Model>(
        model: Model,
        _processed: Self::ProcessedType<I>,
    ) -> Self::OutputType<Model, I> {
        model
    }
}

impl<
    'a,
    S: Span,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    Sel: crate::model_type_selection::ModelTypeSelection,
    IB: crate::initial_states_builder::InitialStatesBuilder<Ix = Sel::Ix>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<Ix = Sel::Ix>,
    CL: crate::choice_labels::ChoiceLabelBuilder<Ix = Sel::Ix>,
    Rew: crate::rewards_builder::RewardsBuilder<Ix = Sel::Ix>,
> ModelBuilder<'a, S, ModelOnly<S>, L, IS, Sel, IB, APs, CL, Rew>
{
    pub fn with_query(
        self,
        query: UnprocessedQuery<S>,
    ) -> ModelBuilder<'a, S, SingleQuery<S>, L, IS, Sel, IB, APs, CL, Rew> {
        self.map_queries(SingleQuery { query })
    }
    pub fn with_queries<Q: Into<Vec<UnprocessedQuery<S>>>>(
        self,
        queries: Q,
    ) -> ModelBuilder<'a, S, QueryVector<S>, L, IS, Sel, IB, APs, CL, Rew> {
        self.map_queries(QueryVector {
            queries: queries.into(),
        })
    }
}

pub struct SingleQuery<S: Span> {
    query: UnprocessedQuery<S>,
}

pub struct ModelAndQuery<Model, I: Ix> {
    pub model: Model,
    pub query: ProcessedQuery<I>,
}

impl<S: Span> QueryCollection for SingleQuery<S> {
    type Span = S;
    type ProcessedType<I: Ix> = ProcessedQuery<I>;
    type OutputType<Model, I: Ix> = ModelAndQuery<Model, I>;

    fn process_queries<
        I: Ix,
        EC: ExpressionContext<Expression<VariableReference, S>>,
        V: ValuationSource,
    >(
        &self,
        labels: &LabelManager<S, Expression<VariableReference, S>>,
        selected_labels: &mut Labels<AtomicPropositionIndex<I>, Expression<VariableReference, S>>,
        players: &NamedTo1<PlayerIndex<I>, ()>,
        context: &mut EC,
        valuations: &V,
    ) -> Result<Self::ProcessedType<I>, ModelBuildingError> {
        map_query(
            self.query.clone(),
            labels,
            selected_labels,
            players,
            context,
            valuations,
            None,
        )
    }

    fn output<I: Ix, Model>(
        model: Model,
        query: Self::ProcessedType<I>,
    ) -> Self::OutputType<Model, I> {
        ModelAndQuery { model, query }
    }
}
impl<
    'a,
    S: Span,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    Sel: crate::model_type_selection::ModelTypeSelection,
    IB: crate::initial_states_builder::InitialStatesBuilder<Ix = Sel::Ix>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<Ix = Sel::Ix>,
    CL: crate::choice_labels::ChoiceLabelBuilder<Ix = Sel::Ix>,
    Rew: crate::rewards_builder::RewardsBuilder<Ix = Sel::Ix>,
> ModelBuilder<'a, S, SingleQuery<S>, L, IS, Sel, IB, APs, CL, Rew>
{
    pub fn and_with_query(
        self,
        query: UnprocessedQuery<S>,
    ) -> ModelBuilder<'a, S, QueryVector<S>, L, IS, Sel, IB, APs, CL, Rew> {
        self.map_queries_with(|q| QueryVector {
            queries: vec![q.query, query],
        })
    }
    pub fn and_with_queries(
        self,
        queries: Vec<UnprocessedQuery<S>>,
    ) -> ModelBuilder<'a, S, QueryVector<S>, L, IS, Sel, IB, APs, CL, Rew> {
        self.map_queries_with(|q| QueryVector {
            queries: std::iter::once(q.query)
                .chain(queries.into_iter())
                .collect(),
        })
    }
}

#[derive(Default)]
pub struct QueryVector<S: Span> {
    queries: Vec<UnprocessedQuery<S>>,
}

pub struct ModelAndQueries<Model, I: Ix> {
    pub model: Model,
    pub queries: Vec<ProcessedQuery<I>>,
}

impl<S: Span> QueryCollection for QueryVector<S> {
    type Span = S;
    type ProcessedType<I: Ix> = Vec<ProcessedQuery<I>>;
    type OutputType<Model, I: Ix> = ModelAndQueries<Model, I>;

    fn process_queries<
        I: Ix,
        EC: ExpressionContext<Expression<VariableReference, S>>,
        V: ValuationSource,
    >(
        &self,
        labels: &LabelManager<S, Expression<VariableReference, S>>,
        selected_labels: &mut Labels<AtomicPropositionIndex<I>, Expression<VariableReference, S>>,
        players: &NamedTo1<PlayerIndex<I>, ()>,
        context: &mut EC,
        valuations: &V,
    ) -> Result<Self::ProcessedType<I>, ModelBuildingError> {
        self.queries
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, query)| {
                map_query(
                    query,
                    labels,
                    selected_labels,
                    players,
                    context,
                    valuations,
                    Some(index),
                )
            })
            .collect()
    }

    fn output<I: Ix, Model>(
        model: Model,
        queries: Self::ProcessedType<I>,
    ) -> Self::OutputType<Model, I> {
        ModelAndQueries { model, queries }
    }
}
impl<
    'a,
    S: Span,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    Sel: crate::model_type_selection::ModelTypeSelection,
    IB: crate::initial_states_builder::InitialStatesBuilder<Ix = Sel::Ix>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<Ix = Sel::Ix>,
    CL: crate::choice_labels::ChoiceLabelBuilder<Ix = Sel::Ix>,
    Rew: crate::rewards_builder::RewardsBuilder<Ix = Sel::Ix>,
> ModelBuilder<'a, S, QueryVector<S>, L, IS, Sel, IB, APs, CL, Rew>
{
    pub fn and_with_query(
        self,
        query: UnprocessedQuery<S>,
    ) -> ModelBuilder<'a, S, QueryVector<S>, L, IS, Sel, IB, APs, CL, Rew> {
        self.map_queries_with(|mut qs| {
            qs.queries.push(query);
            QueryVector {
                queries: qs.queries,
            }
        })
    }
    pub fn and_with_queries(
        self,
        mut queries: Vec<UnprocessedQuery<S>>,
    ) -> ModelBuilder<'a, S, QueryVector<S>, L, IS, Sel, IB, APs, CL, Rew> {
        self.map_queries_with(|mut qs| {
            qs.queries.append(&mut queries);
            QueryVector {
                queries: qs.queries,
            }
        })
    }
}

fn map_query<
    S: Span,
    I: Ix,
    EC: ExpressionContext<Expression<VariableReference, S>>,
    V: ValuationSource,
>(
    property: UnprocessedQuery<S>,
    labels: &LabelManager<S, Expression<VariableReference, S>>,
    selected_labels: &mut Labels<AtomicPropositionIndex<I>, Expression<VariableReference, S>>,
    players: &NamedTo1<PlayerIndex<I>, ()>,
    context: &mut EC,
    valuations: &V,
    index: Option<usize>,
) -> Result<ProcessedQuery<I>, ModelBuildingError> {
    let property = property
        .try_map_c(&mut |coalition| coalition.to_index_based_coalition(players))
        .map_err(ModelBuildingError::UnknownCoalitionPlayer)?;
    Ok(property
        .map_i(&mut |i| context.evaluate_int(&i, valuations))
        .map_f(&mut |i| context.evaluate_float(&i, valuations))
        .map_e(&mut |e| {
            if let Expression::Label(label, _) = e {
                let label = labels
                    .get(labels.index_of_name(&label.name).unwrap())
                    .unwrap();
                selected_labels.get_or_add(label.name.name.clone(), label.condition.clone())
            } else {
                let name = if let Some(index) = index {
                    selected_labels.next_free_name(format!("query_ap_{index}"))
                } else {
                    selected_labels.next_free_name("query_ap".to_string())
                };
                selected_labels.get_or_add(name, e.clone())
            }
        }))
}
