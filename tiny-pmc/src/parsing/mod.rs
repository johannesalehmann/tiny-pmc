use crate::PrismModel;
use ariadne::Source;
use chumsky::error::RichPattern;
use chumsky::util::MaybeRef;
use prism_model::{InvalidName, InvalidRangeForScopeKind, ModuleExpansionError, Span};
use prism_parser::{ParserError, ParserSpan, ValidationError};

mod constants;
mod maybe_builder;

use crate::parsing::maybe_builder::{MaybeLabel, MaybeReportBuilder};
pub use constants::{ConstParsingError, parse_const_assignments};

// TODO: Also support single property functions here?
// TODO: Make error printing available for any consumer of prism-parser instead of putting it in
//  tiny-pmc

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
                        print_error(&file_name, source, error.error);
                    }
                    prism_parser::ErrorSource::Property { index } => {
                        let name = format!("Property {}", index + 1);
                        print_error(&Some(&name[..]), properties[index].as_ref(), error.error);
                    }
                }
            }
            None
        }
        Ok(model_and_props) => Some((model_and_props.model, model_and_props.properties)),
    }
}

fn print_error(file_name: &Option<&str>, source: &str, error: ParserError<ParserSpan, String>) {
    let file_name = match file_name {
        Some(name) => name,
        None => "input",
    };

    let maybe_builder = match error {
        ParserError::ExpectedFound {
            span,
            expected,
            found,
            contexts,
            help,
        } => build_expected_found(span, &expected, found, &contexts, &help),

        ParserError::Validation(validation) => build_validation(validation),
    };
    let builder = maybe_builder.to_ariadne_builder(file_name);
    builder
        .finish()
        .print((file_name, Source::from(source)))
        .unwrap();
}

fn build_expected_found(
    span: ParserSpan,
    expected: &Vec<RichPattern<String>>,
    found: Option<MaybeRef<String>>,
    contexts: &Vec<(RichPattern<String>, ParserSpan)>,
    help: &Option<String>,
) -> MaybeReportBuilder<ParserSpan> {
    let mut builder = MaybeReportBuilder::new_error(&span);
    builder.set_message(format!(
        "Unexpected character{}",
        context_message(&contexts)
    ));
    builder.add_label(MaybeLabel::new(&span).with_message(found_message(found)));

    if !expected.is_empty() {
        builder.add_note(expected_message(&expected));
    }

    if let Some((_, location)) = contexts.first() {
        builder.add_label(MaybeLabel::new(location));
    }

    if let Some(help) = help {
        builder.add_help(help);
    }

    builder
}

fn expected_message(expected: &Vec<RichPattern<String>>) -> String {
    let mut message = Vec::new();
    message.push("Expected ".to_string());
    message.push(pattern_to_string(&expected[0]));
    if expected.len() > 1 {
        for pat in &expected[1..expected.len() - 1] {
            message.push(", ".to_string());
            message.push(pattern_to_string(&pat));
        }
        message.push(" or ".to_string());
        message.push(pattern_to_string(&expected[expected.len() - 1]));
    }
    message.push(".".to_string());
    let message = message.join("");
    message
}

fn found_message(found: Option<MaybeRef<String>>) -> String {
    match found {
        None => "Found end of file".to_string(),
        Some(found) => {
            format!("Found `{}`", found.to_string())
        }
    }
}

fn context_message(contexts: &Vec<(RichPattern<String>, ParserSpan)>) -> String {
    let context_message = contexts
        .iter()
        .map(|(pattern, _)| format!("\"{:?}\"", pattern))
        .collect::<Vec<_>>()
        .join(" in ");
    let context = if contexts.len() > 0 {
        format!(" while parsing {}", context_message)
    } else {
        "".to_string()
    };
    context
}

fn pattern_to_string(pattern: &RichPattern<String>) -> String {
    match pattern {
        RichPattern::Token(tok) => format!("`{}`", tok.to_string()),
        RichPattern::Label(l) => format!("{l}"),
        RichPattern::Identifier(i) => format!("'{i}'"),
        RichPattern::Any => "any".to_string(),
        RichPattern::SomethingElse => "something else".to_string(),
        RichPattern::EndOfInput => "end of input".to_string(),
    }
}

fn build_validation(error: ValidationError<ParserSpan>) -> MaybeReportBuilder<ParserSpan> {
    match error {
        ValidationError::UnsupportedModelType { model_type, span } => {
            let mut builder = MaybeReportBuilder::new_error(&span);
            builder.set_message("Unsupported model type");
            builder.add_label(
                MaybeLabel::new(&span)
                    .with_message(format!("Model type {} is not supported", model_type)),
            );
            builder.add_help("Supported model types are `dtmc`, `mdp` and `ctmc`.");
            builder
        }
        ValidationError::MissingModelType => {
            let mut builder = MaybeReportBuilder::new_error(&ParserSpan::empty());
            builder.set_message("Missing model type");
            builder.add_label(MaybeLabel::new(&ParserSpan::from_range(0..1)));
            builder.add_help("Add a line with `dtmc`, `mdp` or `ctmc` to your model.");
            builder
        }
        ValidationError::DuplicateModelType {
            first_occurrence,
            duplicate_occurrence,
        } => {
            let mut builder = MaybeReportBuilder::new_error(&duplicate_occurrence);
            builder.set_message("Duplicate model type");
            builder.add_label(
                MaybeLabel::new(&first_occurrence).with_message("Model type is first set here"),
            );
            builder.add_label(
                MaybeLabel::new(&duplicate_occurrence)
                    .with_message("Model type is defined again here"),
            );
            builder
        }
        ValidationError::DuplicateInitConstraint {
            first_occurrence,
            first_occurrence_inner,
            duplicate_occurrence,
            duplicate_occurrence_inner,
        } => {
            let mut builder = MaybeReportBuilder::new_error(&duplicate_occurrence);
            builder.set_message("Duplicate init constraint");
            builder.add_label(
                MaybeLabel::new(&first_occurrence)
                    .with_message("The init constraint is first set here"),
            );
            builder.add_label(MaybeLabel::new(&first_occurrence_inner));
            builder.add_label(
                MaybeLabel::new(&duplicate_occurrence)
                    .with_message("The duplicate init construct is set here"),
            );
            builder.add_label(MaybeLabel::new(&duplicate_occurrence_inner));
            builder
        }
        ValidationError::InvalidRangeForScope { span, range, kind } => {
            let mut builder = MaybeReportBuilder::new_error(&span);
            let (message, label, help) = match kind {
                InvalidRangeForScopeKind::BoundedIntConstant => (
                    "Illegal constant type",
                    format!("Type `{}` is not legal for constants", range.name()),
                    "Constants must not be of type bounded int. Use an unbounded `int` instead.",
                ),
                InvalidRangeForScopeKind::FloatVariable => (
                    "Illegal variable type",
                    format!("Type `{}` is not legal for variables", range.name()),
                    "Variables must not be of type `double`. Floating point values are only \
                     allowed for constants.",
                ),
            };
            builder.set_message(message);
            builder.add_label(MaybeLabel::new(&span));
            builder.add_label(MaybeLabel::new(range.span()).with_message(label));
            builder.add_help(help);
            builder
        }
        ValidationError::DuplicateElement {
            previous_occurrence,
            new_definition,
            ..
        } => {
            let mut builder = MaybeReportBuilder::new_error(&new_definition);
            builder.set_message("Duplicate name");
            builder.add_label(
                MaybeLabel::new(&previous_occurrence).with_message("First defined here"),
            );
            builder.add_label(MaybeLabel::new(&new_definition).with_message("Defined again here"));

            builder
        }
        ValidationError::InvalidIdentifierName { span, reason } => {
            let mut builder = MaybeReportBuilder::new_error(&span);
            builder.set_message("Invalid name");
            let (message, label_span, help) = match reason {
                InvalidName::Empty => ("Identifier must not be empty", span.clone(), None),
                InvalidName::StartsWithNumber { .. } => (
                    "Identifier must not start with number",
                    span.sliced(0..1),
                    None,
                ),
                InvalidName::InvalidCharacter { location, .. } => (
                    "Invalid character",
                    span.sliced(location..location + 1),
                    Some("Valid characters are `A`..`Z`, `a`..`z`, `0`..`9` and `_`."),
                ),
                InvalidName::Reserved { .. } => ("Is a reserved keyword", span.clone(), None),
            };
            builder.add_label(MaybeLabel::new(&label_span).with_message(message));
            if let Some(help) = help {
                builder.add_help(help);
            }
            builder
        }

        ValidationError::CyclicFormulaDependency { cycle } => {
            let mut builder = MaybeReportBuilder::new_error(&cycle.entries[0].formula_span);
            builder.set_message("Cyclic dependency between formulas");

            for i in 0..cycle.entries.len() {
                let depends_on_index = match i {
                    0 => cycle.entries.len() - 1,
                    i => i - 1,
                };
                let depends_on = cycle.entries[depends_on_index].formula_name.name.clone();
                let entry = &cycle.entries[i];
                builder.add_label(
                    MaybeLabel::new(&entry.dependency_span).with_message(format!(
                        "{} depends on {} here",
                        entry.formula_name.name, depends_on
                    )),
                );
                builder.add_label(MaybeLabel::new(&entry.formula_span));
            }

            builder
        }

        ValidationError::ModuleExpansion {
            error:
                ModuleExpansionError::DuplicateModule {
                    name,
                    original_module,
                    rename_rule: renaming_rule,
                },
        } => {
            let mut builder = MaybeReportBuilder::new_error(&renaming_rule);
            builder.set_message("Duplicate module during renaming");
            builder.add_label(MaybeLabel::new(&renaming_rule).with_message(format!(
                "This renaming rule creates a module with name {}",
                name
            )));
            builder.add_label(
                MaybeLabel::new(&original_module)
                    .with_message(format!("A module with name {} is first defined here", name)),
            );

            builder
        }
        ValidationError::ModuleExpansion {
            error:
                ModuleExpansionError::MissingVariableRenaming {
                    variable_name,
                    original_definition,
                    rename_rule: renaming_rule,
                },
        } => {
            let mut builder = MaybeReportBuilder::new_error(&renaming_rule);
            builder.set_message("Missing variable renaming during module renaming");

            builder.add_label(MaybeLabel::new(&renaming_rule).with_message(format!(
                "This renaming rule does not rename variable {}",
                variable_name
            )));
            builder.add_label(
                MaybeLabel::new(&original_definition)
                    .with_message(format!("{} is defined here", variable_name)),
            );

            builder.add_note(
                "When renaming a module, a new name must be given for every variable of the module.",
            );
            builder
        }
        ValidationError::ModuleExpansion {
            error:
                ModuleExpansionError::RenamingSourceDoesNotExist {
                    old_name,
                    renaming_rule,
                    ..
                },
        } => {
            let mut builder = MaybeReportBuilder::new_error(&renaming_rule);
            builder.set_message("Renamed module does not exist");

            builder.add_label(
                MaybeLabel::new(&old_name.span)
                    .with_message(format!("Cannot find module with name {}", old_name.name)),
            );

            builder
        }

        ValidationError::UnknownVariable { identifier } => {
            let mut builder = MaybeReportBuilder::new_error(&identifier.span);
            builder.set_message("Unknown variable or constant");

            builder.add_label(MaybeLabel::new(&identifier.span).with_message(format!(
                "Cannot find variable or constant {}",
                identifier.name
            )));

            builder
        }
    }
}
