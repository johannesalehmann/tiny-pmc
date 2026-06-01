#[cfg(doc)]
use crate::Model;
use crate::module::AddModuleError;
use crate::spans::Span;
use crate::{Expression, Identifier, Module};

impl<S: Span> super::Model<Identifier<S>, S, Expression<Identifier<S>, S>, Identifier<S>> {
    /// Expands every renamed module into a full module.
    ///
    /// After this function, [`Model::renamed_modules`] is empty, with every entry transformed into
    /// a normal module according to the semantics described in
    /// [the PRISM manual](https://www.prismmodelchecker.org/manual/ThePRISMLanguage/ModuleRenaming).
    ///
    /// # Panics
    ///
    /// If the model contains any formulas, this panics. This is because module renaming is done
    /// after formula expansion, so renaming a module with formulas produces incorrect semantics.
    ///
    /// # Example
    ///
    /// A renamed module is a copy of an existing module in which variables and action labels are
    /// substituted according to its renaming rules. This lets two modules refer to each other.
    /// Consider the following model, where each counter advances only while it is not ahead of the
    /// other.
    ///
    /// ```prism
    /// mdp
    /// formula ahead = c > d;
    /// module counter
    ///     c: [0..3] init 0;
    ///     [] !ahead -> 1.0: (c'=c+1);
    /// endmodule
    /// module counter2 = counter [ c = d, d = c ] endmodule
    /// ```
    ///
    /// [`Model::substitute_formulas`] must be called first, since this function panics on a model
    /// that still contains formulas. Substituting formulas inlines `ahead` into `counter`:
    ///
    /// ```prism
    /// mdp
    /// module counter
    ///     c: [0..3] init 0;
    ///     [] !(c > d) -> 1.0: (c'=c+1);
    /// endmodule
    /// module counter2 = counter [ c = d, d = c ] endmodule
    /// ```
    ///
    /// Calling `expand_renamed_modules()` then results in the following model:
    ///
    /// ```prism
    /// mdp
    /// module counter
    ///     c: [0..3] init 0;
    ///     [] !(c > d) -> 1.0: (c'=c+1);
    /// endmodule
    /// module counter2
    ///     d: [0..3] init 0;
    ///     [] !(d > c) -> 1.0: (d'=d+1);
    /// endmodule
    /// ```
    pub fn expand_renamed_modules(&mut self) -> Result<(), ModuleExpansionError<S>> {
        assert!(
            self.formulas.formulas.is_empty(),
            "Cannot expand renamed modules in a model that contains formulas"
        );
        let renamed_modules = std::mem::replace(&mut self.renamed_modules, Vec::new());
        for renamed_module in renamed_modules {
            let source_index = self
                .modules
                .get_index_by_name(&renamed_module.old_name)
                .ok_or(ModuleExpansionError::RenamingSourceDoesNotExist {
                    old_name: renamed_module.old_name.clone(),
                    new_name: renamed_module.new_name.clone(),
                    renaming_rule: renamed_module.span.clone(),
                })?;
            let source_module = &self.modules.get(source_index).unwrap();
            let module = Module {
                name: renamed_module.new_name.clone(),
                commands: source_module
                    .commands
                    .iter()
                    .map(|c| c.renamed(&renamed_module.rename_rules))
                    .collect(),
                span: renamed_module.span.clone(),
            };
            self.variable_manager
                .add_renamed(
                    source_index,
                    self.modules.modules.len(),
                    &renamed_module.rename_rules,
                )
                .map_err(|e| ModuleExpansionError::MissingVariableRenaming {
                    variable_name: e.variable_name,
                    original_definition: e.original_definition_span,
                    rename_rule: renamed_module.span.clone(),
                })?;

            match self.modules.add(module) {
                Ok(_) => Ok(()),
                Err(AddModuleError::ModuleExists { index }) => {
                    Err(ModuleExpansionError::DuplicateModule {
                        name: renamed_module.new_name.clone(),
                        original_module: self.modules.get(index).unwrap().span.clone(),
                        rename_rule: renamed_module.span.clone(),
                    })
                }
            }?;
        }
        Ok(())
    }
}

/// An error produced by [`Model::expand_renamed_modules()`], indicating that the rename rules
/// did not adhere to the requirements outlined in
/// [the PRISM manual](https://www.prismmodelchecker.org/manual/ThePRISMLanguage/ModuleRenaming).
#[derive(Debug, PartialEq)]
pub enum ModuleExpansionError<S: Span> {
    /// The rename rule is trying to rename a module that does not exist.
    ///
    /// # Example
    ///
    /// The following example produces this error:
    ///
    /// ```prism
    /// mdp
    /// module counter
    ///     c: [0..3] init 0;
    ///     [] c < 3 -> 1.0: (c'=c+1);
    /// endmodule
    /// module counter2 = kounter [ c = d ] endmodule
    /// ```
    RenamingSourceDoesNotExist {
        /// The old name that is supposed to be the source of the renaming. No module with this
        /// name exists, which causes this error.
        ///
        /// # Example
        ///
        /// In this renamed module definition, `old_name` is the part marked by `^`.
        ///
        /// ```prism
        /// module counter2 = kounter [ c = d, d = c ] endmodule
        ///                   ^^^^^^^
        /// ```
        old_name: Identifier<S>,

        /// The name the module should get after renaming.
        ///
        /// # Example
        ///
        /// In this renamed module definition, `new_name` is the part marked by `^`.
        ///
        /// ```prism
        /// module counter2 = kounter [ c = d, d = c ] endmodule
        ///        ^^^^^^^^
        /// ```
        new_name: Identifier<S>,

        /// The [`Span`] of the renaming rule.
        renaming_rule: S,
    },

    /// A variable in the source module is not assigned a new name by the renaming rules.
    ///
    /// For every local variable of the source module, there must be a renaming rule.
    ///
    /// # Example
    ///
    /// The following example produces this error, as variable `c` is not assigned a new name for
    /// `counter2`.
    ///
    /// ```prism
    /// mdp
    /// module counter
    ///     b: bool init true;
    ///     c: [0..3] init 0;
    ///     [] c < 3 -> 1.0: (c'=c+1);
    /// endmodule
    /// module counter2 = counter [ b = d ] endmodule
    /// ```
    MissingVariableRenaming {
        /// The variable in the source module that is not renamed in the rename rules
        variable_name: Identifier<S>,

        /// [`Span`] of the entire source module
        original_definition: S,

        /// [`Span`] of the rename rule
        rename_rule: S,
    },

    /// A module with the same name already exists.
    ///
    /// # Example
    ///
    /// The following example produces this error, as the renamed module `counter2` has the same
    /// name as an already existing module.
    ///
    /// ```prism
    /// mdp
    /// module counter
    ///     c: [0..3] init 0;
    ///     [] c < 3 -> 1.0: (c'=c+1);
    /// endmodule
    /// module counter2
    ///     d: [0..3] init 0;
    ///     [] d < 3 -> 1.0: (d'=d+1);
    /// endmodule
    /// module counter2 = counter [ c = e ] endmodule
    /// ```
    DuplicateModule {
        /// The duplicate name
        name: Identifier<S>,
        /// The [`Span`] of the entire existing module with the same name
        original_module: S,
        /// The [`Span`] of the entire renamed module that is a duplicate
        rename_rule: S,
    },
}
