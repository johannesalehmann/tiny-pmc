use crate::parser::{identifier_parser, E};
use crate::{ElementKind, ParserSpan, Token, ValidationError};
use chumsky::input::ValueInput;
use chumsky::prelude::*;
use chumsky::Parser;
use prism_model::{AttributeValue, Attributes, Span};

pub fn attributes_parser<'a, 'b, I>() -> impl Parser<'a, I, prism_model::Attributes, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    just(Token::LeftCurlyBracket)
        .ignore_then(
            attribute_parser()
                .separated_by(just(Token::Comma))
                .collect(),
        )
        .then_ignore(just(Token::RightCurlyBracket))
        .map(|a| Attributes::with_attributes(a))
        .validate(|attributes, _, emitter| match attributes {
            Ok(attributes) => attributes,
            Err(err) => {
                emitter.emit(
                    ValidationError::DuplicateElement {
                        previous_occurrence: err.existing_span,
                        new_definition: err.new_span,
                        kind: ElementKind::Attribute,
                    }
                    .into(),
                );
                Attributes::new()
            }
        })
        .or_not()
        .map(|a| a.unwrap_or_else(|| Attributes::new()))
}

pub fn attribute_parser<'a, 'b, I>() -> impl Parser<'a, I, prism_model::Attribute, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    let value_parser = none_of(&[Token::Comma, Token::RightCurlyBracket])
        .map_with(|t, e| (t, e.span()))
        .repeated()
        .collect::<Vec<(Token, _)>>()
        .map(string_from_spanned_tokens)
        .map_with(|value, e| AttributeValue {
            value,
            span: e.span(),
        });

    identifier_parser()
        .then(just(Token::Equal).ignore_then(value_parser).or_not())
        .map_with(|(key, value), e| prism_model::Attribute {
            key,
            span: e.span(),
            value,
        })
}

fn string_from_spanned_tokens(tokens: Vec<(Token, ParserSpan)>) -> String {
    let mut res = Vec::new();

    let mut prev_end = None;
    for (token, span) in tokens {
        // Tokens should always be spanned at this stage, but if not, then as a fallback, use
        // range 0..1. This has the effect of adding a space between any tokens, which is better
        // than missing spaces.
        let range = span.range().unwrap_or(0..1);
        if let Some(prev_end) = prev_end {
            for _ in prev_end..range.start {
                res.push(" ".to_string());
            }
        }
        prev_end = Some(range.end);
        res.push(token.to_string())
    }

    res.join("")
}

#[cfg(test)]
mod tests {
    use crate::parser::attributes::{attribute_parser, attributes_parser};
    use crate::{parse_error, parse_success, ParserError};
    use chumsky::error::RichPattern;
    use prism_model::{Attribute, Attributes, FullSpan, Identifier, Span};
    use std::ops::Range;

    macro_rules! test_attribute {
        ($key: expr, $range: expr) => {
            (
                Attribute::flag_spanned(
                    Identifier::new($key.to_string()).unwrap(),
                    FullSpan::from_range($range),
                ),
                FullSpan::from_range($range),
            )
        };
        ($key: expr, $value: expr, $range: expr, $value_range: expr) => {
            (
                Attribute::key_value_spanned(
                    Identifier::new($key.to_string()).unwrap(),
                    $value.to_string(),
                    FullSpan::from_range($range),
                    FullSpan::from_range($value_range),
                ),
                FullSpan::from_range($range),
            )
        };
    }

    #[test]
    fn attribute_key() {
        let input = r#"key"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", 0..3));
    }
    #[test]
    fn attribute_key_leading_number() {
        let input = r#"3valued"#;
        parse_error!(
            input,
            attribute_parser(),
            ParserError::ExpectedFound {
                span: FullSpan::from_range(0..1),
                expected: vec![RichPattern::Label("identifier".into())],
                found: Some(Token::Integer("3".to_string()).into()),
                contexts: vec![],
                help: None,
            }
        );
    }

    #[test]
    fn attribute_key_value() {
        let input = r#"key=value"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "value", 0..9, 4..9));
    }

    #[test]
    fn attribute_key_value_empty() {
        let input = r#"key="#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "", 0..4, 4..4));
    }

    #[test]
    fn attribute_key_value_leading_whitespace() {
        let input = r#"key=   value"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "value", 0..12, 7..12));
    }

    #[test]
    fn attribute_key_value_trailing_whitespace() {
        let input = r#"key=value   "#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "value", 0..9, 4..9));
    }

    #[test]
    fn attribute_key_value_integer() {
        let input = r#"key=3"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "3", 0..5, 4..5));
    }

    #[test]
    fn attribute_key_value_function() {
        let input = r#"key=max(3)"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "max(3)", 0..10, 4..10));
    }

    #[test]
    fn attribute_key_value_merge_with_space() {
        let input = r#"key=asdf jkl"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "asdf jkl", 0..12, 4..12));
    }

    #[test]
    fn attribute_key_value_merge_without_space() {
        let input = r#"key=asdf-jkl"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "asdf-jkl", 0..12, 4..12));
    }

    #[test]
    fn attribute_key_value_merge_with_double_space() {
        let input = r#"key=asdf  jkl"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "asdf  jkl", 0..13, 4..13));
    }

    #[test]
    fn attribute_key_value_merging_complex() {
        let input = r#"key=a b+c d e "asdf" (wer e = ==    ; init mdp ?"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(
            output,
            test_attribute!(
                "key",
                r#"a b+c d e "asdf" (wer e = ==    ; init mdp ?"#,
                0..48,
                4..48
            )
        );
    }

    macro_rules! attr {
        ($key: expr, $span: expr) => {{
            let span = FullSpan::from_range($span);
            Attribute::flag_spanned(Identifier::new_spanned($key, span.clone()).unwrap(), span)
        }};
        ($key: expr, $value: expr, $key_span: expr, $value_span: expr) => {{
            let key_span: Range<usize> = $key_span;
            let value_span: Range<usize> = $value_span;
            let span = key_span.start..value_span.end;
            Attribute::key_value_spanned(
                Identifier::new_spanned($key, FullSpan::from_range(key_span)).unwrap(),
                $value,
                FullSpan::from_range(span),
                FullSpan::from_range(value_span),
            )
        }};
    }

    macro_rules! test_attributes {
        ([$($attr: expr),*], $range: expr) => {
            (
                Attributes::with_attributes(vec![
                    $($attr),*
                ]).unwrap(),
                FullSpan::from_range($range),
            )
        };
    }

    #[test]
    fn attributes_none() {
        let input = r#""#;
        let output = parse_success!(input, attributes_parser());
        assert_eq!(output, test_attributes!([], 0..0));
    }
    #[test]
    fn attributes_empty() {
        let input = r#"{}"#;
        let output = parse_success!(input, attributes_parser());
        assert_eq!(output, test_attributes!([], 0..2));
    }
    #[test]
    fn attributes_empty_space() {
        let input = r#"{   }"#;
        let output = parse_success!(input, attributes_parser());
        assert_eq!(output, test_attributes!([], 0..5));
    }
    #[test]
    fn attributes_single_key() {
        let input = r#"{key}"#;
        let output = parse_success!(input, attributes_parser());
        assert_eq!(output, test_attributes!([attr!("key", 1..4)], 0..5));
    }
    #[test]
    fn attributes_single_key_value() {
        let input = r#"{key=value}"#;
        let output = parse_success!(input, attributes_parser());
        assert_eq!(
            output,
            test_attributes!([attr!("key", "value", 1..4, 5..10)], 0..11)
        );
    }
    #[test]
    fn attributes_single_key_value_complex() {
        let input = r#"{key=a b  c+d e f}"#;
        let output = parse_success!(input, attributes_parser());
        assert_eq!(
            output,
            test_attributes!([attr!("key", "a b  c+d e f", 1..4, 5..17)], 0..18)
        );
    }
    #[test]
    fn attributes_single_key_value_complex_spaces() {
        let input = r#"{  key  =    a b  c+d e f    }"#;
        let output = parse_success!(input, attributes_parser());
        assert_eq!(
            output,
            test_attributes!([attr!("key", "a b  c+d e f", 3..6, 13..25)], 0..30)
        );
    }
    #[test]
    fn attributes_mixed_keys_and_key_values() {
        let input = r#"{asdf, key=value, jkl}"#;
        let output = parse_success!(input, attributes_parser());
        assert_eq!(
            output,
            test_attributes!(
                [
                    attr!("asdf", 1..5),
                    attr!("key", "value", 7..10, 11..16),
                    attr!("jkl", 18..21)
                ],
                0..22
            )
        );
    }
    #[test]
    fn attributes_mixed_keys_and_key_values_complex() {
        let input = r#"{ asdf    , qwe=3+4  +5, key=value, jkl, test=x=y, e=  5     }"#;
        let output = parse_success!(input, attributes_parser());
        assert_eq!(
            output,
            test_attributes!(
                [
                    attr!("asdf", 2..6),
                    attr!("qwe", "3+4  +5", 12..15, 16..23),
                    attr!("key", "value", 25..28, 29..34),
                    attr!("jkl", 36..39),
                    attr!("test", "x=y", 41..45, 46..49),
                    attr!("e", "5", 51..52, 55..56)
                ],
                0..62
            )
        );
    }
}
