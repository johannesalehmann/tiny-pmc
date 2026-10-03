use std::fmt::{Display, Formatter};

pub struct ModelAndPropArgs {
    pub prism_files: Vec<String>,
    pub umb_files: Vec<String>,
    pub property_sources: Vec<PropertySource>,
    pub property_names: Vec<String>,
}

pub enum PropertySource {
    File(String),
    String(String),
}

#[derive(Debug)]
pub struct UnknownExtension {
    pub file: String,
}

impl Display for UnknownExtension {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unknown extension for file `{}`. Supported endings are `.umb` (for Universal Markov Binaries), `.props` (for property files) `.prism`, `.nm`, `.pm` and `.sm` (all for PRISM models)",
            self.file
        )
    }
}

impl ModelAndPropArgs {
    // TODO: Accept other types, e.g. &[&str]
    pub fn from_cli_args(arguments: &[String]) -> Result<Self, UnknownExtension> {
        let mut prism_files = Vec::new();
        let mut umb_files = Vec::new();
        let mut property_sources = Vec::new();
        let mut property_names = Vec::new();

        for argument in arguments {
            if argument.ends_with(".umb") {
                umb_files.push(argument.clone());
            } else if argument.ends_with(".nm") // Deprecated file extension for MDPs
                || argument.ends_with(".pm") // Deprecated file extension for DTMCs
                || argument.ends_with(".sm") // Deprecated file extension for CTMCs
                || argument.ends_with(".prism")
            {
                prism_files.push(argument.clone());
            } else if argument.ends_with(".props") {
                property_sources.push(PropertySource::File(argument.clone()));
            } else {
                // Distinguish between property names (referring to some property file) and an
                // in-line property specification.
                if argument.contains('[')
                    || argument.contains(']')
                    || argument.contains(' ')
                    || argument.contains('<')
                    || argument.contains('>')
                    || argument.contains('=')
                {
                    property_sources.push(PropertySource::String(argument.clone()));
                } else if argument.contains("/") || argument.contains("\\") {
                    // Note that it is important that this check comes after the previous check.
                    // After all, properties could easily contain `/` in expressions.
                    return Err(UnknownExtension {
                        file: argument.clone(),
                    });
                } else {
                    property_names.push(argument.clone());
                }
            }
        }

        Ok(Self {
            prism_files,
            umb_files,
            property_sources,
            property_names,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ModelAndPropArgs, PropertySource, UnknownExtension};

    const VALID_PROPERTIES: &[&str] = &[
        r#"P=? [ F "goal" ]"#,
        r#"P=?[F"goal"]"#,
        r#"Pmin=? [ "safe" U "goal" ]"#,
        r#"P>=0.5 [ !"bad" U<=5 "goal" ]"#,
        r#"P=? [ F[2,5] "goal" ]"#,
        r#"R{"energy"}max=? [ C<=100 ]"#,
        r#"LRA=? [ "up" ]"#,
        r#"E [ F "goal" ]"#,
        r#""goal" | "fail""#,
        r#"x>3"#,
        r#"filter(forall, P>=1 [ F "done" ], "init")"#,
        r#""reach": Pmax=? [ F "goal" ]"#,
        // Contain substrings that look like file extensions or file paths
        r#"P=? [ F "model.prism" ]"#,
        r#"Rmax=? [ F "goal.props" ]"#,
        r#"P=? [ F "a/b.nm" ]"#,
    ];

    // Each should be classified as a property string, so the property parser can report the error.
    const INVALID_PROPERTIES: &[&str] = &[
        r#"P=? [ F "goal""#,
        r#"P=? F "goal" ]"#,
        r#"P=? [ F "goal" ]]"#,
        r#"Pmax=? [ F goal" ]"#,
        r#"P=?"#,
        r#"[ F "goal" ]"#,
        r#"Pmax=? [ "safe" U ]"#,
        r#"P=? ( F "goal" )"#,
        r#"R{"energy}max=? [ C<=100 ]"#,
        r#"P=? [ F "model.prism""#,
    ];

    const PRISM_FILES: &[&str] = &[
        "model.prism",
        "brp.pm",
        "consensus.nm",
        "embedded.sm",
        "../models/coin2.nm",
        r"C:\models\die.pm",
        "my model.prism",
        "crowds[N=5].pm",
        "model.props.prism",
    ];

    const UMB_FILES: &[&str] = &[
        "model.umb",
        "out/brp.umb",
        "zeroconf-N=1000.umb",
        "model.prism.umb",
    ];

    const PROPERTY_FILES: &[&str] = &[
        "props.props",
        "coin[K=2].props",
        "my properties.props",
        "model.prism.props",
    ];

    const PROPERTY_NAMES: &[&str] = &[
        "reach_goal",
        "safety-check",
        "Pmax",
        "goal.reach",
        "prism",
        "props",
    ];

    const UNKNOWN_EXTENSION_FILES: &[&str] = &[
        "models/model.txt",
        "../models/coin2",
        "./model.PRISM",
        r"C:\models\die.pm.bak",
        "out/model.umb.gz",
    ];

    #[derive(Debug, Default, PartialEq)]
    struct Classification<'a> {
        prism_files: Vec<&'a str>,
        umb_files: Vec<&'a str>,
        property_files: Vec<&'a str>,
        property_strings: Vec<&'a str>,
        property_names: Vec<&'a str>,
    }

    #[derive(Debug, PartialEq)]
    enum Source<'a> {
        File(&'a str),
        String(&'a str),
    }

    fn parse(arguments: &[&str]) -> ModelAndPropArgs {
        let arguments: Vec<String> = arguments.iter().map(|a| a.to_string()).collect();
        ModelAndPropArgs::from_cli_args(&arguments).unwrap()
    }

    fn parse_err(arguments: &[&str]) -> UnknownExtension {
        let arguments: Vec<String> = arguments.iter().map(|a| a.to_string()).collect();
        match ModelAndPropArgs::from_cli_args(&arguments) {
            Ok(_) => panic!("expected an error for arguments {arguments:?}"),
            Err(err) => err,
        }
    }

    fn classify(args: &ModelAndPropArgs) -> Classification<'_> {
        let mut classification = Classification {
            prism_files: args.prism_files.iter().map(String::as_str).collect(),
            umb_files: args.umb_files.iter().map(String::as_str).collect(),
            property_names: args.property_names.iter().map(String::as_str).collect(),
            ..Default::default()
        };
        for source in &args.property_sources {
            match source {
                PropertySource::File(file) => classification.property_files.push(file),
                PropertySource::String(string) => classification.property_strings.push(string),
            }
        }
        classification
    }

    fn sources(args: &ModelAndPropArgs) -> Vec<Source<'_>> {
        args.property_sources
            .iter()
            .map(|source| match source {
                PropertySource::File(file) => Source::File(file),
                PropertySource::String(string) => Source::String(string),
            })
            .collect()
    }

    fn assert_single_inputs(
        inputs: &[&'static str],
        expected: impl Fn(&'static str) -> Classification<'static>,
    ) {
        for &input in inputs {
            let args = parse(&[input]);
            assert_eq!(classify(&args), expected(input), "input: {input:?}");
        }
    }

    #[test]
    fn empty_input() {
        let args = parse(&[]);
        assert_eq!(classify(&args), Classification::default());
    }

    #[test]
    fn single_valid_property() {
        assert_single_inputs(VALID_PROPERTIES, |input| Classification {
            property_strings: vec![input],
            ..Default::default()
        });
    }

    #[test]
    fn single_invalid_property() {
        assert_single_inputs(INVALID_PROPERTIES, |input| Classification {
            property_strings: vec![input],
            ..Default::default()
        });
    }

    #[test]
    fn single_prism_file() {
        assert_single_inputs(PRISM_FILES, |input| Classification {
            prism_files: vec![input],
            ..Default::default()
        });
    }

    #[test]
    fn single_umb_file() {
        assert_single_inputs(UMB_FILES, |input| Classification {
            umb_files: vec![input],
            ..Default::default()
        });
    }

    #[test]
    fn single_property_file() {
        assert_single_inputs(PROPERTY_FILES, |input| Classification {
            property_files: vec![input],
            ..Default::default()
        });
    }

    #[test]
    fn single_property_name() {
        assert_single_inputs(PROPERTY_NAMES, |input| Classification {
            property_names: vec![input],
            ..Default::default()
        });
    }

    #[test]
    fn single_unknown_extension_file() {
        for &input in UNKNOWN_EXTENSION_FILES {
            assert_eq!(parse_err(&[input]).file, input, "input: {input:?}");
        }
    }

    #[test]
    fn unknown_extension_file_among_valid_arguments() {
        let err = parse_err(&["brp.pm", "brp.props", "models/brp.txt", "reach_goal"]);
        assert_eq!(err.file, "models/brp.txt");
    }

    #[test]
    fn prism_file_with_property_file_and_names() {
        let args = parse(&["brp.pm", "brp.props", "reach_goal", "phi"]);
        assert_eq!(
            classify(&args),
            Classification {
                prism_files: vec!["brp.pm"],
                property_files: vec!["brp.props"],
                property_names: vec!["reach_goal", "phi"],
                ..Default::default()
            }
        );
    }

    #[test]
    fn umb_file_with_inline_properties() {
        let args = parse(&[
            r#"P=? [ F "out.umb" ]"#,
            "model.umb",
            r#"Rmin=? [ F "goal" ]"#,
        ]);
        assert_eq!(
            classify(&args),
            Classification {
                umb_files: vec!["model.umb"],
                property_strings: vec![r#"P=? [ F "out.umb" ]"#, r#"Rmin=? [ F "goal" ]"#],
                ..Default::default()
            }
        );
    }

    #[test]
    fn property_sources_keep_order() {
        let args = parse(&[
            "a.props",
            r#"P=? [ F "goal" ]"#,
            "b.props",
            r#"P=? [ F "goal""#,
        ]);
        assert_eq!(
            sources(&args),
            vec![
                Source::File("a.props"),
                Source::String(r#"P=? [ F "goal" ]"#),
                Source::File("b.props"),
                Source::String(r#"P=? [ F "goal""#),
            ]
        );
    }

    #[test]
    fn property_names_keep_order() {
        let args = parse(&[
            "phi",
            "reach_goal",
            "model.prism",
            "Pmax",
            r#"P=? [ F "goal" ]"#,
            "a",
        ]);
        assert_eq!(args.property_names, vec!["phi", "reach_goal", "Pmax", "a"]);
    }

    #[test]
    fn all_kinds_interleaved() {
        let args = parse(&[
            "prop1",
            "crowds[N=5].pm",
            r#"Pmax=? [ F<=10 "done" ]"#,
            "zeroconf-N=1000.umb",
            "coin[K=2].props",
            "consensus.nm",
            "Pmax",
            r#"P=?[F"out.umb""#,
            "model.umb",
        ]);
        assert_eq!(
            classify(&args),
            Classification {
                prism_files: vec!["crowds[N=5].pm", "consensus.nm"],
                umb_files: vec!["zeroconf-N=1000.umb", "model.umb"],
                property_files: vec!["coin[K=2].props"],
                property_strings: vec![r#"Pmax=? [ F<=10 "done" ]"#, r#"P=?[F"out.umb""#],
                property_names: vec!["prop1", "Pmax"],
            }
        );
    }

    #[test]
    fn duplicate_arguments_are_kept() {
        let args = parse(&["model.prism", "model.prism", "phi", "phi"]);
        assert_eq!(
            classify(&args),
            Classification {
                prism_files: vec!["model.prism", "model.prism"],
                property_names: vec!["phi", "phi"],
                ..Default::default()
            }
        );
    }
}
