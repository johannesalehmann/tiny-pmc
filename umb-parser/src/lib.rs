mod annotations;
mod archive;
mod csr;
mod file_input;
mod index;
#[cfg(test)]
mod tests;
mod to1;

use crate::archive::UmbArchive;
use crate::index::{Time, Type, TypeKind};
use crate::to1::BoolTo1Error;
use probabilistic_models::annotations::{AtomicPropositions, RewardAnnotations};
use probabilistic_models::base_model::{ChoiceBranchGraph, Mdp};
use probabilistic_models::initial_states::InitialStates;
use probabilistic_models::typed_index_collections::index;
use probabilistic_models::{BranchIndex, ChoiceIndex, DefaultIx, Index, Model, StateIndex};
use std::convert::Infallible;
use std::path::Path;

type SI = StateIndex<DefaultIx>;
type CI = ChoiceIndex<DefaultIx>;
type BI = BranchIndex<DefaultIx>;
pub fn parse_umb<P: AsRef<Path>>(
    path: P,
) -> Result<
    Model<
        Mdp<DefaultIx>,
        Option<InitialStates<DefaultIx>>,
        (),
        (),
        (),
        Option<AtomicPropositions<DefaultIx>>,
        Option<RewardAnnotations<DefaultIx>>,
        (),
        (),
        (),
    >,
    UmbError,
> {
    let bytes = file_input::get_contents(path).map_err(UmbError::FileError)?;
    let mut archive = UmbArchive::new(&bytes)?;
    let index = index::parse_index(archive.read_required_file("index.json")?)?;
    let sizes = &index.transition_system;

    assert_eq!(
        sizes.num_players, 1,
        "The UMB parser currently only supports 1-player games"
    );
    assert_eq!(
        sizes.time,
        Time::Discrete,
        "The UMB parser currently only supports discrete-time models"
    );
    if !matches!(
        sizes.branch_probability_type,
        Some(Type {
            kind: TypeKind::Double,
            ..
        })
    ) {
        // TODO: Distinguish between unsupported and invalid types
        // TODO: For floats, check that the specified size is correct (or do so above)
        panic!("Missing, invalid or unsupported branch probability type")
    }

    let state_to_choice = archive.csr_or_identity("state-to-choices.bin", sizes.num_states)?;
    let choice_to_branch = archive.csr_or_identity("choice-to-branches.bin", sizes.num_choices)?;
    let branch_destinations =
        to1::parse_to1::<BI, u64, _>(archive.read_required_file("branch-to-target.bin")?)?
            .map(|i| SI::from_raw(*i as usize));
    let branch_probabilities =
        to1::parse_to1::<BI, f64, _>(archive.read_required_file("branch-to-probability.bin")?)?;
    let initial = archive
        .read_file("state-is-initial.bin")
        .map(|contents| to1::parse_to1_bool::<SI, _>(contents, sizes.num_states as usize))
        .transpose()?;

    let mut atomic_propositions = None;
    let mut rewards = None;
    for (group, annotations) in index.annotations.iter().flatten() {
        match group.as_str() {
            "aps" => {
                atomic_propositions =
                    Some(annotations::parse_aps(&mut archive, annotations, sizes)?)
            }
            "rewards" => rewards = Some(annotations::parse_rewards(&mut archive, annotations)?),
            // Ignore other annotations for now. They will be reported as unread further below.
            _ => {}
        }
    }

    for path in archive.unread_files() {
        println!("Warning: The UMB parser ignored the unsupported or unexpected file {path}");
    }

    let mdp = Mdp::with_graph(ChoiceBranchGraph {
        state_to_choice,
        choice_to_branch,
        branch_probabilities,
        branch_destinations,
    });

    Ok(Model {
        base: mdp,
        initial,
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

index!(StringIndex);
index!(SequenceIndex);
index!(AnnotationEntryIndex);

#[derive(Debug)]
pub enum UmbError {
    FileError(file_input::InputError),
    MissingFile(String),
    MalformattedIndex(serde_json::Error),

    // TODO: Include more context here, i.e. in which file the error occurred
    MalformedCsr(csr::CsrError),

    // TODO: Include context here. Currently, for simplicity, the entire error is omitted, as
    //  `To1Error` is generic. However, here, we should know for which types To1 is used, and can
    //  provide an appropriate custom error for each.
    MalformedTo1,

    MalformedBooleanTo1(to1::BoolTo1Error),

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
