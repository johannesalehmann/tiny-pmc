use chumsky::error::{Error, LabelError, RichPattern};
use chumsky::input::Input;
use chumsky::util::MaybeRef;
use prism_model::{
    CyclicDependency, Expression, Identifier, ModuleExpansionError, UnknownVariableError,
};

#[derive(Debug, PartialEq)]
pub enum ParserError<'a, S: prism_model::Span, T> {
    ExpectedFound {
        span: S,
        expected: Vec<RichPattern<'a, T>>,
        found: Option<MaybeRef<'a, T>>,
        contexts: Vec<(RichPattern<'a, T>, S)>,
        help: Option<String>,
    },
    Validation(ValidationError<S>),
}

#[derive(Debug, PartialEq)]
pub enum ValidationError<S: prism_model::Span> {
    UnsupportedModelType {
        model_type: &'static str,
        span: S,
    },
    MissingModelType,
    DuplicateModelType {
        first_occurrence: S,
        duplicate_occurrence: S,
    },
    DuplicateInitConstraint {
        first_occurrence: S,
        first_occurrence_inner: S,
        duplicate_occurrence: S,
        duplicate_occurrence_inner: S,
    },
    InvalidRangeForScope {
        span: S,
        range: prism_model::VariableRange<S, Expression<Identifier<S>, S>>,
        kind: prism_model::InvalidRangeForScopeKind,
    },
    DuplicateElement {
        previous_occurrence: S,
        new_definition: S,
        kind: ElementKind,
    },
    InvalidIdentifierName {
        span: S,
        reason: prism_model::InvalidName,
    },
    CyclicFormulaDependency {
        cycle: prism_model::CyclicDependency<S>,
    },
    ModuleExpansion {
        error: prism_model::ModuleExpansionError<S>,
    },
    UnknownVariable {
        identifier: Identifier<S>,
    },
}

#[derive(Debug, PartialEq)]
pub enum ElementKind {
    GlobalVar,
    LocalVar,
    Const,
    Label,
    Formula,
    Reward,
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

// impl<'a, E: Into<ValidationError<S>>, S: prism_model::Span, T> Into<ParserError<'a, S, T>> for E {
//     fn into(self) -> ParserError<'a, S, T> {
//         Into::<ValidationError<E>>::into(self).into()
//     }
// }
//
// impl<S: prism_model::Span> Into<ValidationError<S>> for CyclicDependency<S> {
//     fn into(self) -> ValidationError<S> {
//         ValidationError::CyclicFormulaDependency { cycle: self }
//     }
// }

impl<'a, S: prism_model::Span, T> Into<ParserError<'a, S, T>> for ValidationError<S> {
    fn into(self) -> ParserError<'a, S, T> {
        ParserError::Validation(self)
    }
}

impl<'a, S: prism_model::Span, T> ParserError<'a, S, T> {
    pub fn into_owned<'b>(self) -> ParserError<'b, S, T>
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

    pub fn map_token<U, F: FnMut(T) -> U>(self, mut f: F) -> ParserError<'a, S, U>
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
