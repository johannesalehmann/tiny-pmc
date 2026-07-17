use prism_model::{Expression, VariableReference};
use probabilistic_properties::Query;
use typed_index_collections::Index;

pub trait QueryCollection {
    type ProcessedType<APIdx>;
    type OutputType<Model, APIdx>;
    fn process_properties<APIdx: Index>(self) -> Self::ProcessedType<APIdx>;
    fn output<APIdx: Index, Model>(
        model: Model,
        processed: Self::ProcessedType<APIdx>,
    ) -> Self::OutputType<Model, APIdx>;
}

pub type ProcessedQuery<APIdx> = Query<i64, f64, APIdx>;

#[derive(Default)]
pub struct ModelOnly {}

impl QueryCollection for ModelOnly {
    type ProcessedType<APIdx> = ();
    type OutputType<Model, APIdx> = Model;

    fn process_properties<APIdx: Index>(self) -> Self::ProcessedType<APIdx> {
        ()
    }

    fn output<APIdx: Index, Model>(
        model: Model,
        _processed: Self::ProcessedType<APIdx>,
    ) -> Self::OutputType<Model, APIdx> {
        model
    }
}

#[derive(Default)]
pub struct SingleQuery<S: prism_model::Span> {
    property: Query<
        Expression<VariableReference, S>,
        Expression<VariableReference, S>,
        Expression<VariableReference, S>,
    >,
}

pub struct ModelAndQuery<Model, APIdx: Index> {
    model: Model,
    query: ProcessedQuery<APIdx>,
}

impl<S: prism_model::Span> QueryCollection for SingleQuery<S> {
    type ProcessedType<APIdx> = ProcessedQuery<APIdx>;
    type OutputType<Model, APIdx> = ModelAndQuery<Model, APIdx>;

    fn process_properties<APIdx: Index>(self) -> Self::ProcessedType<APIdx> {
        todo!()
    }

    fn output<APIdx: Index, Model>(
        model: Model,
        query: Self::ProcessedType<APIdx>,
    ) -> Self::OutputType<Model, APIdx> {
        ModelAndQuery { model, query }
    }
}

#[derive(Default)]
pub struct QueryVector<S: prism_model::Span> {
    property: Query<
        Expression<VariableReference, S>,
        Expression<VariableReference, S>,
        Expression<VariableReference, S>,
    >,
}

pub struct ModelAndQueries<Model, APIdx: Index> {
    model: Model,
    queries: Vec<ProcessedQuery<APIdx>>,
}

impl<S: prism_model::Span> QueryCollection for QueryVector<S> {
    type ProcessedType<APIdx> = Vec<ProcessedQuery<APIdx>>;
    type OutputType<Model, APIdx> = ModelAndQueries<Model, APIdx>;

    fn process_properties<APIdx: Index>(self) -> Self::ProcessedType<APIdx> {
        todo!()
    }

    fn output<APIdx: Index, Model>(
        model: Model,
        queries: Self::ProcessedType<APIdx>,
    ) -> Self::OutputType<Model, APIdx> {
        ModelAndQueries { model, queries }
    }
}
