use crate::ModelBuilder;
use crate::expression_context::ExpressionContext;
use crate::expressions::ValuationSource;
use crate::labels::Labels;
use prism_model::{Expression, LabelManager, Span, VariableReference};
use probabilistic_properties::Query;
use std::marker::PhantomData;
use typed_index_collections::Index;

pub trait QueryCollection {
    type Span: Span;
    type ProcessedType<APIdx: Index>;
    type OutputType<Model, APIdx: Index>;
    fn process_queries<
        APIdx: Index,
        EC: ExpressionContext<Expression<VariableReference, Self::Span>>,
        V: ValuationSource,
    >(
        &self,
        labels: &LabelManager<Self::Span, Expression<VariableReference, Self::Span>>,
        selected_labels: &mut Labels<APIdx, Expression<VariableReference, Self::Span>>,
        context: &mut EC,
        valuations: &V,
    ) -> Self::ProcessedType<APIdx>;
    fn output<APIdx: Index, Model>(
        model: Model,
        processed: Self::ProcessedType<APIdx>,
    ) -> Self::OutputType<Model, APIdx>;
}

pub type UnprocessedQuery<S> = Query<
    Expression<VariableReference, S>,
    Expression<VariableReference, S>,
    Expression<VariableReference, S>,
>;
pub type ProcessedQuery<APIdx> = Query<i64, f64, APIdx>;

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
    type ProcessedType<APIdx: Index> = ();
    type OutputType<Model, APIdx: Index> = Model;

    fn process_queries<
        APIdx: Index,
        EC: ExpressionContext<Expression<VariableReference, S>>,
        V: ValuationSource,
    >(
        &self,
        _labels: &LabelManager<S, Expression<VariableReference, S>>,
        _selected_labels: &mut Labels<APIdx, Expression<VariableReference, S>>,
        _context: &mut EC,
        _valuations: &V,
    ) -> Self::ProcessedType<APIdx> {
        ()
    }

    fn output<APIdx: Index, Model>(
        model: Model,
        _processed: Self::ProcessedType<APIdx>,
    ) -> Self::OutputType<Model, APIdx> {
        model
    }
}

impl<
    'a,
    S: Span,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    B: crate::bases::BaseModelBuilder,
    IB: crate::initial_states_builder::InitialStatesBuilder<StateIdx = B::StateIdx>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = B::StateIdx>,
    CL: crate::choice_labels::ChoiceLabelBuilder<ChoiceIdx = B::ChoiceIdx>,
> ModelBuilder<'a, S, ModelOnly<S>, L, IS, B, IB, APs, CL>
{
    pub fn with_query(
        self,
        query: UnprocessedQuery<S>,
    ) -> ModelBuilder<'a, S, SingleQuery<S>, L, IS, B, IB, APs, CL> {
        self.map_queries(SingleQuery { query })
    }
    pub fn with_queries(
        self,
        queries: Vec<UnprocessedQuery<S>>,
    ) -> ModelBuilder<'a, S, QueryVector<S>, L, IS, B, IB, APs, CL> {
        self.map_queries(QueryVector { queries })
    }
}

pub struct SingleQuery<S: Span> {
    query: UnprocessedQuery<S>,
}

pub struct ModelAndQuery<Model, APIdx: Index> {
    pub model: Model,
    pub query: ProcessedQuery<APIdx>,
}

impl<S: Span> QueryCollection for SingleQuery<S> {
    type Span = S;
    type ProcessedType<APIdx: Index> = ProcessedQuery<APIdx>;
    type OutputType<Model, APIdx: Index> = ModelAndQuery<Model, APIdx>;

    fn process_queries<
        APIdx: Index,
        EC: ExpressionContext<Expression<VariableReference, S>>,
        V: ValuationSource,
    >(
        &self,
        labels: &LabelManager<S, Expression<VariableReference, S>>,
        selected_labels: &mut Labels<APIdx, Expression<VariableReference, S>>,
        context: &mut EC,
        valuations: &V,
    ) -> Self::ProcessedType<APIdx> {
        map_query(
            self.query.clone(),
            labels,
            selected_labels,
            context,
            valuations,
            None,
        )
    }

    fn output<APIdx: Index, Model>(
        model: Model,
        query: Self::ProcessedType<APIdx>,
    ) -> Self::OutputType<Model, APIdx> {
        ModelAndQuery { model, query }
    }
}
impl<
    'a,
    S: Span,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    B: crate::bases::BaseModelBuilder,
    IB: crate::initial_states_builder::InitialStatesBuilder<StateIdx = B::StateIdx>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = B::StateIdx>,
    CL: crate::choice_labels::ChoiceLabelBuilder<ChoiceIdx = B::ChoiceIdx>,
> ModelBuilder<'a, S, SingleQuery<S>, L, IS, B, IB, APs, CL>
{
    pub fn and_with_query(
        self,
        query: UnprocessedQuery<S>,
    ) -> ModelBuilder<'a, S, QueryVector<S>, L, IS, B, IB, APs, CL> {
        self.map_queries_with(|q| QueryVector {
            queries: vec![q.query, query],
        })
    }
    pub fn and_with_queries(
        self,
        queries: Vec<UnprocessedQuery<S>>,
    ) -> ModelBuilder<'a, S, QueryVector<S>, L, IS, B, IB, APs, CL> {
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

pub struct ModelAndQueries<Model, APIdx: Index> {
    pub model: Model,
    pub queries: Vec<ProcessedQuery<APIdx>>,
}

impl<S: Span> QueryCollection for QueryVector<S> {
    type Span = S;
    type ProcessedType<APIdx: Index> = Vec<ProcessedQuery<APIdx>>;
    type OutputType<Model, APIdx: Index> = ModelAndQueries<Model, APIdx>;

    fn process_queries<
        APIdx: Index,
        EC: ExpressionContext<Expression<VariableReference, S>>,
        V: ValuationSource,
    >(
        &self,
        labels: &LabelManager<S, Expression<VariableReference, S>>,
        selected_labels: &mut Labels<APIdx, Expression<VariableReference, S>>,
        context: &mut EC,
        valuations: &V,
    ) -> Self::ProcessedType<APIdx> {
        self.queries
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, query)| {
                map_query(
                    query,
                    labels,
                    selected_labels,
                    context,
                    valuations,
                    Some(index),
                )
            })
            .collect()
    }

    fn output<APIdx: Index, Model>(
        model: Model,
        queries: Self::ProcessedType<APIdx>,
    ) -> Self::OutputType<Model, APIdx> {
        ModelAndQueries { model, queries }
    }
}
impl<
    'a,
    S: Span,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    B: crate::bases::BaseModelBuilder,
    IB: crate::initial_states_builder::InitialStatesBuilder<StateIdx = B::StateIdx>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = B::StateIdx>,
    CL: crate::choice_labels::ChoiceLabelBuilder<ChoiceIdx = B::ChoiceIdx>,
> ModelBuilder<'a, S, QueryVector<S>, L, IS, B, IB, APs, CL>
{
    pub fn and_with_query(
        self,
        query: UnprocessedQuery<S>,
    ) -> ModelBuilder<'a, S, QueryVector<S>, L, IS, B, IB, APs, CL> {
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
    ) -> ModelBuilder<'a, S, QueryVector<S>, L, IS, B, IB, APs, CL> {
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
    APIdx: Index,
    EC: ExpressionContext<Expression<VariableReference, S>>,
    V: ValuationSource,
>(
    property: UnprocessedQuery<S>,
    labels: &LabelManager<S, Expression<VariableReference, S>>,
    selected_labels: &mut Labels<APIdx, Expression<VariableReference, S>>,
    context: &mut EC,
    valuations: &V,
    index: Option<usize>,
) -> ProcessedQuery<APIdx> {
    property
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
        })
}
