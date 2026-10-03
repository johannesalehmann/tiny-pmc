use super::model_and_prop_args::ModelAndPropArgs;
use crate::parsing::PropertySource;
use crate::{ExplicitModel, ExplicitQueries, PrismModel, PrismQueries};
use probabilistic_properties::NamedQueries;
use std::fmt::{Display, Formatter};

pub enum Inputs {
    Prism {
        model: PrismModel,
        queries: PrismQueries,
    },
    Explicit {
        model: ExplicitModel,
        queries: ExplicitQueries,
    },
}

#[derive(Debug)]
pub enum InputError {
    InputFileError { path: String, error: std::io::Error },
    NoModelFile,
    MultipleInputFiles(Vec<String>),
    ModelAndPropertyParsingError,
}

trait WithPath<T> {
    fn with_path(self, path: &str) -> Result<T, InputError>;
}

impl<T> WithPath<T> for Result<T, std::io::Error> {
    fn with_path(self, path: &str) -> Result<T, InputError> {
        self.map_err(|error| InputError::InputFileError {
            path: path.to_string(),
            error,
        })
    }
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::InputFileError { path, error } => {
                write!(f, "Could not read file `{path}`: {error}")
            }
            InputError::NoModelFile => {
                write!(
                    f,
                    "No model file provided (recognised extensions: `.prism`, `.umb`, `.nm`, `.pm`, `.sm`)"
                )
            }
            InputError::MultipleInputFiles(files) => {
                write!(
                    f,
                    "More than one model file provided ({})",
                    files
                        .into_iter()
                        .map(|f| format!("`{}`", f))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            InputError::ModelAndPropertyParsingError => {
                // This error is already printed when it is produced
                Ok(())
            }
        }
    }
}

impl Inputs {
    pub fn from_cli_args(args: &[String]) -> Result<Inputs, InputError> {
        let args = ModelAndPropArgs::from_cli_args(args);
        Self::new(args)
    }

    pub fn new(args: ModelAndPropArgs) -> Result<Inputs, InputError> {
        if args.prism_files.is_empty() && args.umb_files.is_empty() {
            return Err(InputError::NoModelFile);
        }
        if args.prism_files.len() + args.umb_files.len() > 1 {
            let files = args
                .prism_files
                .into_iter()
                .chain(args.umb_files.into_iter())
                .collect::<Vec<_>>();
            return Err(InputError::MultipleInputFiles(files));
        }

        let property_strings: Vec<_> = args
            .property_sources
            .into_iter()
            .map(|p| match p {
                PropertySource::File(path) => read_input_file(&path),
                PropertySource::String(val) => Ok(val),
            })
            .collect::<Result<_, _>>()?;

        if let Some(prism_file) = args.prism_files.get(0) {
            let source = read_input_file(prism_file)?;
            let parsed_model_and_objectives = crate::parsing::parse_prism_and_print_errors(
                Some(&prism_file),
                &source,
                &property_strings
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(), // TODO: Allow `parse_prism_and_print_errors` to take &[String] in addition to &[&str]
            );
            match parsed_model_and_objectives {
                Some((model, queries)) => Ok(Inputs::Prism {
                    model,
                    queries: Self::filter_queries(queries, args.property_names),
                }),
                None => Err(InputError::ModelAndPropertyParsingError),
            }
        } else {
            // TODO: Proper error handling in this path
            let umb_file = args.umb_files.get(0).unwrap();
            let model = umb_parser::parse_umb(umb_file).unwrap();

            let Some(unprocessed_queries) = crate::parsing::parse_properties(
                &property_strings
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
            ) else {
                return Err(InputError::ModelAndPropertyParsingError);
            };
            let queries = if let Some(ap_model) = model.as_ref().atomic_propositions_unwrapped() {
                crate::parsing::from_unprocessed_to_explicit_properties(
                    unprocessed_queries,
                    &ap_model,
                )
                .unwrap()
            } else {
                if !unprocessed_queries.is_empty() {
                    println!(
                        "Warning: UMB model did not come with atomic propositions. Thus, the queries cannot be parsed."
                    )
                }
                NamedQueries::new()
            };

            Ok(Inputs::Explicit {
                model,
                queries: Self::filter_queries(queries, args.property_names),
            })
        }
    }

    fn filter_queries<I, F, E>(
        queries: NamedQueries<I, F, E>,
        names: Vec<String>,
    ) -> NamedQueries<I, F, E> {
        // TODO: This isn't quite doing what it's supposed to do: If names contains a query that
        //  doesn't exist, it will just ignore that, instead of flagging an error.
        if names.is_empty() {
            queries
        } else {
            queries.filtered_by_name(|n| names.iter().any(|name| name == n))
        }
    }
}

fn read_input_file(path: &str) -> Result<String, InputError> {
    std::fs::read_to_string(path).with_path(path)
}
