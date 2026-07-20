use clap::Parser;
use prism_model_builder::{ModelBuildingError, To1};
use probabilistic_models::traits::ReadStateSpace;
use probabilistic_models::{
    BranchIndex, ChoiceIndex, StateIndex, ValuationClassEntryIndex, ValuationClassIndex,
    ValuationIndex,
};
use tiny_pmc::CheckerError;
use tiny_pmc::parsing::ConstParsingError;

mod input;

mod arg_parsing;
#[cfg(test)]
mod tests;

fn main() {
    let exit_code = match checker() {
        Ok(()) => 0,
        Err(err) => err.print_and_get_error_code(),
    };
    std::process::exit(exit_code);
}

fn checker() -> Result<(), ModelCheckerError> {
    let start_time = std::time::Instant::now();

    let arguments = arg_parsing::Arguments::parse();
    let source = read_model_file(&arguments.model)?;
    let constants = tiny_pmc::parsing::parse_const_assignments(&arguments.constants)?;

    let parsed_model_and_objectives = tiny_pmc::parsing::parse_prism_and_print_errors(
        Some(&arguments.model),
        &source,
        &[&arguments.property],
    );
    let (mut prism_model, properties) = match parsed_model_and_objectives {
        None => return Err(ModelCheckerError::ModelAndPropertyParsingError),
        Some((prism_model, properties)) => (prism_model, properties),
    };

    let mut atomic_propositions = To1::new();
    let properties = tiny_pmc::building::prism_objectives_to_atomic_propositions(
        &mut atomic_propositions,
        properties,
    );
    let builder_builder = probabilistic_models::builder::ModelBuilderBuilder::new(
        probabilistic_models::builder::MdpBuilder::<
            StateIndex<usize>,
            ChoiceIndex<usize>,
            BranchIndex<usize>,
            ValuationClassIndex<u8>,
            ValuationClassEntryIndex<u16>,
            ValuationIndex<usize>,
        >::default(),
    );
    let builder = builder_builder.finish();
    let start_build = std::time::Instant::now();

    let builder = prism_model_builder::ModelBuilder::new_mdp_builder(&mut prism_model);
    let builder_output = prism_model_builder::build_model(
        &mut prism_model,
        builder,
        &atomic_propositions,
        properties.into_iter(),
        &constants,
    )?;
    println!("Built model in {:?}", start_build.elapsed());
    let model = builder_output.model;
    let properties = builder_output.properties;

    println!("Model has {} states", model.states().len());

    model.tra_file().write_to_file("model.tra").unwrap();
    model.sta_file().write_to_file("model.sta").unwrap();
    model.lab_file().write_to_file("model.lab").unwrap();
    println!("Wrote files to `model.tra`, `model.sta` and `model.lab`");

    if properties.len() > 1 {
        panic!("Checking multiple properties is temporarily unsupported");
    }
    // for (i, property) in properties.iter().enumerate() {
    println!("Checking property {} of {}", 0 + 1, properties.len());
    // tiny_pmc::checking::check(model, properties[0].clone())?;
    // }

    println!("Finished in {:?}", start_time.elapsed());
    Ok(())
}

fn read_model_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

enum ModelCheckerError {
    InputFileError(std::io::Error),
    ConstParsingError(ConstParsingError),
    ModelAndPropertyParsingError,
    ModelBuildingError(ModelBuildingError),
    ModelCheckingError(CheckerError),
}

impl ModelCheckerError {
    pub fn print_and_get_error_code(self) -> i32 {
        match self {
            ModelCheckerError::InputFileError(err) => {
                println!("Could not read input file: {err}");
                1
            }
            ModelCheckerError::ConstParsingError(err) => {
                println!("{err}");
                2
            }
            ModelCheckerError::ModelAndPropertyParsingError => 3, // This error is already printed when it is produced
            ModelCheckerError::ModelBuildingError(err) => {
                println!("Error during model building: {:?}", err);
                4
            }
            ModelCheckerError::ModelCheckingError(err) => {
                println!("Error during model checking: {:?}", err);
                5
            }
        }
    }
}

impl From<std::io::Error> for ModelCheckerError {
    fn from(value: std::io::Error) -> Self {
        ModelCheckerError::InputFileError(value)
    }
}

impl From<ConstParsingError> for ModelCheckerError {
    fn from(value: ConstParsingError) -> Self {
        ModelCheckerError::ConstParsingError(value)
    }
}

impl From<ModelBuildingError> for ModelCheckerError {
    fn from(value: ModelBuildingError) -> Self {
        ModelCheckerError::ModelBuildingError(value)
    }
}

impl From<CheckerError> for ModelCheckerError {
    fn from(value: CheckerError) -> Self {
        ModelCheckerError::ModelCheckingError(value)
    }
}
