use crate::{ParserError, ParserSpan, ValidationError};
use chumsky::error::RichPattern;
use chumsky::util::MaybeRef;
use maybe_spanned_error_helper::{MaybeLabel, MaybeReportBuilder, Source};
use prism_model::{InvalidName, InvalidRangeForScopeKind, ModuleExpansionError, Span};

impl<'a> ParserError<'a, ParserSpan, String> {
    /// Prints the error to stdout. If the error has [`Span`]`s`, the relevant sections of source
    /// code are highlighted.
    ///
    /// - `file_name` is the name of the source file. This is used to annotate the errors in the
    ///   debug output
    /// - `source` is the content of the source file.
    pub fn print(self, file_name: Option<&str>, source: &str) {
        let file_name = file_name.unwrap_or_else(|| "input");

        let maybe_builder = match self {
            ParserError::ExpectedFound {
                span,
                expected,
                found,
                contexts,
                help,
            } => Self::build_expected_found(span, &expected, found, &contexts, &help),

            ParserError::Validation(validation) => Self::build_validation(validation),
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
            Self::context_message(&contexts)
        ));
        builder.add_label(MaybeLabel::new(&span).with_message(Self::found_message(found)));

        if !expected.is_empty() {
            builder.add_note(Self::expected_message(&expected));
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
        message.push(Self::pattern_to_string(&expected[0]));
        if expected.len() > 1 {
            for pat in &expected[1..expected.len() - 1] {
                message.push(", ".to_string());
                message.push(Self::pattern_to_string(&pat));
            }
            message.push(" or ".to_string());
            message.push(Self::pattern_to_string(&expected[expected.len() - 1]));
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
                builder
                    .add_label(MaybeLabel::new(&new_definition).with_message("Defined again here"));

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
                    builder.add_label(MaybeLabel::new(&entry.dependency_span).with_message(
                        format!("{} depends on {} here", entry.formula_name.name, depends_on),
                    ));
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
}
