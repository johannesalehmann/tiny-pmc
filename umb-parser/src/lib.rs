mod annotations;
mod csr;
mod file_input;
mod index;
#[cfg(test)]
mod tests;
mod to1;

use crate::annotations::AnnotationGroupInProgress;
use crate::index::{Time, Type, TypeKind};
use crate::to1::BoolTo1Error;
use probabilistic_models::annotations::{AtomicPropositions, RewardAnnotations};
use probabilistic_models::base_model::Mdp;
use probabilistic_models::typed_index_collections::{Csr, To1, index};
use probabilistic_models::{
    AnnotationEntryIndex, AnnotationIndex, AtomicPropositionIndex, BranchIndex, ChoiceIndex, Index,
    InitialStates, Model, PlayerIndex, StateIndex,
};
use std::convert::Infallible;
use std::path::Path;

type SI = StateIndex<usize>;
type CI = ChoiceIndex<usize>;
type BI = BranchIndex<usize>;
type PI = PlayerIndex<usize>;
type OI = AnnotationIndex<usize>; // TODO: Figure out which index this should be once observations are fully supported
type AEI = AnnotationEntryIndex<usize>;
type API = AtomicPropositionIndex<usize>;
type RI = AnnotationIndex<usize>;
pub fn parse_umb<P: AsRef<Path>>(
    path: P,
) -> Result<
    Model<
        Mdp<SI, CI, BI>,
        Option<InitialStates<SI>>,
        (),
        (),
        (),
        Option<AtomicPropositions<API, SI, AEI>>,
        Option<RewardAnnotations<RI, SI, CI, AEI>>,
        (),
        (),
        (),
    >,
    UmbError,
> {
    let mut file = file_input::get_contents(path).map_err(UmbError::FileError)?;
    let mut entries = file.entries()?;
    let Some(index_file) = entries.next() else {
        return Err(UmbError::NoIndex);
    };
    let index_file = index_file?;
    if index_file.path()?.to_str() != Some("index.json") {
        return Err(UmbError::NoIndex);
    }
    let index = index::parse_index(index_file)?;

    // TODO: Support non-f64 values for exit rates and probabilities (currently panics)
    let mut state_to_choice: Option<Csr<SI, CI>> = None;
    #[allow(unused)]
    let mut state_to_player: Option<To1<SI, PI>> = None;
    let mut state_is_initial: Option<To1<SI, bool>> = None;
    #[allow(unused)]
    let mut state_is_markovian: Option<To1<SI, bool>> = None;
    #[allow(unused)]
    let mut state_to_exit_rate: Option<To1<SI, f64>> = None;
    let mut choice_to_branch: Option<Csr<CI, BI>> = None;
    let mut branch_to_target: Option<To1<BI, SI>> = None;
    let mut branch_to_probability: Option<To1<BI, f64>> = None;
    let mut atomic_propositions: Option<AnnotationGroupInProgress<bool>> = None;
    let mut rewards: Option<AnnotationGroupInProgress<f64>> = None;

    while let Some(entry) = entries.next() {
        let entry = entry?;
        match entry.path()?.to_string_lossy().into_owned().as_str() {
            "state-to-choices.bin" => state_to_choice = Some(csr::parse_csr::<SI, CI, _>(entry)?),
            "state-to-player.bin" => {
                state_to_player =
                    Some(to1::parse_to1::<SI, u32, _>(entry)?.map(|i| PI::from_raw(*i as usize)))
            }
            "state-is-initial.bin" => {
                state_is_initial = Some(to1::parse_to1_bool::<SI, _>(
                    entry,
                    index.transition_system.num_states as usize,
                )?)
            }
            "state-is-markovian.bin" => {
                if index.transition_system.time != Time::UrgentStochastic {
                    return Err(UmbError::StateIsMarkovianForInvalidModelType);
                }
                state_is_markovian = Some(to1::parse_to1_bool::<SI, _>(
                    entry,
                    index.transition_system.num_states as usize,
                )?)
            }
            "state-to-exit-rate.bin" => {
                if !matches!(
                    index.transition_system.exit_rate_type,
                    Some(Type {
                        kind: TypeKind::Double,
                        ..
                    })
                ) {
                    // TODO: Distinguish between unsupported and invalid types
                    // TODO: For floats, check that the specified size is correct (or do so above)
                    panic!("Missing, invalid or unsupported exit rate type")
                }
                state_to_exit_rate = Some(to1::parse_to1::<SI, f64, _>(entry)?)
            }
            "choice-to-branches.bin" => {
                choice_to_branch = Some(csr::parse_csr::<CI, BI, _>(entry)?)
            }
            "branch-to-target.bin" => {
                branch_to_target =
                    Some(to1::parse_to1::<BI, u64, _>(entry)?.map(|i| SI::from_raw(*i as usize)))
            }
            "branch-to-probability.bin" => {
                if !matches!(
                    index.transition_system.branch_probability_type,
                    Some(Type {
                        kind: TypeKind::Double,
                        ..
                    })
                ) {
                    // TODO: See exit rate above
                    panic!("Missing, invalid or unsupported branch probability type")
                }
                branch_to_probability = Some(to1::parse_to1::<BI, f64, _>(entry)?)
            }
            name if name.starts_with("actions/") => {
                println!(
                    "Warning: The UMB parser does not yet support action labels (ignoring {name})"
                )
            }
            name if name.starts_with("observations/") => {
                println!(
                    "Warning: The UMB parser does not yet support action labels (ignoring {name})"
                )
            }
            name if name.starts_with("annotations/rewards/") => {
                if rewards.is_none() {
                    rewards = Some(Default::default());
                }
                rewards
                    .as_mut()
                    .unwrap()
                    .process_file(name, entry, &index.transition_system)
                    .map_err(UmbError::RewardsError)?
            }
            name if name.starts_with("annotations/aps/") => {
                if atomic_propositions.is_none() {
                    atomic_propositions = Some(Default::default());
                }
                atomic_propositions
                    .as_mut()
                    .unwrap()
                    .process_file(name, entry, &index.transition_system)
                    .map_err(UmbError::AtomicPropositionsError)?
            }
            name if name.starts_with("annotations/") => {
                println!(
                    "Warning: The UMB parser does not yet annotations, apart from rewards and atomic propositions (ignoring {name})"
                )
            }
            name => println!("File {name} currently unsupported and will be ignored"),
        }
    }

    assert_eq!(
        index.transition_system.num_players, 1,
        "The UMB parser currently only supports 1-player games"
    );
    assert_eq!(
        index.transition_system.time,
        Time::Discrete,
        "The UMB parser currently only supports discrete-time models"
    );

    let state_to_choice = unwrap_csr(state_to_choice, index.transition_system.num_states);
    let choice_to_branch = unwrap_csr(choice_to_branch, index.transition_system.num_choices);
    let branch_probabilities =
        unwrap_to1(branch_to_probability, index.transition_system.num_branches);
    let branch_destinations = unwrap_to1(branch_to_target, index.transition_system.num_branches);
    let atomic_propositions = atomic_propositions.map(|ap| ap.finish_as_aps());
    let rewards = rewards.map(|rew| rew.finish_as_rewards());

    let mdp = Mdp {
        state_to_choice,
        choice_to_branch,
        branch_probabilities,
        branch_destinations,
    };

    Ok(Model {
        base: mdp,
        initial: state_is_initial,
        choice_labels: (),
        branch_labels: (),
        observations: (),
        atomic_propositions,
        rewards,
        annotations: (),
        state_valuations: (),
        predecessors: (),
    })
}

fn unwrap_csr<From: Index, To: Index>(csr: Option<Csr<From, To>>, len: u64) -> Csr<From, To> {
    match csr {
        None => Csr::identity(len as usize),
        Some(csr) => csr,
    }
}
fn unwrap_to1<From: Index, E: Default + Clone>(
    to1: Option<To1<From, E>>,
    len: u64,
) -> To1<From, E> {
    match to1 {
        None => To1::with_entries(vec![Default::default(); len as usize]),
        Some(to1) => to1,
    }
}

index!(StringIndex);
index!(SequenceIndex);

#[derive(Debug)]
pub enum UmbError {
    FileError(file_input::InputError),
    NoIndex,
    MalformattedIndex(serde_json::Error),

    // TODO: Include more context here, i.e. in which file the error occurred
    MalformedCsr(csr::CsrError),

    // TODO: Include context here. Currently, for simplicity, the entire error is omitted, as
    //  `To1Error` is generic. However, here, we should know for which types To1 is used, and can
    //  provide an appropriate custom error for each.
    MalformedTo1,

    MalformedBooleanTo1(to1::BoolTo1Error),

    /// `state-is-markovian.bin` must only be present if `time` is `urgent-stochastic`
    StateIsMarkovianForInvalidModelType,

    RewardsError(annotations::AnnotationError<Infallible>),
    AtomicPropositionsError(annotations::AnnotationError<BoolTo1Error>),
}

impl From<std::io::Error> for UmbError {
    fn from(value: std::io::Error) -> Self {
        UmbError::FileError(file_input::InputError::IoError(value))
    }
}

impl From<serde_json::Error> for UmbError {
    fn from(value: serde_json::Error) -> Self {
        UmbError::MalformattedIndex(value)
    }
}
impl From<csr::CsrError> for UmbError {
    fn from(value: csr::CsrError) -> Self {
        UmbError::MalformedCsr(value)
    }
}
impl<E> From<to1::To1Error<E>> for UmbError {
    fn from(_value: to1::To1Error<E>) -> Self {
        UmbError::MalformedTo1
    }
}
impl From<to1::BoolTo1Error> for UmbError {
    fn from(value: to1::BoolTo1Error) -> Self {
        UmbError::MalformedBooleanTo1(value)
    }
}
