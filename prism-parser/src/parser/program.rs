use super::{
    E, const_parser, formula_parser, global_variable_declaration_parser, init_constraint_parser,
    label_parser, model_type_parser, module_parser, renamed_module_parser, rewards_parser,
};
use crate::error::ElementKind;
use crate::{ParserSpan, PrismParserError, PrismParserValidationError, Token};
use chumsky::IterParser;
use chumsky::Parser;
use chumsky::input::ValueInput;
use prism_model::{
    Expression, Identifier, ModelType, ModuleManager, Span, VariableAddError, VariableInfo,
    VariableManager,
};

pub fn program_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    prism_model::Model<
        Identifier<ParserSpan>,
        ParserSpan,
        Expression<Identifier<ParserSpan>, ParserSpan>,
        Identifier<ParserSpan>,
    >,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    program_element_parser()
        .repeated()
        .collect::<Vec<_>>()
        .validate(|els, e, emitter| build_program_from_type_and_elements(els, e.span(), emitter))
}

fn add_or_emit_variable(
    manager: &mut VariableManager<ParserSpan, Expression<Identifier<ParserSpan>, ParserSpan>>,
    variable: VariableInfo<ParserSpan, Expression<Identifier<ParserSpan>, ParserSpan>>,
    kind: ElementKind,
    emitter: &mut chumsky::input::Emitter<PrismParserError<ParserSpan, Token>>,
) {
    let span = variable.span;
    match manager.add_variable(variable) {
        Ok(_) => {}
        Err(err) => match err {
            VariableAddError::VariableExists { reference } => emitter.emit(
                PrismParserValidationError::DuplicateElement {
                    previous_occurrence: manager.get(&reference).unwrap().span,
                    new_definition: span,
                    kind,
                }
                .into(),
            ),
        },
    }
}

fn build_program_from_type_and_elements<'a>(
    elements: Vec<ProgramElement>,
    span: ParserSpan,
    emitter: &mut chumsky::input::Emitter<PrismParserError<ParserSpan, Token>>,
) -> prism_model::Model<
    Identifier<ParserSpan>,
    ParserSpan,
    Expression<Identifier<ParserSpan>, ParserSpan>,
    Identifier<ParserSpan>,
> {
    let mut model_type = Option::None;
    let mut modules = ModuleManager::new();
    let mut renamed_modules = Vec::new();
    let mut variables = VariableManager::new();
    let mut labels = prism_model::LabelManager::new();
    let mut formulas = prism_model::FormulaManager::new();
    let mut init_constraint = None;
    let mut rewards = prism_model::RewardsManager::new();

    for element in elements {
        match element {
            ProgramElement::Module(m, m_vars) => {
                let span = m.span.clone();

                match modules.add(m) {
                    Ok(module_index) => {
                        for mut variable in m_vars {
                            variable.scope = Some(module_index);
                            add_or_emit_variable(
                                &mut variables,
                                variable,
                                ElementKind::LocalVar,
                                emitter,
                            );
                        }
                    }
                    Err(prism_model::AddModuleError::ModuleExists { index }) => emitter.emit(
                        PrismParserValidationError::DuplicateElement {
                            previous_occurrence: modules.get(index).unwrap().span,
                            new_definition: span,
                            kind: ElementKind::Module,
                        }
                        .into(),
                    ),
                }
            }
            ProgramElement::RenamedModule(m) => renamed_modules.push(m),
            ProgramElement::Const(c) => {
                add_or_emit_variable(&mut variables, c, ElementKind::Const, emitter);
            }
            ProgramElement::GlobalVariable(v) => {
                add_or_emit_variable(&mut variables, v, ElementKind::GlobalVar, emitter);
            }
            ProgramElement::Label(l) => {
                let span = l.span;
                match labels.add_label(l) {
                    Ok(_) => {}
                    Err(prism_model::AddLabelError::LabelExists { index }) => {
                        let previous = labels.get(index).unwrap();
                        emitter.emit(
                            PrismParserValidationError::DuplicateElement {
                                previous_occurrence: previous.span,
                                new_definition: span,
                                kind: ElementKind::Label,
                            }
                            .into(),
                        )
                    }
                }
            }
            ProgramElement::Formula(f) => {
                let span = f.span;
                match formulas.add_formula(f) {
                    Ok(_) => {}
                    Err(prism_model::AddFormulaError::FormulaExists { index }) => {
                        let previous = formulas.get(index).unwrap();
                        emitter.emit(
                            PrismParserValidationError::DuplicateElement {
                                previous_occurrence: previous.span,
                                new_definition: span,
                                kind: ElementKind::Formula,
                            }
                            .into(),
                        )
                    }
                }
            }
            ProgramElement::ModelType(t) => match model_type {
                None => model_type = Some(t),
                Some(first) => {
                    emitter.emit(
                        PrismParserValidationError::DuplicateModelType {
                            first_occurrence: *first.span(),
                            duplicate_occurrence: *t.span(),
                        }
                        .into(),
                    );
                }
            },
            ProgramElement::InitConstraint(i, span) => match &init_constraint {
                None => init_constraint = Some((i, span)),
                Some((first, first_span)) => {
                    emitter.emit(
                        PrismParserValidationError::DuplicateInitConstraint {
                            first_occurrence: *first_span,
                            first_occurrence_inner: *first.span(),
                            duplicate_occurrence: span,
                            duplicate_occurrence_inner: *i.span(),
                        }
                        .into(),
                    );
                }
            },
            ProgramElement::Rewards(r) => {
                let span = r.span;
                match rewards.add(r) {
                    Ok(_) => {}
                    Err(prism_model::AddRewardsError::RewardsExist { index }) => {
                        let previous = rewards.get(index).unwrap();
                        emitter.emit(
                            PrismParserValidationError::DuplicateElement {
                                previous_occurrence: previous.span,
                                new_definition: span,
                                kind: ElementKind::Reward,
                            }
                            .into(),
                        )
                    }
                }
            }
        }
    }

    let model_type = match model_type {
        None => {
            emitter.emit(PrismParserValidationError::MissingModelType.into());
            ModelType::Mdp(ParserSpan::empty())
        }
        Some(model_type) => model_type,
    };

    prism_model::Model::from_components(
        model_type,
        variables,
        formulas,
        modules,
        renamed_modules,
        init_constraint.map(|(i, _)| i),
        labels,
        rewards,
        span,
    )
}

enum ProgramElement {
    ModelType(ModelType<ParserSpan>),
    Const(prism_model::VariableInfo<ParserSpan, Expression<Identifier<ParserSpan>, ParserSpan>>),
    Label(prism_model::Label<ParserSpan, Expression<Identifier<ParserSpan>, ParserSpan>>),
    Module(
        prism_model::Module<
            Identifier<ParserSpan>,
            ParserSpan,
            Expression<Identifier<ParserSpan>, ParserSpan>,
            Identifier<ParserSpan>,
        >,
        Vec<VariableInfo<ParserSpan, Expression<Identifier<ParserSpan>, ParserSpan>>>,
    ),
    RenamedModule(prism_model::RenamedModule<ParserSpan>),
    GlobalVariable(
        prism_model::VariableInfo<ParserSpan, Expression<Identifier<ParserSpan>, ParserSpan>>,
    ),
    Formula(prism_model::Formula<ParserSpan, Expression<Identifier<ParserSpan>, ParserSpan>>),
    InitConstraint(
        prism_model::Expression<Identifier<ParserSpan>, ParserSpan>,
        ParserSpan,
    ),
    Rewards(
        prism_model::Rewards<
            ParserSpan,
            Expression<Identifier<ParserSpan>, ParserSpan>,
            Identifier<ParserSpan>,
        >,
    ),
}

fn program_element_parser<'a, 'b, I>() -> impl Parser<'a, I, ProgramElement, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    model_type_parser()
        .map(ProgramElement::ModelType)
        .or(module_parser().map(|(m, v)| ProgramElement::Module(m, v)))
        .or(renamed_module_parser().map(ProgramElement::RenamedModule))
        .or(const_parser().map(ProgramElement::Const))
        .or(label_parser().map(ProgramElement::Label))
        .or(formula_parser().map(ProgramElement::Formula))
        .or(global_variable_declaration_parser().map(ProgramElement::GlobalVariable))
        .or(rewards_parser().map(ProgramElement::Rewards))
        .or(init_constraint_parser().map(|(i, e)| ProgramElement::InitConstraint(i, e)))
}
