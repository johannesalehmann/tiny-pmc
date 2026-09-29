use serde::Deserialize;
use std::collections::HashMap;
use std::io::Read;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct IndexFile {
    pub format_version: u32,
    pub format_revision: u32,
    pub model_data: Option<ModelData>,
    pub file_data: Option<FileData>,
    pub transition_system: TransitionSystem,
    /// Annotation kind (`rewards`, `aps`, ...) -> identifier -> annotation
    pub annotations: Option<HashMap<String, HashMap<String, Annotation>>>,
    pub valuations: Option<Valuations>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ModelData {
    pub name: Option<String>,
    pub version: Option<String>,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub comment: Option<String>,
    pub doi: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FileData {
    pub tool: Option<String>,
    pub tool_version: Option<String>,
    pub creation_date: Option<u64>,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Time {
    Discrete,
    Stochastic,
    UrgentStochastic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservationsApplyTo {
    States,
    Branches,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TransitionSystem {
    pub time: Time,
    #[serde(rename = "#players")]
    pub num_players: u32,
    #[serde(rename = "#states")]
    pub num_states: u64,
    #[serde(rename = "#initial-states")]
    pub num_initial_states: u64,
    #[serde(rename = "#choices")]
    pub num_choices: u64,
    #[serde(rename = "#choice-actions")]
    pub num_choice_actions: u32,
    #[serde(rename = "#branches")]
    pub num_branches: u64,
    #[serde(rename = "#branch-actions")]
    pub num_branch_actions: u32,
    #[serde(rename = "#observations")]
    pub num_observations: u64,
    pub observations_apply_to: Option<ObservationsApplyTo>,
    pub branch_probability_type: Option<Type>,
    pub exit_rate_type: Option<Type>,
    pub observation_probability_type: Option<Type>,
    pub player_names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TypeKind {
    Bool,
    Int,
    Uint,
    IntInterval,
    UintInterval,
    Double,
    DoubleInterval,
    Rational,
    RationalInterval,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Type {
    #[serde(rename = "type")]
    pub kind: TypeKind,
    /// Size in bits; `None` means the default size of `kind`
    pub size: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntityKind {
    States,
    Choices,
    Branches,
    Observations,
    Players,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Annotation {
    pub alias: Option<String>,
    pub description: Option<String>,
    pub applies_to: Vec<EntityKind>,
    #[serde(rename = "type")]
    pub value_type: Type,
    pub lower: Option<i64>,
    pub upper: Option<i64>,
    #[serde(rename = "#strings")]
    pub num_strings: Option<u32>,
    pub probability_type: Option<Type>,
    #[serde(rename = "#probabilities")]
    pub num_probabilities: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Valuations {
    pub states: Option<ValuationDescription>,
    pub choices: Option<ValuationDescription>,
    pub branches: Option<ValuationDescription>,
    pub observations: Option<ValuationDescription>,
    pub players: Option<ValuationDescription>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ValuationDescription {
    pub unique: bool,
    #[serde(rename = "#strings")]
    pub num_strings: Option<u32>,
    pub classes: Vec<ValuationClass>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ValuationClass {
    pub variables: Vec<ValuationEntry>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ValuationEntry {
    Padding {
        padding: u64,
    },
    Variable {
        name: String,
        #[serde(rename = "is-optional")]
        is_optional: Option<bool>,
        #[serde(rename = "type")]
        value_type: Type,
        lower: Option<i64>,
        upper: Option<i64>,
        offset: Option<i64>,
    },
}

pub fn parse_index<R: Read>(reader: R) -> Result<IndexFile, serde_json::Error> {
    serde_json::from_reader(reader)
}

// TODO: Provide function to verify that the index file adheres to the rules from the spec

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_input::get_contents;

    // TODO: More comprehensive tests

    fn index_of(path: &str) -> IndexFile {
        let mut archive = get_contents(path).unwrap();
        let entry = archive
            .entries()
            .unwrap()
            .map(Result::unwrap)
            .find(|e| e.path().unwrap().to_str() == Some("index.json"))
            .unwrap();
        parse_index(entry).unwrap()
    }

    #[test]
    fn spec_examples() {
        for path in [
            "examples/from_umb_spec/mdp-uncompressed.umb",
            "examples/from_umb_spec/mdp-gzip.umb",
            "examples/from_umb_spec/mdp-xz.umb",
        ] {
            let index = index_of(path);
            assert_eq!(index.transition_system.time, Time::Discrete);
            assert_eq!(index.transition_system.num_players, 1);
            assert!(index.transition_system.branch_probability_type.is_some());
        }
    }

    #[test]
    fn valuation_entries() {
        let json = r##"{"format-version":0,"format-revision":0,
            "transition-system":{"time":"discrete","#players":0,"#states":1,"#initial-states":1,
              "#choices":1,"#choice-actions":0,"#branches":1,"#branch-actions":0,"#observations":0},
            "valuations":{"states":{"unique":true,"classes":[{"variables":[
              {"name":"x","type":{"type":"uint","size":3},"offset":-1},{"padding":5}]}]}}}"##;
        let index = parse_index(json.as_bytes()).unwrap();
        let class = &index.valuations.unwrap().states.unwrap().classes[0];
        assert!(matches!(
            class.variables[0],
            ValuationEntry::Variable {
                offset: Some(-1),
                ..
            }
        ));
        assert_eq!(class.variables[1], ValuationEntry::Padding { padding: 5 });
    }
}
