use crate::expressions::UnknownVariableError;
use crate::module::RenameRules;
use crate::spans::{FullSpan, Span};
use crate::{Displayable, Expression, Identifier};
use std::fmt::Formatter;

pub type VariableManagerNamedVars<S: Span = FullSpan> =
    VariableManager<S, Expression<Identifier<S>, S>>;
pub struct VariableManager<S: Span = FullSpan, E = Expression<VariableReference, S>> {
    pub variables: Vec<VariableInfo<S, E>>,
}

impl<S: Span, E> VariableManager<S, E> {
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
        }
    }

    pub fn add_variable(
        &mut self,
        variable_info: VariableInfo<S, E>,
    ) -> Result<VariableReference, VariableAddError> {
        if let Some(existing_variable) = self.get_reference(&variable_info.name) {
            Err(VariableAddError::VariableExists {
                reference: existing_variable,
            })
        } else {
            let index = VariableReference::new(self.variables.len());
            self.variables.push(variable_info);
            Ok(index)
        }
    }

    pub fn get_reference(&self, name: &Identifier<S>) -> Option<VariableReference> {
        for (index, var) in self.variables.iter().enumerate() {
            if &var.name == name {
                return Some(VariableReference::new(index));
            }
        }
        None
    }
    pub fn get_reference_by_str(&self, name: &str) -> Option<VariableReference> {
        for (index, var) in self.variables.iter().enumerate() {
            if &var.name.name == name {
                return Some(VariableReference::new(index));
            }
        }
        None
    }

    pub fn get(&self, reference: &VariableReference) -> Option<&VariableInfo<S, E>> {
        self.variables.get(reference.index)
    }
}

impl<S: Span, E> crate::private::Sealed for VariableManager<S, E> {}
impl<Ctx, E: Displayable<Ctx>, S: Span> Displayable<(VariablePrintingStyle, &Ctx)>
    for VariableManager<S, E>
{
    fn fmt_internal(
        &self,
        f: &mut Formatter<'_>,
        (printing_style, context): &(VariablePrintingStyle, &Ctx),
    ) -> std::fmt::Result {
        for variable in &self.variables {
            if !printing_style.accepts(variable) {
                continue;
            }
            if printing_style == &VariablePrintingStyle::Const {
                write!(
                    f,
                    "const {} {}",
                    variable.range.displayable(context),
                    variable.name
                )?;
                if let Some(initial) = &variable.initial_value {
                    write!(f, " = {}", initial.displayable(context))?;
                }
            } else {
                if printing_style == &VariablePrintingStyle::GlobalVar {
                    write!(f, "global ")?;
                } else {
                    write!(f, "    ")?;
                }
                write!(
                    f,
                    "{} : {}",
                    variable.name,
                    variable.range.displayable(context)
                )?;
                if let Some(initial) = &variable.initial_value {
                    write!(f, " init {}", initial.displayable(context))?;
                }
            }
            writeln!(f, ";")?;
        }
        if self.variables.len() > 0 {
            writeln!(f, "")?;
        }
        Ok(())
    }
}

impl<V, S: Span> VariableManager<S, Expression<V, S>> {
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> VariableManager<S2, Expression<V, S2>> {
        VariableManager {
            variables: self
                .variables
                .into_iter()
                .map(|v| v.map_span(map))
                .collect(),
        }
    }
}
impl<S: Span> VariableManager<S, Expression<Identifier<S>, S>> {
    pub fn add_renamed(
        &mut self,
        old_module_index: usize,
        new_module_index: usize,
        rename_rules: &RenameRules<S>,
    ) -> Result<VariableManager<S, Expression<Identifier<S>, S>>, MissingVariableRenaming<S>> {
        let variables = Vec::with_capacity(self.variables.len());
        for i in 0..self.variables.len() {
            let variable = &self.variables[i];
            if variable.is_constant || variable.scope != Some(old_module_index) {
                continue;
            }
            match rename_rules.get_renaming(&variable.name) {
                None => {
                    return Err(MissingVariableRenaming {
                        variable_name: variable.name.clone(),
                        original_definition: variable.span.clone(),
                    });
                }
                Some(renaming) => {
                    let new_var = VariableInfo {
                        range: variable.range.renamed(rename_rules),
                        name: renaming,
                        initial_value: variable
                            .initial_value
                            .as_ref()
                            .map(|i| i.renamed(rename_rules)),
                        span: variable.span.clone(),
                        is_constant: false,
                        scope: Some(new_module_index),
                    };
                    self.variables.push(new_var)
                }
            }
        }

        Ok(VariableManager { variables })
    }
}

pub enum VariableAddError {
    VariableExists { reference: VariableReference },
}

pub struct MissingVariableRenaming<S: Span> {
    pub variable_name: Identifier<S>,
    pub original_definition: S,
}

impl std::fmt::Debug for VariableAddError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableAddError::VariableExists { reference } => {
                write!(
                    f,
                    "Variable already exists in this manager (index {})",
                    reference.index
                )
            }
        }
    }
}

pub type VariableInfoNamedVars<S> = VariableInfo<S, Expression<Identifier<S>, S>>;

pub struct VariableInfo<S: Span = FullSpan, E = Expression<VariableReference, S>> {
    pub is_constant: bool,
    pub scope: Option<usize>,
    pub range: VariableRange<S, E>,
    pub name: Identifier<S>,
    pub initial_value: Option<E>,
    pub span: S,
}

impl<S: Span, E> VariableInfo<S, E> {
    pub fn global_var(name: Identifier<S>, range: VariableRange<S, E>) -> Self {
        Self::global_var_spanned(name, range, S::empty())
    }
    pub fn global_var_spanned(name: Identifier<S>, range: VariableRange<S, E>, span: S) -> Self {
        Self::new(name, range, false, None, span)
    }

    pub fn global_const(name: Identifier<S>, range: VariableRange<S, E>) -> Self {
        Self::global_const_spanned(name, range, S::empty())
    }
    pub fn global_const_spanned(name: Identifier<S>, range: VariableRange<S, E>, span: S) -> Self {
        Self::new(name, range, true, None, span)
    }

    pub fn local_var(name: Identifier<S>, range: VariableRange<S, E>, module: usize) -> Self {
        Self::local_var_spanned(name, range, module, S::empty())
    }

    pub fn local_var_spanned(
        name: Identifier<S>,
        range: VariableRange<S, E>,
        module: usize,
        span: S,
    ) -> Self {
        Self::new(name, range, true, Some(module), span)
    }

    pub fn new(
        name: Identifier<S>,
        range: VariableRange<S, E>,
        is_constant: bool,
        scope: Option<usize>,
        span: S,
    ) -> Self {
        Self {
            name,
            range,
            initial_value: None,
            span,
            is_constant,
            scope,
        }
    }

    pub fn with_initial_value(
        name: Identifier<S>,
        range: VariableRange<S, E>,
        is_constant: bool,
        scope: Option<usize>,
        initial_value: E,
    ) -> Self {
        Self::with_initial_value_spanned(name, range, is_constant, scope, initial_value, S::empty())
    }

    pub fn with_initial_value_spanned(
        name: Identifier<S>,
        range: VariableRange<S, E>,
        is_constant: bool,
        scope: Option<usize>,
        initial_value: E,
        span: S,
    ) -> Self {
        Self {
            name,
            range,
            initial_value: Some(initial_value),
            span,
            is_constant,
            scope,
        }
    }

    pub fn with_optional_initial_value(
        name: Identifier<S>,
        range: VariableRange<S, E>,
        is_constant: bool,
        scope: Option<usize>,
        initial_value: Option<E>,
        span: S,
    ) -> Self {
        Self {
            name,
            range,
            initial_value,
            span,
            is_constant,
            scope,
        }
    }
}
impl<V, S: Span> VariableInfo<S, Expression<V, S>> {
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> VariableInfo<S2, Expression<V, S2>> {
        VariableInfo {
            is_constant: self.is_constant,
            scope: self.scope,
            range: self.range.map_span(map),
            name: self.name.map_span(map),
            initial_value: self.initial_value.map(|i| i.map_span(map)),
            span: map(self.span),
        }
    }
}

pub type VariableRangeNamedVars<S: Span = FullSpan> =
    VariableRange<S, Expression<Identifier<S>, S>>;

#[derive(Debug, PartialEq, Clone)]
pub enum VariableRange<S: Span = FullSpan, E = Expression<VariableReference, S>> {
    BoundedInt { min: E, max: E, span: S },
    UnboundedInt { span: S },
    Boolean { span: S },
    Float { span: S },
}

impl<S: Span, E> VariableRange<S, E> {
    pub fn bounded_int(min: E, max: E) -> Self {
        Self::bounded_int_spanned(min, max, S::empty())
    }
    pub fn bounded_int_spanned(min: E, max: E, span: S) -> Self {
        Self::BoundedInt { min, max, span }
    }
    pub fn unbounded_int() -> Self {
        Self::unbounded_int_spanned(S::empty())
    }
    pub fn unbounded_int_spanned(span: S) -> Self {
        Self::UnboundedInt { span }
    }
    pub fn bool() -> Self {
        Self::bool_spanned(S::empty())
    }
    pub fn bool_spanned(span: S) -> Self {
        Self::Boolean { span }
    }
    pub fn float() -> Self {
        Self::float_spanned(S::empty())
    }
    pub fn float_spanned(span: S) -> Self {
        Self::Float { span }
    }

    pub fn is_legal_for_constant(&self) -> bool {
        match self {
            VariableRange::BoundedInt { .. } => false,
            VariableRange::UnboundedInt { .. } => true,
            VariableRange::Boolean { .. } => true,
            VariableRange::Float { .. } => true,
        }
    }
    pub fn is_legal_for_variable(&self) -> bool {
        match self {
            VariableRange::BoundedInt { .. } => true,
            VariableRange::UnboundedInt { .. } => true,
            VariableRange::Boolean { .. } => true,
            VariableRange::Float { .. } => false,
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            VariableRange::BoundedInt { .. } => "bounded int",
            VariableRange::UnboundedInt { .. } => "int",
            VariableRange::Boolean { .. } => "bool",
            VariableRange::Float { .. } => "double",
        }
    }

    pub fn span(&self) -> &S {
        match self {
            VariableRange::BoundedInt { span, .. } => span,
            VariableRange::UnboundedInt { span } => span,
            VariableRange::Boolean { span } => span,
            VariableRange::Float { span } => span,
        }
    }
}

impl<V, S: Span> VariableRange<S, Expression<V, S>> {
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> VariableRange<S2, Expression<V, S2>> {
        match self {
            VariableRange::BoundedInt { min, max, span } => VariableRange::BoundedInt {
                min: min.map_span(map),
                max: max.map_span(map),
                span: map(span),
            },
            VariableRange::UnboundedInt { span } => VariableRange::UnboundedInt { span: map(span) },
            VariableRange::Boolean { span } => VariableRange::Boolean { span: map(span) },
            VariableRange::Float { span } => VariableRange::Float { span: map(span) },
        }
    }
}

impl<S: Span, E> crate::private::Sealed for VariableRange<S, E> {}
impl<Ctx, E: Displayable<Ctx>, S: Span> Displayable<Ctx> for VariableRange<S, E> {
    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result {
        match self {
            VariableRange::BoundedInt { min, max, .. } => {
                write!(
                    f,
                    "[{}..{}]",
                    min.displayable(context),
                    max.displayable(context)
                )
            }
            VariableRange::UnboundedInt { .. } => {
                write!(f, "int")
            }
            VariableRange::Boolean { .. } => {
                write!(f, "bool")
            }
            VariableRange::Float { .. } => {
                write!(f, "double")
            }
        }
    }
}
impl<S: Span> VariableRange<S, Expression<Identifier<S>, S>> {
    pub fn renamed(&self, rename_rules: &RenameRules<S>) -> Self {
        match self {
            VariableRange::BoundedInt { min, max, span } => VariableRange::BoundedInt {
                min: min.renamed(rename_rules),
                max: max.renamed(rename_rules),
                span: span.clone(),
            },
            VariableRange::UnboundedInt { span } => {
                VariableRange::UnboundedInt { span: span.clone() }
            }
            VariableRange::Boolean { span } => VariableRange::Boolean { span: span.clone() },
            VariableRange::Float { span } => VariableRange::Float { span: span.clone() },
        }
    }

    pub fn replace_identifiers_by_variable_indices(
        &self,
        variable_manager: &VariableManager<S, Expression<Identifier<S>, S>>,
    ) -> Result<VariableRange<S, Expression<VariableReference, S>>, Vec<UnknownVariableError<S>>>
    {
        match self {
            VariableRange::BoundedInt { min, max, span } => {
                let mut errors = Vec::new();
                let min = min
                    .clone()
                    .replace_identifiers_by_variable_indices(variable_manager);
                let max = max
                    .clone()
                    .replace_identifiers_by_variable_indices(variable_manager);
                if let Err(err) = &min {
                    errors.extend_from_slice(&err[..]);
                }
                if let Err(err) = &max {
                    errors.extend_from_slice(&err[..]);
                }
                if let (Ok(min), Ok(max)) = (min, max) {
                    Ok(VariableRange::BoundedInt {
                        min,
                        max,
                        span: span.clone(),
                    })
                } else {
                    Err(errors)
                }
            }
            VariableRange::UnboundedInt { span } => {
                Ok(VariableRange::UnboundedInt { span: span.clone() })
            }
            VariableRange::Boolean { span } => Ok(VariableRange::Boolean { span: span.clone() }),
            VariableRange::Float { span } => Ok(VariableRange::Float { span: span.clone() }),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub struct VariableReference {
    pub index: usize,
}

impl VariableReference {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

impl std::fmt::Debug for VariableReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "var_{}", self.index)
    }
}

impl crate::private::Sealed for VariableReference {}
impl<S: Span> Displayable<VariableManager<S, Expression<VariableReference, S>>>
    for VariableReference
{
    fn fmt_internal(
        &self,
        f: &mut Formatter<'_>,
        context: &VariableManager<S, Expression<VariableReference, S>>,
    ) -> std::fmt::Result {
        let variable = context.get(&self).unwrap();
        write!(f, "{}", variable.name)
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum VariablePrintingStyle {
    Const,
    GlobalVar,
    LocalVar { module_index: usize },
}

impl VariablePrintingStyle {
    fn accepts<S: Span, E>(&self, variable: &VariableInfo<S, E>) -> bool {
        match self {
            VariablePrintingStyle::Const => variable.is_constant,
            VariablePrintingStyle::GlobalVar => !variable.is_constant && variable.scope.is_none(),
            VariablePrintingStyle::LocalVar { module_index } => {
                !variable.is_constant && variable.scope == Some(*module_index)
            }
        }
    }
}
