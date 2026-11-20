use super::{E, expression_parser, expression_parser_with, identifier_parser};
use crate::{ParserSpan, Token, ValidationError};
use chumsky::input::ValueInput;
use chumsky::prelude::{Recursive, just};
use chumsky::{IterParser, Parser, select};
use prism_model::{Expression, Identifier};
use probabilistic_properties::{
    Bound, BoundOperator, Coalition, NamedQuery, NoCoalition, NonDeterminismKind, PathFormula,
    Query, QueryEnum, RewardFormula, StateFormula,
};
use std::cell::Cell;
use std::collections::BTreeSet;

// TODO: Support property-file constants
// TODO: Support property-file labels

// We don't return NamedQueries here, as the name deduplication happens at a later stage (because
// there could be multiple query files that will be parsed separately).
pub fn named_queries_parser<'a, 'b, I>()
-> impl Parser<'a, I, Vec<NamedQuery<ParsedExpression, ParsedExpression, ParsedExpression>>, E<'a>>
+ Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    named_query_parser()
        .separated_by(just(Token::Semicolon))
        .collect()
        .then_ignore(just(Token::Semicolon).or_not())
}
pub fn named_query_parser<'a, 'b, I>()
-> impl Parser<'a, I, NamedQuery<ParsedExpression, ParsedExpression, ParsedExpression>, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    let name = just(Token::Quote)
        .ignore_then(identifier_parser())
        .then_ignore(just(Token::Quote))
        .then_ignore(just(Token::Colon));

    let query = query_parser().try_map(|query, span| {
        into_query_enum(query).map_err(|()| ValidationError::InconsistentCoalitions { span }.into())
    });

    name.map(|n| n.name)
        .or_not()
        .then(query)
        .map(|(n, q)| NamedQuery::new(n, q))
}

/// Determines whether the parsed `query` is a pCTL query (no operator has a coalition) or an rPATL
/// query (every operator has a coalition). Returns `Err(())` if the query mixes both.
fn into_query_enum(
    query: ParsedQuery,
) -> Result<QueryEnum<ParsedExpression, ParsedExpression, ParsedExpression>, ()> {
    let mut with_coalition = false;
    let mut without_coalition = false;
    let query = query.map_c(&mut |coalition: Option<Coalition>| {
        match coalition {
            Some(_) => with_coalition = true,
            None => without_coalition = true,
        }
        coalition
    });
    match (with_coalition, without_coalition) {
        (true, true) => Err(()),
        (true, false) => {
            Ok(QueryEnum::Rpatl(query.map_c(&mut |coalition| {
                coalition.expect("all operators have a coalition")
            })))
        }
        (false, _) => Ok(QueryEnum::Pctl(query.map_c(&mut |_| NoCoalition))),
    }
}
pub fn query_parser<'a, 'b, I>() -> impl Parser<'a, I, ParsedQuery, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    let mut state_formula_parser = Recursive::declare();
    let mut path_formula_parser = Recursive::declare();

    state_formula_parser.define(define_state_formula_parser(
        state_formula_parser.clone(),
        path_formula_parser.clone(),
    ));
    path_formula_parser.define(define_path_formula_parser(state_formula_parser.clone()));

    let probability_value = coalition_parser()
        .or_not()
        .then(probability_min_max())
        .then_ignore(just(Token::Equal))
        .then_ignore(just(Token::Questionmark))
        .then_ignore(just(Token::LeftSqBracket))
        .then(path_formula_parser)
        .then_ignore(just(Token::RightSqBracket))
        .map(
            |((coalition, non_determinism), path)| Query::ProbabilityValue {
                coalition,
                non_determinism,
                path,
            },
        );

    let state_formula = state_formula_parser.clone().map(|s| Query::StateFormula(s));

    let reward_bound = coalition_parser()
        .or_not()
        .then(reward_name_and_min_max())
        .then(bound_parser())
        .then_ignore(just(Token::LeftSqBracket))
        .then(reward_formula(state_formula_parser.clone()))
        .then_ignore(just(Token::RightSqBracket))
        .map(
            |(((coalition, (name, non_determinism)), bound), reward)| Query::RewardBound {
                coalition,
                non_determinism,
                name: name.map(|i| i.name), // TODO: Store span instead of name in reward
                bound,
                reward,
            },
        );

    let reward_value = coalition_parser()
        .or_not()
        .then(reward_name_and_min_max())
        .then_ignore(just(Token::Equal))
        .then_ignore(just(Token::Questionmark))
        .then_ignore(just(Token::LeftSqBracket))
        .then(reward_formula(state_formula_parser.clone()))
        .then_ignore(just(Token::RightSqBracket))
        .map(
            |((coalition, (name, non_determinism)), reward)| Query::RewardValue {
                coalition,
                non_determinism,
                name: name.map(|i| i.name), // TODO: Store span instead of name in reward
                reward,
            },
        );

    let time_bound = coalition_parser()
        .or_not()
        .then(time_min_max())
        .then(bound_parser())
        .then_ignore(just(Token::LeftSqBracket))
        .then(reward_formula(state_formula_parser.clone()))
        .then_ignore(just(Token::RightSqBracket))
        .map(
            |(((coalition, non_determinism), bound), reward)| Query::TimeBound {
                coalition,
                non_determinism,
                bound,
                reward,
            },
        );

    let time_value = coalition_parser()
        .or_not()
        .then(time_min_max())
        .then_ignore(just(Token::Equal))
        .then_ignore(just(Token::Questionmark))
        .then_ignore(just(Token::LeftSqBracket))
        .then(reward_formula(state_formula_parser.clone()))
        .then_ignore(just(Token::RightSqBracket))
        .map(|((coalition, non_determinism), reward)| Query::TimeValue {
            coalition,
            non_determinism,
            reward,
        });

    probability_value
        .or(state_formula)
        .or(reward_bound)
        .or(reward_value)
        .or(time_bound)
        .or(time_value)
}

/// Parses a coalition of the form `<<p1,p2,3>>`, where each player is given either by its name or
/// by its 1-based index.
pub fn coalition_parser<'a, 'b, I>() -> impl Parser<'a, I, Coalition, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    enum Player {
        Name(String),
        Index(usize),
    }

    let index = select! { Token::Integer(index) => index }.try_map(|index, span| {
        match index.parse::<usize>() {
            Ok(index) if index > 0 => Ok(index),
            _ => Err(ValidationError::InvalidPlayerIndex { span }.into()),
        }
    });

    let player = identifier_parser()
        .map(|identifier| Player::Name(identifier.name))
        .or(index.map(Player::Index));

    player
        .separated_by(just(Token::Comma))
        .collect::<Vec<_>>()
        .delimited_by(
            just(Token::LessThan).then(just(Token::LessThan)),
            just(Token::GreaterThan).then(just(Token::GreaterThan)),
        )
        .map(|players| {
            let mut names = BTreeSet::new();
            let mut indices = BTreeSet::new();
            for player in players {
                match player {
                    Player::Name(name) => {
                        names.insert(name);
                    }
                    Player::Index(index) => {
                        indices.insert(index);
                    }
                }
            }
            Coalition::new(names, indices)
        })
        .labelled("coalition")
}

pub fn probability_min_max<'a, 'b, I>()
-> impl Parser<'a, I, Option<NonDeterminismKind>, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    just(Token::P)
        .map(|_| None)
        .or(just(Token::PMax).map(|_| Some(NonDeterminismKind::Maximise)))
        .or(just(Token::PMin).map(|_| Some(NonDeterminismKind::Minimise)))
}
pub fn reward_name_and_min_max<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    (
        Option<prism_model::Identifier<ParserSpan>>,
        Option<NonDeterminismKind>,
    ),
    E<'a>,
> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    just(Token::R)
        .ignore_then(reward_name())
        .then(
            just(Token::Max)
                .map(|_| NonDeterminismKind::Maximise)
                .or(just(Token::Min).map(|_| NonDeterminismKind::Minimise))
                .or_not(),
        )
        .map(|(n, m)| (Some(n), m))
        .or(just(Token::R).map(|_| (None, None)))
        .or(just(Token::RMax).map(|_| (None, Some(NonDeterminismKind::Maximise))))
        .or(just(Token::RMin).map(|_| (None, Some(NonDeterminismKind::Minimise))))
}

pub fn reward_name<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::Identifier<ParserSpan>, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    just(Token::LeftCurlyBracket).ignore_then(
        just(Token::Quote)
            .ignore_then(identifier_parser())
            .then_ignore(just(Token::Quote))
            .then_ignore(just(Token::RightCurlyBracket)),
    )
}

pub fn time_min_max<'a, 'b, I>() -> impl Parser<'a, I, Option<NonDeterminismKind>, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    just(Token::T)
        .map(|_| None)
        .or(just(Token::TMax).map(|_| Some(NonDeterminismKind::Maximise)))
        .or(just(Token::TMin).map(|_| Some(NonDeterminismKind::Minimise)))
}
pub fn lra_min_max<'a, 'b, I>() -> impl Parser<'a, I, Option<NonDeterminismKind>, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    just(Token::LRA)
        .map(|_| None)
        .or(just(Token::LRAMax).map(|_| Some(NonDeterminismKind::Maximise)))
        .or(just(Token::LRAMin).map(|_| Some(NonDeterminismKind::Minimise)))
}

pub fn define_path_formula_parser<'a, 'b, I, SF: Parser<'a, I, ParsedStateFormula, E<'a>> + Clone>(
    state_formula_parser: SF,
) -> impl Parser<'a, I, ParsedPathFormula, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    let until = state_formula_parser
        .clone()
        .then_ignore(just(Token::Until))
        .then(state_formula_parser.clone())
        .map(|(before, after)| PathFormula::Until {
            before: Box::new(before),
            after: Box::new(after),
        });

    let eventually = just(Token::Finally)
        .ignore_then(state_formula_parser.clone())
        .map(|condition| PathFormula::Eventually {
            condition: Box::new(condition),
        });

    let bounded_until = state_formula_parser
        .clone()
        .then_ignore(just(Token::Until))
        .then(bound_parser())
        .then(state_formula_parser.clone())
        .map(|((before, bound), after)| PathFormula::BoundedUntil {
            before: Box::new(before),
            after: Box::new(after),
            bound,
        });

    let bounded_eventually = just(Token::Finally)
        .ignore_then(bound_parser())
        .then(state_formula_parser.clone())
        .map(|(bound, condition)| PathFormula::BoundedEventually {
            condition: Box::new(condition),
            bound,
        });

    let generally = just(Token::Generally)
        .ignore_then(state_formula_parser)
        .map(|condition| PathFormula::Generally {
            condition: Box::new(condition),
        });

    until
        .or(eventually)
        .or(bounded_until)
        .or(bounded_eventually)
        .or(generally)
}

pub fn define_state_formula_parser<
    'a,
    'b,
    I,
    SF: Parser<'a, I, ParsedStateFormula, E<'a>> + Clone + 'a,
    PF: Parser<'a, I, ParsedPathFormula, E<'a>> + Clone + 'a,
>(
    state_formula_parser: SF,
    path_formula_parser: PF,
) -> impl Parser<'a, I, ParsedStateFormula, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    let probability_bound = coalition_parser()
        .or_not()
        .then(probability_min_max())
        .then(bound_parser())
        .then_ignore(just(Token::LeftSqBracket))
        .then(path_formula_parser.clone())
        .then_ignore(just(Token::RightSqBracket))
        .map(
            |(((coalition, non_determinism), bound), path)| StateFormula::ProbabilityBound {
                coalition,
                non_determinism,
                bound,
                path: Box::new(path),
            },
        );

    let long_run_average = coalition_parser()
        .or_not()
        .then(lra_min_max())
        .then(bound_parser())
        .then(state_formula_parser)
        .map(
            |(((coalition, non_determinism), bound), states)| StateFormula::LongRunAverage {
                coalition,
                non_determinism,
                bound,
                states: Box::new(states),
            },
        );

    let atom = identifier_parser()
        .map(IdentifierOrStateFormula::Identifier)
        .or(probability_bound
            .or(long_run_average)
            .map(|formula| IdentifierOrStateFormula::StateFormula(Box::new(formula))));

    expression_parser_with(atom).try_map(|expression, _| {
        into_state_formula(expression)
            .map_err(|span| ValidationError::InvalidNestedStateFormula { span }.into())
    })
}

// Coalitions are parsed as `Option<Coalition>`. Whether a query consistently uses coalitions on
// all or on none of its operators is checked afterwards by `into_query_enum`.
type ParsedExpression = Expression<Identifier<ParserSpan>, ParserSpan>;
type ParsedQuery = Query<ParsedExpression, ParsedExpression, ParsedExpression, Option<Coalition>>;
type ParsedStateFormula =
    StateFormula<ParsedExpression, ParsedExpression, ParsedExpression, Option<Coalition>>;
type ParsedPathFormula =
    PathFormula<ParsedExpression, ParsedExpression, ParsedExpression, Option<Coalition>>;
type ParsedRewardFormula =
    RewardFormula<ParsedExpression, ParsedExpression, ParsedExpression, Option<Coalition>>;

// To represent nested state formulas, we abuse Expression::VarOrConst by storing this enum in it.
enum IdentifierOrStateFormula {
    Identifier(Identifier<ParserSpan>),
    StateFormula(Box<ParsedStateFormula>),
}

fn into_state_formula(
    expression: Expression<IdentifierOrStateFormula, ParserSpan>,
) -> Result<ParsedStateFormula, ParserSpan> {
    Ok(match expression {
        Expression::VarOrConst(IdentifierOrStateFormula::StateFormula(formula), _) => *formula,
        Expression::Negation(inner, span) => match into_state_formula(*inner)? {
            StateFormula::Expression(inner) => {
                StateFormula::Expression(Expression::Negation(Box::new(inner), span))
            }
            inner => StateFormula::Not(Box::new(inner)),
        },
        Expression::Conjunction(lhs, rhs, span) => {
            match (into_state_formula(*lhs)?, into_state_formula(*rhs)?) {
                (StateFormula::Expression(lhs), StateFormula::Expression(rhs)) => {
                    StateFormula::Expression(Expression::Conjunction(
                        Box::new(lhs),
                        Box::new(rhs),
                        span,
                    ))
                }
                (lhs, rhs) => StateFormula::And(Box::new(lhs), Box::new(rhs)),
            }
        }
        Expression::Disjunction(lhs, rhs, span) => {
            match (into_state_formula(*lhs)?, into_state_formula(*rhs)?) {
                (StateFormula::Expression(lhs), StateFormula::Expression(rhs)) => {
                    StateFormula::Expression(Expression::Disjunction(
                        Box::new(lhs),
                        Box::new(rhs),
                        span,
                    ))
                }
                (lhs, rhs) => StateFormula::Or(Box::new(lhs), Box::new(rhs)),
            }
        }
        expression => {
            let span = *expression.span();
            let contains_state_formula = Cell::new(false);
            let expression = expression.map_variable(&|atom| match atom {
                IdentifierOrStateFormula::Identifier(identifier) => identifier,
                IdentifierOrStateFormula::StateFormula(_) => {
                    contains_state_formula.set(true);
                    Identifier::new_unchecked("")
                }
            });
            if contains_state_formula.get() {
                return Err(span);
            }
            StateFormula::Expression(expression)
        }
    })
}

pub fn bound_parser<'a, 'b, I>()
-> impl Parser<'a, I, Bound<Expression<Identifier<ParserSpan>, ParserSpan>>, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    let operator = just(Token::LessThan)
        .map(|_| BoundOperator::LessThan)
        .or(just(Token::LessOrEqual).map(|_| BoundOperator::LessOrEqual))
        .or(just(Token::GreaterThan).map(|_| BoundOperator::GreaterThan))
        .or(just(Token::GreaterOrEqual).map(|_| BoundOperator::GreaterOrEqual));

    let value = expression_parser();

    operator
        .then(value)
        .map(|(operator, value)| Bound { operator, value })
}

pub fn reward_formula<'a, 'b, I, SF: Parser<'a, I, ParsedStateFormula, E<'a>> + Clone>(
    state_formula_parser: SF,
) -> impl Parser<'a, I, ParsedRewardFormula, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    let instantaneous = just(Token::Instantaneous)
        .ignore_then(just(Token::Equal))
        .ignore_then(expression_parser())
        .map(|k| RewardFormula::Instantaneous { k });

    let cumulative = just(Token::Cumulative)
        .ignore_then(just(Token::LessOrEqual))
        .ignore_then(expression_parser())
        .map(|k| RewardFormula::Cumulative { k });

    let finally = just(Token::Finally)
        .ignore_then(state_formula_parser)
        .map(|states| RewardFormula::Finally { states });

    let long_run_average = just(Token::LRA).map(|_| RewardFormula::LongRunAverage);

    instantaneous
        .or(cumulative)
        .or(finally)
        .or(long_run_average)
}
