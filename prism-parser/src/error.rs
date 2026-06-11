use chumsky::error::{Error, LabelError, RichPattern};
use chumsky::input::Input;
use chumsky::util::MaybeRef;
use prism_model::{
    CyclicDependency, Expression, Identifier, ModuleExpansionError, UnknownVariableError,
};

/// An error produced while parsing and processing the model.
#[derive(Debug, PartialEq)]
pub enum ParserError<'a, S: prism_model::Span, T> {
    /// The given source did not fit the PRISM grammar. This manifests itself in errors of the
    /// form "Found token `found`, expected one of `expected`".
    ///
    /// This error additionally contains a set of `contexts` (during which part of the grammar this
    /// error occurred) and `help`, which is additional context that may be useful.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::borrow::Cow;
    /// use chumsky::error::RichPattern;
    /// use chumsky::util::Maybe;
    /// # use prism_model::{FullSpan, Span};
    /// # use prism_parser::{parse_model, ParserError, Token};
    /// let source =
    /// r"mdp
    /// module main
    ///     x: [0..10] init 0;
    ///     (x<10) -> (x'=x+1); // This command is missing `[]` at the beginning!
    /// endmodule main";
    /// let parsed = parse_model(source);
    /// assert_eq!(parsed, Err(vec![
    ///     ParserError::ExpectedFound {
    ///         span: FullSpan::from_range(43..44),
    ///         expected: vec![
    ///             RichPattern::Label(Cow::Borrowed("command")),
    ///             RichPattern::Label(Cow::Borrowed("variable declaration")),
    ///             RichPattern::Token(Maybe::Val("endmodule".to_string()))
    ///         ],
    ///         found: Some(Maybe::Val("(".to_string())),
    ///         contexts: vec![
    ///             ((RichPattern::Label(Cow::Borrowed("module")), FullSpan::from_range(4..38)))
    ///         ],
    ///         help: None
    ///     }
    /// ]));
    /// ```
    ///
    /// Here, the error indicates that at location `43..44`, the parser expected either a command,
    /// a label or the token `endmodule`. Instead, it found token `(`. This happened in the context
    /// of parsing a module (which, so far, covers `4..38`. No additional help is provided.
    ExpectedFound {
        /// The span where the unexpected token was found
        span: S,

        /// A list of patterns that were legal at this location.
        ///
        /// `RichPattern::Label` is used for named sub-parts of the grammar, e.g. `command` or
        /// `variable declaration`. `RichPattern::Token` is used for individual tokens (e.g.
        /// keywords, numbers, variable names).
        ///
        /// See [the variant docs](ParserError::ExpectedFound) for an example.
        expected: Vec<RichPattern<'a, T>>,

        /// The illegal pattern that was found at this location. If the end of the file was
        /// encountered, this is `None`.
        found: Option<MaybeRef<'a, T>>,

        /// A list of contexts, describing within which parsers this error occurred.
        ///
        /// See [the variant docs](ParserError::ExpectedFound) for an example.
        contexts: Vec<(RichPattern<'a, T>, S)>,

        /// Additional help text associated with this error.
        ///
        /// Currently, this is just used for a specific case where the default error message would
        /// not be very helpful (related to function names being reserved keywords). If you are
        /// printing the error, you can just display the help text at the end.
        help: Option<String>,
    },
    /// The source fit the PRISM grammar, but a semantic rule of the PRISM language was violated.
    Validation(ValidationError<S>),
}

/// A semantic rule of PRISM was violated.
// TODO: Add examples to each variant
#[derive(Debug, PartialEq)]
pub enum ValidationError<S: prism_model::Span> {
    /// This model type is not yet supported by the parser
    UnsupportedModelType {
        /// The name of the unsupported model type
        model_type: &'static str,
        /// The span where the model type was declared
        span: S,
    },

    /// The model does not have a model type.
    ///
    /// PRISM actually allows omitting the model type and will infer it from context, but this is
    /// not yet supported by this crate.
    MissingModelType,

    /// The model contains two model type declarations.
    DuplicateModelType {
        /// The span where the model type is declared first.
        first_occurrence: S,
        /// The span where model type is declared again.
        duplicate_occurrence: S,
    },

    /// The model contains two init constraints.
    DuplicateInitConstraint {
        /// The span covering the first init constraint (including the `init` and `endinit`)
        first_occurrence: S,
        /// The span of the contents of the first init constraint (not including `init`, `endinit`
        /// and surrounding whitespace).
        first_occurrence_inner: S,
        /// The span covering the second init constraint (including the `init` and `endinit`)
        duplicate_occurrence: S,
        /// The span of the contents of the second init constraint (not including `init`, `endinit`
        /// and surrounding whitespace).
        duplicate_occurrence_inner: S,
    },

    /// A variable or constant has a range (i.e. type) that is not legal for this scope.
    ///
    /// This happens if a constant is declared with type bounded int or if a variable (local or
    /// global) is declared with type double.
    InvalidRangeForScope {
        /// The span of the variable declaration.
        span: S,
        /// The range that is illegal for a variable of this scope.
        range: prism_model::VariableRange<S, Expression<Identifier<S>, S>>,
        /// The type of violation encountered.
        kind: prism_model::InvalidRangeForScopeKind,
    },

    /// The program contains two elements with the same name.
    ///
    /// In most cases, this is only triggered if both elements are of the same type. For example,
    /// you can have both a module named `x` and a variable named `x`.
    // TODO: I don't think name clashes between formulas and variables are currently being detected.
    DuplicateElement {
        /// The span where an element with this name is first declared
        previous_occurrence: S,
        /// The span where another element with this name is declared.
        new_definition: S,
        /// The kind of element that is duplicated. If the elements that clash are of different
        /// types, this refers to the element declared at `new_definition`.
        kind: ElementKind,
    },

    /// An identifier used is invalid
    InvalidIdentifierName {
        /// The span of the invalid identifier.
        span: S,
        /// Why the identifier is invalid.
        ///
        /// Note that only some of these errors are produced by the parser. In particular,
        /// `InvalidName::Empty` will never be produced by the parser.
        reason: prism_model::InvalidName,
    },

    /// Formulas cyclically depend on each other.
    ///
    /// See [`prism_model::CyclicDependency`] for detailed examples.
    CyclicFormulaDependency {
        /// The cycle of dependencies between formulas.
        cycle: prism_model::CyclicDependency<S>,
    },

    /// A renamed module could not be expanded, e.g. because it refers to an invalid source module
    /// or because the renaming rules are not exhaustive.
    ModuleExpansion {
        /// The reason why the renamed module could not be expanded.
        error: prism_model::ModuleExpansionError<S>,
    },

    /// A variable is not known.
    ///
    /// PRISM syntax does not distinguish between references to variables, constants and formulas,
    /// so this error might also be caused by an unknown constant or formula.
    UnknownVariable {
        /// The identifier of the unknown variable, constant or formula.
        identifier: Identifier<S>,
    },
}

/// A type of component of a PRISM model (used by [`ValidationError::DuplicateElement`]).
#[derive(Debug, PartialEq)]
pub enum ElementKind {
    /// A global variable
    GlobalVar,
    /// A local variable, i.e. one declared within a module
    LocalVar,
    /// A constant
    Const,
    /// A label
    Label,
    /// A formula
    Formula,
    /// A rewards structure
    Reward,
    /// A module
    Module,
}

macro_rules! derive_into {
    ($name: ident, $constructor: expr) => {
        impl<S: prism_model::Span> Into<ValidationError<S>> for $name<S> {
            fn into(self) -> ValidationError<S> {
                ($constructor)(self)
            }
        }

        impl<'a, S: prism_model::Span, T> Into<ParserError<'a, S, T>> for $name<S> {
            fn into(self) -> ParserError<'a, S, T> {
                Into::<ValidationError<S>>::into(self).into()
            }
        }
    };
}

derive_into!(CyclicDependency, |e| {
    ValidationError::CyclicFormulaDependency { cycle: e }
});

derive_into!(ModuleExpansionError, |e| {
    ValidationError::ModuleExpansion { error: e }
});

derive_into!(UnknownVariableError, |e: UnknownVariableError<_>| {
    ValidationError::UnknownVariable {
        identifier: e.identifier,
    }
});

impl<'a, S: prism_model::Span, T> Into<ParserError<'a, S, T>> for ValidationError<S> {
    fn into(self) -> ParserError<'a, S, T> {
        ParserError::Validation(self)
    }
}

impl<'a, S: prism_model::Span, T> ParserError<'a, S, T> {
    pub(crate) fn into_owned<'b>(self) -> ParserError<'b, S, T>
    where
        T: Clone,
    {
        match self {
            Self::ExpectedFound {
                found,
                expected,
                span,
                contexts,
                help,
            } => ParserError::ExpectedFound {
                expected: expected.into_iter().map(RichPattern::into_owned).collect(),
                found: found.map(MaybeRef::into_owned),
                span,
                contexts: contexts
                    .into_iter()
                    .map(|(p, s)| (p.into_owned(), s))
                    .collect(),
                help,
            },
            Self::Validation(validation) => ParserError::Validation(validation),
        }
    }

    pub(crate) fn map_token<U, F: FnMut(T) -> U>(self, mut f: F) -> ParserError<'a, S, U>
    where
        T: Clone,
    {
        match self {
            Self::ExpectedFound {
                expected,
                found,
                span,
                contexts,
                help,
            } => ParserError::ExpectedFound {
                expected: expected
                    .into_iter()
                    .map(|pat| pat.map_token(&mut f))
                    .collect(),
                span,
                found: found.map(|found| f(found.into_inner()).into()),
                contexts: contexts
                    .into_iter()
                    .map(|(pat, s)| (pat.map_token(&mut f), s))
                    .collect(),
                help,
            },
            Self::Validation(validation) => ParserError::Validation(validation),
        }
    }
}

impl<'a, I: Input<'a>> Error<'a, I> for ParserError<'a, I::Span, I::Token>
where
    I::Token: PartialEq + Clone,
    I::Span: prism_model::Span,
{
    fn merge(mut self, mut other: Self) -> Self {
        if let (
            Self::ExpectedFound { expected, .. },
            Self::ExpectedFound {
                expected: expected_other,
                ..
            },
        ) = (&mut self, &mut other)
        {
            expected.append(expected_other);
        }
        self
    }
}

impl<'a, I: Input<'a>, L> LabelError<'a, I, L> for ParserError<'a, I::Span, I::Token>
where
    I::Token: PartialEq + Clone,
    I::Span: prism_model::Span,
    L: Into<RichPattern<'a, I::Token>>,
{
    fn expected_found<Iter: IntoIterator<Item = L>>(
        expected: Iter,
        found: Option<MaybeRef<'a, I::Token>>,
        span: I::Span,
    ) -> Self {
        Self::ExpectedFound {
            span,
            expected: expected.into_iter().map(|e| e.into()).collect(),
            found,
            contexts: Vec::new(),
            help: None,
        }
    }

    fn label_with(&mut self, label: L) {
        match &mut *self {
            Self::ExpectedFound { expected, .. } => {
                expected.clear();
                expected.push(label.into());
            }
            _ => (),
        }
    }

    fn in_context(&mut self, label: L, span: I::Span) {
        if let Self::ExpectedFound { contexts, .. } = self {
            let label = label.into();
            contexts.push((label, span))
        }
    }
}
