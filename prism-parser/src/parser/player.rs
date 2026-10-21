use super::{E, identifier_parser};
use crate::{ParserSpan, Token};
use chumsky::IterParser;
use chumsky::Parser;
use chumsky::input::ValueInput;
use chumsky::prelude::just;
use prism_model::Identifier;

pub fn player_parser<'a, 'b, I>() -> impl Parser<'a, I, prism_model::Player<ParserSpan>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    just(Token::Player)
        .ignore_then(identifier_parser())
        .then(
            player_element_parser()
                .separated_by(just(Token::Comma))
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(Token::EndPlayer))
        .map_with(|(name, elements), e| {
            let mut controlled_modules = Vec::new();
            let mut controlled_actions = Vec::new();
            for element in elements {
                match element {
                    PlayerElement::Module(m) => controlled_modules.push(m),
                    PlayerElement::Action(a) => controlled_actions.push(a),
                }
            }
            prism_model::Player::new_spanned(name, controlled_modules, controlled_actions, e.span())
        })
        .labelled("player")
        .as_context()
}

enum PlayerElement {
    Module(Identifier<ParserSpan>),
    Action(Identifier<ParserSpan>),
}

fn player_element_parser<'a, 'b, I>() -> impl Parser<'a, I, PlayerElement, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    identifier_parser()
        .map(PlayerElement::Module)
        .or(identifier_parser()
            .delimited_by(just(Token::LeftSqBracket), just(Token::RightSqBracket))
            .map(PlayerElement::Action))
        .labelled("player element")
        .as_context()
}
