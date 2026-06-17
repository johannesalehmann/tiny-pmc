use crate::PrismModel;

mod constants;
pub use constants::{ConstParsingError, parse_const_assignments};

// TODO: Also support single property functions here?

pub fn parse_prism_and_print_errors<'a, 'b>(
    file_name: Option<&'a str>,
    source: &'b str,
    properties: &'b [&'b str],
) -> Option<(PrismModel, Vec<crate::PrismQuery>)> {
    // TODO: Just return a ModelAndProps from this function?
    let parse_result = prism_parser::parse_model_and_props(source, properties);
    match parse_result.all_ok() {
        Err(errors) => {
            for error in errors {
                match error.source {
                    prism_parser::ErrorSource::Model => {
                        error.error.print(file_name, source);
                    }
                    prism_parser::ErrorSource::Property { index } => {
                        let name = format!("Property {}", index + 1);
                        error
                            .error
                            .print(Some(&name[..]), properties[index].as_ref());
                    }
                }
            }
            None
        }
        Ok(model_and_props) => Some((model_and_props.model, model_and_props.properties)),
    }
}
