use crate::UnprocessedPrismQueries;

mod unprocessed_to_explicit_property;
pub use unprocessed_to_explicit_property::{
    FormulaProcessingError, from_unprocessed_to_explicit_properties,
};

// TODO: Also support single property functions here?
pub fn parse_properties(properties: &[&str]) -> Option<UnprocessedPrismQueries> {
    let parse = prism_parser::parse_unprocessed_props(properties);
    match parse.all_ok() {
        Ok(queries) => Some(queries),
        Err(errors) => {
            for error in errors {
                match error.source {
                    prism_parser::ErrorSource::Model => {
                        panic!(
                            "Received an error with `ErrorSource::Model` when parsing properties"
                        )
                    }
                    prism_parser::ErrorSource::Property {
                        property_file_index: index,
                    } => {
                        let name = format!("Property {}", index + 1);
                        error
                            .error
                            .print(Some(&name[..]), properties[index].as_ref());
                    }
                }
            }
            None
        }
    }
}
