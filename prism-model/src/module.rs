use crate::spans::{FullSpan, Span};
use crate::{
    Command, Displayable, Expression, Identifier, VariableManager, VariablePrintingStyle,
    VariableReference,
};
use std::fmt::{Display, Formatter};

pub type ModuleManagerNamedVars<S: Span = FullSpan, A = Identifier<S>> =
    ModuleManager<Identifier<S>, S, Expression<Identifier<S>, S>, A>;
pub struct ModuleManager<
    V = VariableReference,
    S: Span = FullSpan,
    E = Expression<V, S>,
    A = Identifier<S>,
> {
    pub modules: Vec<Module<V, S, E, A>>,
}

impl<V, S: Span, E, A> ModuleManager<V, S, E, A> {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Module<V, S, E, A>> {
        self.modules.get(index)
    }

    pub fn get_index_by_name(&self, name: &Identifier<S>) -> Option<usize> {
        self.modules
            .iter()
            .enumerate()
            .find(|(_, m)| &m.name == name)
            .map(|(i, _)| i)
    }

    pub fn get_by_name(&self, name: &Identifier<S>) -> Option<&Module<V, S, E, A>> {
        self.modules.iter().find(|m| &m.name == name)
    }

    pub fn add(&mut self, module: Module<V, S, E, A>) -> Result<usize, AddModuleError> {
        for (index, other_module) in self.modules.iter().enumerate() {
            if other_module.name == module.name {
                return Err(AddModuleError::ModuleExists { index });
            }
        }
        let index = self.modules.len();
        self.modules.push(module);
        Ok(index)
    }
}
impl<V, S: Span, A> ModuleManager<V, S, Expression<V, S>, A> {
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> ModuleManager<V, S2, Expression<V, S2>, A> {
        let mut module_manager = ModuleManager {
            modules: Vec::with_capacity(self.modules.len()),
        };
        for module in self.modules {
            module_manager.modules.push(module.map_span(map))
        }

        module_manager
    }
}

#[derive(Debug)]
pub enum AddModuleError {
    ModuleExists { index: usize },
}

pub type ModuleNamedVars<S: Span = FullSpan, A = Identifier<S>> =
    Module<Identifier<S>, S, Expression<Identifier<S>, S>, A>;
pub struct Module<
    V = VariableReference,
    S: Span = FullSpan,
    E = Expression<V, S>,
    A = Identifier<S>,
> {
    pub name: Identifier<S>,
    pub commands: Vec<Command<V, S, E, A>>,
    pub span: S,
}

impl<V, S: Span, E, A> Module<V, S, E, A> {
    pub fn new(name: Identifier<S>) -> Self {
        Self::new_spanned(name, S::empty())
    }
    pub fn new_spanned(name: Identifier<S>, span: S) -> Self {
        Self {
            name,
            commands: Vec::new(),
            span,
        }
    }
}
impl<V, S: Span, A> Module<V, S, Expression<V, S>, A> {
    pub fn map_span<S2: Span, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> Module<V, S2, Expression<V, S2>, A> {
        Module {
            name: self.name.map_span(map),
            commands: self.commands.into_iter().map(|c| c.map_span(map)).collect(),
            span: map(self.span),
        }
    }
}

impl<V, S: Span, E, A> crate::private::Sealed for Module<V, S, E, A> {}
impl<'a, 'b, Ctx, V: Displayable<Ctx>, S: Span, E: Displayable<Ctx>, A: Display>
    Displayable<(usize, &VariableManager<S, E>, &Ctx)> for Module<V, S, E, A>
{
    fn fmt_internal(
        &self,
        f: &mut Formatter<'_>,
        (own_index, variable_manager, context): &(usize, &VariableManager<S, E>, &Ctx),
    ) -> std::fmt::Result {
        writeln!(f, "module {}", self.name)?;
        write!(
            f,
            "{}",
            variable_manager.displayable(&(
                VariablePrintingStyle::LocalVar {
                    module_index: *own_index
                },
                context
            ))
        )?;
        for command in &self.commands {
            writeln!(f, "    {}", command.displayable(context))?;
        }
        writeln!(f, "endmodule")
    }
}

#[derive(Clone)]
pub struct RenamedModule<S: Span = FullSpan> {
    pub old_name: Identifier<S>,
    pub new_name: Identifier<S>,
    pub rename_rules: RenameRules<S>,
    pub span: S,
}

impl<S: Span> RenamedModule<S> {
    pub fn new(
        old_name: Identifier<S>,
        new_name: Identifier<S>,
        rename_rules: RenameRules<S>,
        span: S,
    ) -> Self {
        Self {
            old_name,
            new_name,
            rename_rules,
            span,
        }
    }

    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> RenamedModule<S2> {
        RenamedModule {
            old_name: self.old_name.map_span(map),
            new_name: self.new_name.map_span(map),
            rename_rules: RenameRules {
                rules: self
                    .rename_rules
                    .rules
                    .into_iter()
                    .map(|i| i.map_span(map))
                    .collect(),
            },
            span: map(self.span),
        }
    }
}

impl<S: Span> Display for RenamedModule<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "module {} = {} [", self.new_name, self.old_name)?;
        let mut is_first = true;
        for rename_rule in &self.rename_rules.rules {
            if !is_first {
                write!(f, ", ")?;
            }
            is_first = false;
            write!(f, "{}", rename_rule)?;
        }
        write!(f, "] endmodule")
    }
}

#[derive(Clone)]
pub struct RenameRules<S: Span> {
    pub rules: Vec<RenameRule<S>>,
}

impl<S: Span> RenameRules<S> {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn get_renaming(&self, old_name: &Identifier<S>) -> Option<Identifier<S>> {
        for rule in &self.rules {
            if &rule.old_name == old_name {
                return Some(rule.new_name.clone());
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct RenameRule<S: Span> {
    pub old_name: Identifier<S>,
    pub new_name: Identifier<S>,
    pub span: S,
}
impl<S: Span> RenameRule<S> {
    pub fn new(old_name: Identifier<S>, new_name: Identifier<S>, span: S) -> Self {
        Self {
            old_name,
            new_name,
            span,
        }
    }

    pub fn map_span<S2: Span, F: Fn(S) -> S2>(self, map: &F) -> RenameRule<S2> {
        RenameRule {
            old_name: self.old_name.map_span(map),
            new_name: self.new_name.map_span(map),
            span: map(self.span),
        }
    }
}
impl<S: Span> Display for RenameRule<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.old_name, self.new_name)
    }
}
