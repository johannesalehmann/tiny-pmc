use prism_model_builder::UserProvidedConstValue;
use prism_model_builder::queries::ProcessedQuery;
use probabilistic_models::AtomicPropositionIndex;
use probabilistic_models::traits::ReadStateSpace;

pub mod checking;
pub mod parsing;
pub mod state_space_restriction;
#[cfg(test)]
mod tests;
pub mod types;

pub use crate::checking::{CheckerError, CheckerOptions};
use crate::parsing::Inputs;
pub use types::*;

pub struct ProcessingOptions {
    pub dfs: Option<probabilistic_models::SuccessorOrder>,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self { dfs: None }
    }
}

pub fn build_and_check_model<Out: CheckerOutput>(
    input: Inputs,
    constants: impl IntoIterator<Item = (String, UserProvidedConstValue)>,
    processing_options: &ProcessingOptions,
    checker_options: &CheckerOptions,
    output: Out,
) -> Result<Out::Return, CheckerError> {
    let (model, queries) = build_model(input, constants, processing_options)?;
    check_model(model, queries, checker_options, output)
}

// TODO: Fold const assignments into inputs and then remove the separate parameter
pub fn build_model(
    input: Inputs,
    constants: impl IntoIterator<Item = (String, UserProvidedConstValue)>,
    options: &ProcessingOptions,
) -> Result<
    (
        OptionalModel,
        Vec<ProcessedQuery<AtomicPropositionIndex<usize>>>,
    ),
    CheckerError,
> {
    // TODO: Preserve property names through the following block. The UMB case does this already,
    //  whereas for PRISM, we'd have to change the `with_queries` function to take NamedQueries.
    //  Perhaps it makes sense to add a separate `with_named_queries` function.
    let (model, queries): (OptionalModel, _) = match input {
        Inputs::Prism { mut model, queries } => {
            let state_space_restriction = if queries.len() == 1 {
                crate::state_space_restriction::get_state_space_restriction(queries[0].as_ref())
            } else {
                None
            };

            let start_build = std::time::Instant::now();

            let builder = prism_model_builder::ModelBuilder::new_mdp_builder(&mut model)
                .with_necessary_labels()
                .with_queries(queries)
                .with_constants(constants)
                .without_choice_labels()
                .with_initial_state_vector()
                .with_state_space_restriction_maybe(state_space_restriction);

            let builder_output = builder.build();
            // TODO: Make this output configurable
            println!("Built model in {:?}", start_build.elapsed());
            (builder_output.model.into_optional(), builder_output.queries)
        }
        Inputs::Explicit { model, queries } => (model.into_optional(), queries.into()),
    };
    // let model = model.without_predecessors();

    // TODO: Print more information about the model (but make this also configurable)
    println!("Model has {} states", model.states().len());

    let model = match options.dfs {
        None => model,
        Some(dfs) => {
            // Drop labels and valuations, as dfs reordering does not yet support them. In the future,
            // it would be nice to add support for those.
            let model = model
                .without_choice_labels()
                .without_branch_labels()
                .without_valuations()
                .without_predecessors()
                .without_observations()
                .without_annotations();
            if let Some(model) = model.initial_states_unwrapped() {
                let start_reorder = std::time::Instant::now();
                let model = model.reorder_dfs(dfs.into());
                println!("Reordered states (dfs) in {:?}", start_reorder.elapsed());
                model.into_optional()
            } else {
                panic!(
                    "Cannot reorder model in depth-first order because the model has no initial states"
                );
            }
        }
    };

    Ok((model, queries))
}

// TODO: It would be nicer to pass in a NamedQueries struct. This could then also be used to print
//  property names for the results.
pub fn check_model<Out: CheckerOutput>(
    model: OptionalModel,
    queries: Vec<ProcessedQuery<AtomicPropositionIndex<usize>>>,
    checker_options: &CheckerOptions,
    mut output: Out,
) -> Result<Out::Return, CheckerError> {
    let model = model.unwrap_or_compute_predecessors();

    // TODO: This unwrapping should not happen here. Instead, pass the optional model to the check
    //  function and let it dynamically decide which features it needs.
    // TODO: The call to _unwrapped().unwrap() reads poorly. Probably, the _unwrapped function
    //  should be renamed and a new _unwrapped() function that returns the bare model should be
    //  created.
    let model = model
        .initial_states_unwrapped()
        .unwrap()
        .atomic_propositions_unwrapped()
        .unwrap()
        .rewards_unwrapped()
        .unwrap();

    for property in queries {
        output.started_check();
        let result = crate::checking::check(&model, property, &checker_options)?;
        output.finished_check(result);
    }
    Ok(output.into_return())
}

pub trait CheckerOutput {
    type Return;
    fn started_check(&mut self);
    fn finished_check(&mut self, result: f64);
    fn into_return(self) -> Self::Return;
}

pub struct OutputPrinter {
    start_time: std::time::Instant,
    counter: usize,
}

impl OutputPrinter {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            counter: 0,
        }
    }
}

impl CheckerOutput for OutputPrinter {
    type Return = ();

    fn started_check(&mut self) {
        self.start_time = std::time::Instant::now();
        print!("Property 1:");
        self.counter += 1;
    }

    fn finished_check(&mut self, result: f64) {
        println!(" {result} (in {:?})", self.start_time.elapsed());
    }

    fn into_return(self) -> Self::Return {
        ()
    }
}

pub struct OutputReturner {
    results: Vec<f64>,
}

impl OutputReturner {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
}

impl CheckerOutput for OutputReturner {
    type Return = Self;

    fn started_check(&mut self) {}

    fn finished_check(&mut self, result: f64) {
        self.results.push(result);
    }

    fn into_return(self) -> Self::Return {
        self
    }
}
