use clap::Parser;
use prism_model_builder::ModelBuildingError;
use tiny_pmc::parsing::{ConstParsingError, Inputs, ModelAndPropArgs};
use tiny_pmc::{CheckerError, OutputPrinter};

mod arg_parsing;

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
    let processing_options = arguments.processing_options();
    let checker_options = arguments
        .value_iteration
        .to_checker_options()
        .map_err(ModelCheckerError::InvalidArguments)?;
    let constants = tiny_pmc::parsing::parse_const_assignments(&arguments.constants)?;
    let inputs = Inputs::from_cli_args(&arguments.files)?;

    tiny_pmc::build_and_check_model(
        inputs,
        constants,
        &processing_options,
        &checker_options,
        OutputPrinter::new(),
    )?;
    println!("Finished in {:?}", start_time.elapsed());
    Ok(())
}

enum ModelCheckerError {
    InvalidArguments(String),
    InputError(tiny_pmc::parsing::InputError),
    ConstParsingError(ConstParsingError),
    ModelBuildingError(ModelBuildingError),
    ModelCheckingError(CheckerError),
}

impl From<tiny_pmc::parsing::InputError> for ModelCheckerError {
    fn from(value: tiny_pmc::parsing::InputError) -> Self {
        Self::InputError(value)
    }
}

impl ModelCheckerError {
    pub fn print_and_get_error_code(self) -> i32 {
        match self {
            ModelCheckerError::InvalidArguments(err) => {
                println!("Invalid arguments: {err}");
                1
            }
            ModelCheckerError::InputError(err) => {
                println!("Input error: {err}");
                2
            }
            ModelCheckerError::ConstParsingError(err) => {
                println!("{err}");
                3
            }
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
