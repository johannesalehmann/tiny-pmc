use crate::archive::UmbArchive;
use crate::index::{Annotation, EntityKind, TransitionSystem, Type, TypeKind};
use crate::to1::{BoolTo1Error, To1Error, parse_to1, parse_to1_bool};
use crate::{CI, SI, UmbError};
use probabilistic_models::annotations::{
    AtomicPropositions, EntityRewards, RewardAnnotations, StateChoiceRewards, TypedAnnotation,
};
use probabilistic_models::typed_index_collections::To1;
use probabilistic_models::{DefaultIx, Index};
use std::collections::HashMap;
use std::convert::Infallible;

#[derive(Debug)]
pub enum AnnotationError<E> {
    InvalidValues {
        path: String,
        error: To1Error<E>,
    },
    UnsupportedType {
        id: String,
        value_type: Type,
    },
    /// Annotations whose values are distributions are not supported yet
    UnsupportedDistribution {
        id: String,
    },
    UnsupportedTarget {
        id: String,
        target: EntityKind,
    },
    MissingTarget {
        id: String,
        target: EntityKind,
    },
}

fn values_path(group: &str, id: &str, target: EntityKind) -> String {
    format!(
        "annotations/{group}/{id}/{}/values.bin",
        target.directory_name()
    )
}

fn check_supported<E>(
    id: &str,
    annotation: &Annotation,
    kind: TypeKind,
) -> Result<(), AnnotationError<E>> {
    // TODO: For floats, check that the specified size is correct
    if annotation.value_type.kind != kind {
        return Err(AnnotationError::UnsupportedType {
            id: id.into(),
            value_type: annotation.value_type,
        });
    }
    if annotation.probability_type.is_some() {
        return Err(AnnotationError::UnsupportedDistribution { id: id.into() });
    }
    Ok(())
}

pub fn parse_aps(
    archive: &mut UmbArchive,
    annotations: &HashMap<String, Annotation>,
    sizes: &TransitionSystem,
) -> Result<AtomicPropositions<DefaultIx>, UmbError> {
    let error = UmbError::AtomicPropositionsError;
    let mut aps: AtomicPropositions<DefaultIx> = Default::default();
    for (id, annotation) in annotations {
        check_supported(id, annotation, TypeKind::Bool).map_err(error)?;
        if let Some(&target) = annotation
            .applies_to
            .iter()
            .find(|&&target| target != EntityKind::States)
        {
            return Err(error(AnnotationError::UnsupportedTarget {
                id: id.clone(),
                target,
            }));
        }
        if !annotation.applies_to.contains(&EntityKind::States) {
            return Err(error(AnnotationError::MissingTarget {
                id: id.clone(),
                target: EntityKind::States,
            }));
        }

        let path = values_path("aps", id, EntityKind::States);
        let values = parse_to1_bool::<SI, _>(
            archive.read_required_file(&path)?,
            sizes.num_states as usize,
        )
        .map_err(|e: BoolTo1Error| {
            error(AnnotationError::InvalidValues {
                path,
                error: e.into(),
            })
        })?;
        let name = annotation.alias.clone().unwrap_or_else(|| id.clone());
        aps.add_entry(
            name,
            TypedAnnotation::with_identity_distribution_and_entries(values),
        );
    }
    Ok(aps)
}

pub fn parse_rewards(
    archive: &mut UmbArchive,
    annotations: &HashMap<String, Annotation>,
) -> Result<RewardAnnotations<DefaultIx>, UmbError> {
    let error = UmbError::RewardsError;
    let mut rewards: RewardAnnotations<DefaultIx> = RewardAnnotations::new();
    for (id, annotation) in annotations {
        check_supported(id, annotation, TypeKind::Double).map_err(error)?;
        let mut rewards_entry: StateChoiceRewards<DefaultIx> = StateChoiceRewards::new();
        for &target in &annotation.applies_to {
            let path = values_path("rewards", id, target);
            match target {
                EntityKind::States => rewards_entry.add_state_rewards(
                    EntityRewards::with_identity_distribution_and_entries(parse_f64_values::<SI>(
                        archive, path,
                    )?),
                ),
                EntityKind::Choices => rewards_entry.add_choice_rewards(
                    EntityRewards::with_identity_distribution_and_entries(parse_f64_values::<CI>(
                        archive, path,
                    )?),
                ),
                _ => {
                    return Err(error(AnnotationError::UnsupportedTarget {
                        id: id.clone(),
                        target,
                    }));
                }
            }
        }
        let name = annotation.alias.clone().unwrap_or_else(|| id.clone());
        rewards.add_entry(name, rewards_entry);
    }
    Ok(rewards)
}

fn parse_f64_values<I: Index>(
    archive: &mut UmbArchive,
    path: String,
) -> Result<To1<I, f64>, UmbError> {
    parse_to1(archive.read_required_file(&path)?).map_err(|error: To1Error<Infallible>| {
        UmbError::RewardsError(AnnotationError::InvalidValues { path, error })
    })
}
