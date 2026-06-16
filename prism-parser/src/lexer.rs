use crate::ParserError;
use chumsky::prelude::*;
use prism_model::{FullSpan, Span};
use std::fmt::{Display, Formatter};

/// The span used by the parser to link PRISM components to the corresponding source code
pub type ParserSpan = FullSpan;
pub type Spanned<T> = (T, ParserSpan);

/// A token of a PRISM file
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token {
    // ********
    // Keywords
    // ********
    /// The keyword `dtmc`
    Dtmc,
    /// The keyword `ctmc`
    Ctmc,
    /// The keyword `mdp`
    Mdp,
    /// The keyword `pta`
    Pta,
    /// The keyword `pomdp`
    Pomdp,
    /// The keyword `popta`
    Popta,

    /// The keyword `module`
    Module,
    /// The keyword `endmodule`
    EndModule,
    /// The keyword `const`
    Const,
    /// The keyword `global`
    Global,
    /// The keyword `label`
    Label,
    /// The keyword `formula`
    Formula,
    /// The keyword `init`
    Init,
    /// The keyword `endinit`
    EndInit,
    /// The keyword `rewards`
    Rewards,
    /// The keyword `endrewards`
    EndRewards,

    /// The type keyword `int`
    Int,
    /// The type keyword `double`
    Double,
    /// The type keyword `bool`
    Bool,

    /// The probability operator `P`
    P,
    /// The probability operator `Pmax` (also accepts `PMax`)
    PMax,
    /// The probability operator `Pmin` (also accepts `PMin`)
    PMin,

    /// The reward operator `R`
    R,
    /// The reward operator `Rmax` (also accepts `RMax`)
    RMax,
    /// The reward operator `Rmin` (also accepts `RMin`)
    RMin,

    /// The function `max` (also accepts `Max`)
    Max,
    /// The function `min` (also accepts `Min`)
    Min,

    /// The time operator `T`
    T,
    /// The time operator `Tmax` (also accepts `TMax`)
    TMax,
    /// The time operator `Tmin` (also accepts `TMin`)
    TMin,

    /// The long-run average operator `LRA`
    LRA,
    /// The long-run average operator `LRAmax` (also accepts `LRAMax`)
    LRAMax,
    /// The long-run average operator `LRAmin` (also accepts `LRAMin`)
    LRAMin,

    /// The instantaneous reward bound `I`
    Instantaneous,
    /// The cumulative reward bound `C`
    Cumulative,

    /// The path formula operator `F` (finally)
    Finally,
    /// The path formula operator `G` (generally)
    Generally,
    /// The path formula operator `U` (until)
    Until,

    /// An identifier with the given text
    Identifier(String),

    // ****************
    // Syntax elements:
    // ****************
    /// A left square bracket: `[`
    LeftSqBracket,
    /// A right square bracket: `]`
    RightSqBracket,
    /// A left parenthesis: `(`
    LeftBracket,
    /// A right parenthesis: `)`
    RightBracket,
    /// A left curly bracket: `{`
    LeftCurlyBracket,
    /// A right curly bracket: `}`
    RightCurlyBracket,
    /// An arrow: `->`
    Arrow,
    /// A primed assignment: `' =` (apostrophe, optional whitespace, equals sign)
    AssignedTo,
    /// A colon: `:`
    Colon,
    /// A double dot: `..`
    DotDot,
    /// A semicolon: `;`
    Semicolon,
    /// A double quote: `"`
    Quote,
    /// A comma: `,`
    Comma,

    // ************
    // Expressions:
    // ************
    /// An integer.
    ///
    /// This uses a string that is converted into a number later
    Integer(String),
    /// A floating-point number.
    ///
    /// This uses a string that is converted into a number later.
    ///
    /// The string may use scientific notation (I'm not sure whether PRISM supports this).
    Float(String),

    /// The boolean literal `true`
    True,
    /// The boolean literal `false`
    False,

    /// A minus sign: `-`
    Minus,
    /// A multiplication sign: `*`
    Multiply,
    /// A division sign: `/`
    Divide,
    /// A modulo sign: `%`
    Modulo,
    /// A plus sign: `+`
    Plus,
    /// A less-than sign: `<`
    LessThan,
    /// A less-than-or-equal sign: `<=`
    LessOrEqual,
    /// A greater-than sign: `>`
    GreaterThan,
    /// A greater-than-or-equal sign: `>=`
    GreaterOrEqual,
    /// An equals sign: `=`
    Equal,
    /// A not-equal sign: `!=`
    NotEqual,
    /// A negation sign: `!`
    Negation,
    /// A logical and sign: `&`
    And,
    /// A logical or sign: `|`
    Or,
    /// An if-and-only-if sign: `<=>`
    IfAndOnlyIf,
    /// An implies sign: `=>`
    Implies,
    /// A question mark: `?`
    Questionmark,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Dtmc => write!(f, "dtmc"),
            Token::Ctmc => write!(f, "ctmc"),
            Token::Mdp => write!(f, "mdp"),
            Token::Pta => write!(f, "pta"),
            Token::Pomdp => write!(f, "pomdp"),
            Token::Popta => write!(f, "popta"),
            Token::Module => write!(f, "module"),
            Token::EndModule => write!(f, "endmodule"),
            Token::Const => write!(f, "const"),
            Token::Global => write!(f, "global"),
            Token::Label => write!(f, "label"),
            Token::Formula => write!(f, "formula"),
            Token::Init => write!(f, "init"),
            Token::EndInit => write!(f, "endinit"),
            Token::Rewards => write!(f, "rewards"),
            Token::EndRewards => write!(f, "endrewards"),
            Token::Int => write!(f, "int"),
            Token::Double => write!(f, "double"),
            Token::Bool => write!(f, "bool"),
            Token::P => write!(f, "P"),
            Token::PMax => write!(f, "Pmax"),
            Token::PMin => write!(f, "Pmin"),
            Token::R => write!(f, "R"),
            Token::RMax => write!(f, "Rmax"),
            Token::RMin => write!(f, "Rmin"),
            Token::Max => write!(f, "max"),
            Token::Min => write!(f, "min"),
            Token::T => write!(f, "T"),
            Token::TMax => write!(f, "Tmax"),
            Token::TMin => write!(f, "Tmin"),
            Token::LRA => write!(f, "LRA"),
            Token::LRAMax => write!(f, "LRAmax"),
            Token::LRAMin => write!(f, "LRAmin"),
            Token::Instantaneous => write!(f, "I"),
            Token::Cumulative => write!(f, "C"),
            Token::Finally => write!(f, "F"),
            Token::Generally => write!(f, "G"),
            Token::Until => write!(f, "U"),
            Token::Identifier(id) => write!(f, "{id}"),
            Token::LeftSqBracket => write!(f, "["),
            Token::RightSqBracket => write!(f, "]"),
            Token::LeftBracket => write!(f, "("),
            Token::RightBracket => write!(f, ")"),
            Token::Arrow => write!(f, "->"),
            Token::AssignedTo => write!(f, "'="),
            Token::Colon => write!(f, ":"),
            Token::DotDot => write!(f, ".."),
            Token::Semicolon => write!(f, ";"),
            Token::Quote => write!(f, "\""),
            Token::Comma => write!(f, ","),
            Token::Integer(val) => write!(f, "{val}"),
            Token::Float(val) => write!(f, "{val}"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Modulo => write!(f, "%"),
            Token::Plus => write!(f, "+"),
            Token::LessThan => write!(f, "<"),
            Token::LessOrEqual => write!(f, "<="),
            Token::GreaterThan => write!(f, ">"),
            Token::GreaterOrEqual => write!(f, ">="),
            Token::Equal => write!(f, "="),
            Token::NotEqual => write!(f, "!="),
            Token::Negation => write!(f, "!"),
            Token::And => write!(f, "&"),
            Token::Or => write!(f, "|"),
            Token::IfAndOnlyIf => write!(f, "<=>"),
            Token::Implies => write!(f, "=>"),
            Token::Questionmark => write!(f, "?"),
            Token::LeftCurlyBracket => write!(f, "{{"),
            Token::RightCurlyBracket => write!(f, "}}"),
        }
    }
}

fn lexer<'a, F: Fn(SimpleSpan) -> FullSpan + 'static>() -> impl Parser<
    'a,
    chumsky::input::MappedSpan<FullSpan, &'a str, F>,
    Vec<Spanned<Token>>,
    extra::Err<ParserError<'a, ParserSpan, char>>,
> {
    let float = text::int(10)
        .then(just('.').then(text::digits(10)))
        .then(
            just('e')
                .then(just('-').or_not())
                .then(just('0').repeated())
                .then(text::int(10))
                .or_not(),
        )
        .to_slice()
        .from_str()
        .unwrapped()
        .map(Token::Float);

    let int = text::int(10)
        .to_slice()
        .from_str()
        .unwrapped()
        .map(Token::Integer);

    let operator = just('-')
        .map(|_| Token::Minus)
        .or(just('*').map(|_| Token::Multiply))
        .or(just('/').map(|_| Token::Divide))
        .or(just('%').map(|_| Token::Modulo))
        .or(just('+').map(|_| Token::Plus))
        .or(just("=>").map(|_| Token::Implies))
        .or(just("<=>").map(|_| Token::IfAndOnlyIf))
        .or(just("<=").map(|_| Token::LessOrEqual))
        .or(just('<').map(|_| Token::LessThan))
        .or(just(">=").map(|_| Token::GreaterOrEqual))
        .or(just('>').map(|_| Token::GreaterThan))
        .or(just('=').map(|_| Token::Equal))
        .or(just("!=").map(|_| Token::NotEqual))
        .or(just('!').map(|_| Token::Negation))
        .or(just('&').map(|_| Token::And))
        .or(just('|').map(|_| Token::Or))
        .or(just('?').map(|_| Token::Questionmark))
        .or(just(',').map(|_| Token::Comma));

    let syntax_element = just('[')
        .map(|_| Token::LeftSqBracket)
        .or(just(']').map(|_| Token::RightSqBracket))
        .or(just('(').map(|_| Token::LeftBracket))
        .or(just(')').map(|_| Token::RightBracket))
        .or(just('{').map(|_| Token::LeftCurlyBracket))
        .or(just('}').map(|_| Token::RightCurlyBracket))
        .or(just("->").map(|_| Token::Arrow))
        .or(just('\'')
            .then(chumsky::text::whitespace())
            .then(just('='))
            .map(|_| Token::AssignedTo))
        .or(just(':').map(|_| Token::Colon))
        .or(just("..").map(|_| Token::DotDot))
        .or(just(';').map(|_| Token::Semicolon))
        .or(just('\"').map(|_| Token::Quote));

    let identifier = text::ident().map(|ident: &str| match ident {
        "dtmc" => Token::Dtmc,
        "ctmc" => Token::Ctmc,
        "mdp" => Token::Mdp,
        "pta" => Token::Pta,
        "pomdp" => Token::Pomdp,
        "popta" => Token::Popta,

        "module" => Token::Module,
        "endmodule" => Token::EndModule,
        "const" => Token::Const,
        "global" => Token::Global,
        "label" => Token::Label,
        "formula" => Token::Formula,
        "init" => Token::Init,
        "endinit" => Token::EndInit,
        "rewards" => Token::Rewards,
        "endrewards" => Token::EndRewards,

        "int" => Token::Int,
        "double" => Token::Double,
        "bool" => Token::Bool,

        "PMax" | "Pmax" => Token::PMax,
        "PMin" | "Pmin" => Token::PMin,
        "P" => Token::P,

        "RMax" | "Rmax" => Token::RMax,
        "RMin" | "Rmin" => Token::RMin,
        "R" => Token::R,

        "TMax" | "Tmax" => Token::TMax,
        "TMin" | "Tmin" => Token::TMin,
        "T" => Token::T,

        "LRA" => Token::LRA,
        "LRAMax" | "LRAmax" => Token::LRAMax,
        "LRAMin" | "LRAmin" => Token::LRAMin,

        "I" => Token::Instantaneous,
        "C" => Token::Cumulative,

        "Max" | "max" => Token::Max,
        "Min" | "min" => Token::Min,

        "F" => Token::Finally,
        "G" => Token::Generally,

        "true" => Token::True,
        "false" => Token::False,

        _ => Token::Identifier(ident.to_string()),
    });

    let token = float.or(int).or(syntax_element).or(operator).or(identifier);

    let comment = just("//")
        .then(any().and_is(just('\n').not()).repeated())
        .padded();

    token
        .map_with(|tok, e| (tok, e.span()))
        .padded_by(comment.repeated())
        .padded()
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}

pub fn raw_lex(
    program: &str,
) -> ParseResult<Vec<Spanned<Token>>, ParserError<'_, ParserSpan, char>> {
    lexer().parse(program.map_span(|s| FullSpan::from_range(s.into_range())))
}
