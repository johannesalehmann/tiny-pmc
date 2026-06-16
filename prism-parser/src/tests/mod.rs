#[macro_export]
macro_rules! parse_maybe {
    ($source: expr, $parser: expr, $output: ident, $errors: ident) => {
        use crate::{ParserSpan, Token};
        use chumsky::prelude::*;
        use prism_model::{FullSpan, Span};
        let input = $source;
        let mut errors = Vec::new();
        let lexed = crate::lex(input, &mut errors)
            .unwrap_or_else(|| panic!("Failed to lex input: {:?}", errors));
        let tokens = lexed.as_slice();
        let mapper: fn(&(Token, FullSpan)) -> (&Token, &FullSpan) = |(t, s)| (t, s);
        let ($output, $errors) = $parser
            .map_with(|ast, e| (ast, e.span()))
            .parse(tokens.map(ParserSpan::from_range(input.len()..input.len()), mapper))
            .into_output_errors();
    };
}

#[macro_export]
macro_rules! parse_success {
    ($source: expr, $parser: expr) => {{
        use crate::parse_maybe;
        parse_maybe!($source, $parser, output, parse_errors);
        assert_eq!(parse_errors, Vec::new());
        assert!(output.is_some());
        output.unwrap()
    }};
}

#[macro_export]
macro_rules! parse_error {
    ($source: expr, $parser: expr, $error: expr) => {{
        use crate::parse_maybe;
        parse_maybe!($source, $parser, output, parse_errors);
        assert_eq!(output, None);
        assert_eq!(parse_errors, vec![$error]);
    }};
}

#[macro_export]
macro_rules! parse_errors {
    ($source: expr, $parser: expr) => {{
        use crate::parse_maybe;
        parse_maybe!($source, $parser, output, parse_errors);
        assert_eq!(output, None);
        assert!(!parse_errors.is_empty());
        parse_errors
    }};
}
