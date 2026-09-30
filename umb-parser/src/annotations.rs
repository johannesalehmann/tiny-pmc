use crate::csr::parse_csr;
use crate::index::TransitionSystem;
use crate::to1::{BoolTo1Error, FromLeBytes, To1Error, parse_to1, parse_to1_bool};
use crate::{AEI, API, BI, CI, OI, PI, RI, SI, SequenceIndex, StringIndex};
use probabilistic_models::annotations::{
    AtomicPropositions, EntityRewards, RewardAnnotations, StateChoiceRewards, TypedAnnotation,
};
use probabilistic_models::typed_index_collections::{Csr, To1};
use probabilistic_models::{AnnotationEntryIndex, Index};
use std::collections::HashMap;
use std::io::Read;

#[derive(Debug)]
pub enum AnnotationError<E> {
    InvalidPath {
        path: String,
    },
    InvalidTarget {
        target: String,
        path: String,
    },
    EntryError {
        path: String,
        error: AnnotationEntryError<E>,
    },
}

#[derive(Debug)]
pub enum AnnotationEntryError<E> {
    InvalidValueFile(To1Error<E>),
    InvalidStringMappingFile, // TODO: Add underlying error for the `Invalid...File` variants
    InvalidStringsFile,
    InvalidDistributionMappingFile,
    InvalidProbabilitiesFile,
    InvalidFile { file: String },
}

#[derive(Default)]
pub struct AnnotationGroupInProgress<E> {
    entries: HashMap<String, AnnotationEntryInProgress<E>>,
}

impl<E: ParseTo1> AnnotationGroupInProgress<E> {
    fn get_mut_or_add(&mut self, id: &str) -> &mut AnnotationEntryInProgress<E> {
        if self.entries.get_mut(id).is_none() {
            self.entries
                .insert(id.to_string(), AnnotationEntryInProgress::default());
        }
        self.entries.get_mut(id).unwrap()
    }

    pub fn process_file<R: Read>(
        &mut self,
        path: &str,
        contents: R,
        sizes: &TransitionSystem,
    ) -> Result<(), AnnotationError<E::Error>> {
        let segments = path.split("/").collect::<Vec<_>>();
        if segments.len() != 5 {
            return Err(AnnotationError::InvalidPath { path: path.into() });
        }
        if segments[0] != "annotations" {
            panic!("Can only call `process_file` on files with path prefix `annotations`");
        }
        let id = segments[2];
        let target = segments[3];
        let file = segments[4];
        match target {
            "states" => self
                .get_mut_or_add(id)
                .get_mut_state()
                .process_file(file, contents, sizes.num_states)
                .map_err(|error| AnnotationError::EntryError {
                    error,
                    path: path.to_string(),
                }),
            "choices" => self
                .get_mut_or_add(id)
                .get_mut_choice()
                .process_file(file, contents, sizes.num_choices)
                .map_err(|error| AnnotationError::EntryError {
                    error,
                    path: path.to_string(),
                }),
            "branches" => self
                .get_mut_or_add(id)
                .get_mut_branch()
                .process_file(file, contents, sizes.num_branches)
                .map_err(|error| AnnotationError::EntryError {
                    error,
                    path: path.to_string(),
                }),
            "observations" => self
                .get_mut_or_add(id)
                .get_mut_observation()
                .process_file(file, contents, sizes.num_observations)
                .map_err(|error| AnnotationError::EntryError {
                    error,
                    path: path.to_string(),
                }),
            "players" => self
                .get_mut_or_add(id)
                .get_mut_player()
                .process_file(file, contents, sizes.num_players as u64)
                .map_err(|error| AnnotationError::EntryError {
                    error,
                    path: path.to_string(),
                }),
            _ => Err(AnnotationError::InvalidTarget {
                target: target.into(),
                path: path.into(),
            }),
        }
    }
}

impl AnnotationGroupInProgress<bool> {
    pub fn finish_as_aps(self) -> AtomicPropositions<API, SI, AEI> {
        let mut aps: AtomicPropositions<API, SI, AEI> = Default::default();
        for (name, entry) in self.entries {
            // TODO: Return an error here instead of panicing
            if entry.state_annotations.is_none() {
                panic!("Atomic proposition `{name}` does not have an entry for states");
            }
            if entry.choice_annotations.is_some() {
                panic!("Atomic proposition `{name}` has an entry for choices");
            }
            if entry.branch_annotations.is_some() {
                panic!("Atomic proposition `{name}` has an entry for branches");
            }
            if entry.observation_annotations.is_some() {
                panic!("Atomic proposition `{name}` has an entry for observations");
            }
            if entry.player_annotations.is_some() {
                panic!("Atomic proposition `{name}` has an entry for players");
            }
            let states = entry.state_annotations.unwrap();
            if states.strings.is_some() || states.string_mapping.is_some() {
                panic!("Atomic proposition `{name}` contains strings");
            }
            if states.probabilities.is_some() || states.distribution.is_some() {
                panic!("Atomic proposition `{name}` is probabilistic")
            }
            let Some(values) = states.values else {
                panic!("Atomic proposition `{name}` does not contain state values");
            };

            aps.add_entry(
                name,
                TypedAnnotation::with_identity_distribution_and_entries(values),
            );
        }
        aps
    }
}
impl AnnotationGroupInProgress<f64> {
    pub fn finish_as_rewards(self) -> RewardAnnotations<RI, SI, CI, AEI> {
        // TODO: This function barely does any verification on its inputs, e.g. ignoring any other
        //  files present (such as distributions, strings), annotations besides states and choices.
        let mut rewards: RewardAnnotations<RI, SI, CI, AEI> = RewardAnnotations::new();
        for (name, entry) in self.entries {
            let mut rewards_entry: StateChoiceRewards<SI, CI, AEI> = StateChoiceRewards::new();
            if let Some(states) = entry.state_annotations {
                let values = states
                    .values
                    .unwrap_or_else(|| panic!("No values provided for state reward `{name}`."));
                rewards_entry.add_state_rewards(
                    EntityRewards::with_identity_distribution_and_entries(values),
                );
            }
            if let Some(choice) = entry.choice_annotations {
                let values = choice
                    .values
                    .unwrap_or_else(|| panic!("No values provided for state reward `{name}`."));
                rewards_entry.add_choice_rewards(
                    EntityRewards::with_identity_distribution_and_entries(values),
                );
            }
            rewards.add_entry(name, rewards_entry);
        }
        rewards
    }
}

struct AnnotationEntryInProgress<E> {
    state_annotations: Option<AnnotationInProgress<SI, E>>,
    choice_annotations: Option<AnnotationInProgress<CI, E>>,
    branch_annotations: Option<AnnotationInProgress<BI, E>>,
    observation_annotations: Option<AnnotationInProgress<OI, E>>,
    player_annotations: Option<AnnotationInProgress<PI, E>>,
}

impl<E> Default for AnnotationEntryInProgress<E> {
    fn default() -> Self {
        Self {
            state_annotations: None,
            choice_annotations: None,
            branch_annotations: None,
            observation_annotations: None,
            player_annotations: None,
        }
    }
}

impl<E> AnnotationEntryInProgress<E> {
    fn get_mut_state(&mut self) -> &mut AnnotationInProgress<SI, E> {
        if self.state_annotations.is_none() {
            self.state_annotations = Some(Default::default());
        }
        self.state_annotations.as_mut().unwrap()
    }
    fn get_mut_choice(&mut self) -> &mut AnnotationInProgress<CI, E> {
        if self.choice_annotations.is_none() {
            self.choice_annotations = Some(Default::default());
        }
        self.choice_annotations.as_mut().unwrap()
    }
    fn get_mut_branch(&mut self) -> &mut AnnotationInProgress<BI, E> {
        if self.branch_annotations.is_none() {
            self.branch_annotations = Some(Default::default());
        }
        self.branch_annotations.as_mut().unwrap()
    }
    fn get_mut_observation(&mut self) -> &mut AnnotationInProgress<OI, E> {
        if self.observation_annotations.is_none() {
            self.observation_annotations = Some(Default::default());
        }
        self.observation_annotations.as_mut().unwrap()
    }
    fn get_mut_player(&mut self) -> &mut AnnotationInProgress<PI, E> {
        if self.player_annotations.is_none() {
            self.player_annotations = Some(Default::default());
        }
        self.player_annotations.as_mut().unwrap()
    }
}

struct AnnotationInProgress<I: Index, E> {
    values: Option<To1<I, E>>,
    distribution: Option<Csr<I, AnnotationEntryIndex<u64>>>,
    probabilities: Option<To1<AnnotationEntryIndex<u64>, f64>>,
    string_mapping: Option<Csr<StringIndex<u64>, SequenceIndex<u64>>>,
    strings: Option<String>,
}

impl<I: Index, E> Default for AnnotationInProgress<I, E> {
    fn default() -> Self {
        Self {
            values: None,
            distribution: None,
            probabilities: None,
            string_mapping: None,
            strings: None,
        }
    }
}

impl<I: Index, E: ParseTo1> AnnotationInProgress<I, E> {
    pub fn process_file<R: Read>(
        &mut self,
        file: &str,
        mut contents: R,
        entity_count: u64,
    ) -> Result<(), AnnotationEntryError<E::Error>> {
        match file {
            "values.bin" => {
                let values = E::parse_to1(contents, entity_count)
                    .map_err(|e| AnnotationEntryError::InvalidValueFile(e))?;
                self.values = Some(values);
                Ok(())
            }
            "string-mapping.bin" => Ok(()),
            "strings.bin" => {
                let mut strings = "".to_string();
                contents
                    .read_to_string(&mut strings)
                    .map_err(|_| AnnotationEntryError::InvalidStringsFile)?;
                self.strings = Some(strings);
                Ok(())
            }
            "distribution-mapping.bin" => {
                let distribution_mapping = parse_csr(contents)
                    .map_err(|_| AnnotationEntryError::InvalidDistributionMappingFile)?;
                self.distribution = Some(distribution_mapping);
                Ok(())
            }
            "probabilities.bin" => {
                let probabilities = parse_to1(contents)
                    .map_err(|_| AnnotationEntryError::InvalidProbabilitiesFile)?;
                self.probabilities = Some(probabilities);
                Ok(())
            }
            _ => Err(AnnotationEntryError::InvalidFile { file: file.into() }),
        }
    }
}

// TODO: Could this be used elsewhere in the crate?
pub trait ParseTo1 {
    type Error;
    fn parse_to1<R: Read, From: Index>(
        file: R,
        len: u64,
    ) -> Result<To1<From, Self>, To1Error<Self::Error>>
    where
        Self: Sized;
}

impl<LE: FromLeBytes> ParseTo1 for LE {
    type Error = LE::Error;

    fn parse_to1<R: Read, From: Index>(
        file: R,
        _len: u64,
    ) -> Result<To1<From, Self>, To1Error<LE::Error>> {
        Ok(parse_to1(file)?)
    }
}
impl ParseTo1 for bool {
    type Error = BoolTo1Error;

    fn parse_to1<R: Read, From: Index>(
        file: R,
        len: u64,
    ) -> Result<To1<From, Self>, To1Error<BoolTo1Error>> {
        // TODO: Figure out whether `len` should be usize or u64 everywhere
        Ok(parse_to1_bool(file, len as usize)?)
    }
}
