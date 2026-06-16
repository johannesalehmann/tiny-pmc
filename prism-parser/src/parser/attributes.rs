use crate::parser::{E, identifier_parser};
use crate::{ParserSpan, Token};
use chumsky::Parser;
use chumsky::input::ValueInput;
use chumsky::prelude::*;
use prism_model::{Attributes, Span};

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
        .or_not()
        .map(|a| a.unwrap_or_else(|| Attributes::new()))
}

pub fn attribute_parser<'a, 'b, I>() -> impl Parser<'a, I, prism_model::Attribute, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = ParserSpan>,
{
    identifier_parser()
        .then(
            just(Token::Equal)
                .ignore_then(
                    none_of(&[Token::Comma, Token::RightCurlyBracket])
                        .map_with(|t, e| (t, e.span()))
                        .repeated()
                        .collect::<Vec<(Token, _)>>()
                        .map(string_from_spanned_tokens),
                )
                .or_not(),
        )
        .map(|(key, value)| prism_model::Attribute {
            key: key.name,
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
    use crate::{ParserError, parse_error, parse_success};
    use chumsky::error::RichPattern;
    use prism_model::{Attribute, Attributes, FullSpan, Span};

    macro_rules! test_attribute {
        ($key: expr, $range: expr) => {
            (
                Attribute {
                    key: $key.to_string(),
                    value: None,
                },
                FullSpan::from_range($range),
            )
        };
        ($key: expr, $value: expr, $range: expr) => {
            (
                Attribute {
                    key: $key.to_string(),
                    value: Some($value.to_string()),
                },
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
        assert_eq!(output, test_attribute!("key", "value", 0..9));
    }

    #[test]
    fn attribute_key_value_empty() {
        let input = r#"key="#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "", 0..4));
    }

    #[test]
    fn attribute_key_value_leading_whitespace() {
        let input = r#"key=   value"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "value", 0..12));
    }

    #[test]
    fn attribute_key_value_trailing_whitespace() {
        let input = r#"key=value   "#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "value", 0..9));
    }

    #[test]
    fn attribute_key_value_integer() {
        let input = r#"key=3"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "3", 0..5));
    }

    #[test]
    fn attribute_key_value_function() {
        let input = r#"key=max(3)"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "max(3)", 0..10));
    }

    #[test]
    fn attribute_key_value_merge_with_space() {
        let input = r#"key=asdf jkl"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "asdf jkl", 0..12));
    }

    #[test]
    fn attribute_key_value_merge_without_space() {
        let input = r#"key=asdf-jkl"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "asdf-jkl", 0..12));
    }

    #[test]
    fn attribute_key_value_merge_with_double_space() {
        let input = r#"key=asdf  jkl"#;
        let output = parse_success!(input, attribute_parser());
        assert_eq!(output, test_attribute!("key", "asdf  jkl", 0..13));
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
                0..48
            )
        );
    }

    macro_rules! test_attributes {
        ([$(($key: expr, $value: expr)),*], $range: expr) => {
            (
                Attributes::with_attributes(vec![
                    $(Attribute {
                        key: $key.to_string(),
                        value: match $value {
                            Some(value) => Some(str::to_string(value)),
                            None => None
                        }
                    }),*
                ]),
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
        assert_eq!(output, test_attributes!([("key", None)], 0..5));
    }
    #[test]
    fn attributes_single_key_value() {
        let input = r#"{key=value}"#;
        let output = parse_success!(input, attributes_parser());
        assert_eq!(output, test_attributes!([("key", Some("value"))], 0..11));
    }
    #[test]
    fn attributes_single_key_value_complex() {
        let input = r#"{key=a b  c+d e f}"#;
        let output = parse_success!(input, attributes_parser());
        assert_eq!(
            output,
            test_attributes!([("key", Some("a b  c+d e f"))], 0..18)
        );
    }
    #[test]
    fn attributes_single_key_value_complex_spaces() {
        let input = r#"{  key  =    a b  c+d e f    }"#;
        let output = parse_success!(input, attributes_parser());
        assert_eq!(
            output,
            test_attributes!([("key", Some("a b  c+d e f"))], 0..30)
        );
    }
    #[test]
    fn attributes_mixed_keys_and_key_values() {
        let input = r#"{asdf, key=value, jkl}"#;
        let output = parse_success!(input, attributes_parser());
        assert_eq!(
            output,
            test_attributes!(
                [("asdf", None), ("key", Some("value")), ("jkl", None)],
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
                    ("asdf", None),
                    ("qwe", Some("3+4  +5")),
                    ("key", Some("value")),
                    ("jkl", None),
                    ("test", Some("x=y")),
                    ("e", Some("5"))
                ],
                0..62
            )
        );
    }
}
