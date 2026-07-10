use std::marker::PhantomData;
use typed_index_collections::{Csr, Index, IndexRange, RawIndex, To1};

pub trait DeltaDistribution<From: Index, To: Index> {
    fn annotation_of_state(&self, state: From) -> To;
}

pub trait Distribution<From: Index, To: Index> {
    fn annotations_of_state(&self, state: From) -> IndexRange<To>;
    fn probability(&self, index: To) -> f32;
}

#[derive(Default)]
pub struct ProbabilisticDistribution<From: Index, To: Index> {
    entity_to_annotations: Csr<From, To>,
    probabilities: To1<To, f32>,
}

impl<From: Index, To: Index> Distribution<From, To> for ProbabilisticDistribution<From, To> {
    fn annotations_of_state(&self, state: From) -> IndexRange<To> {
        self.entity_to_annotations.index(state)
    }

    fn probability(&self, index: To) -> f32 {
        self.probabilities[index]
    }
}

#[derive(Default)]
pub struct IdentityDistribution<From: Index, To: Index> {
    phantom_data: PhantomData<(From, To)>,
}

impl<From: Index, To: Index> IdentityDistribution<From, To> {
    pub fn annotation_index(&self, from: From) -> To {
        To::from_raw(To::RawType::from_usize(from.raw().as_usize()))
    }
}

impl<From: Index, To: Index> Distribution<From, To> for IdentityDistribution<From, To> {
    fn annotations_of_state(&self, state: From) -> IndexRange<To> {
        IndexRange::with_single_index(To::from_raw(To::RawType::from_usize(
            state.raw().as_usize(),
        )))
    }

    fn probability(&self, _index: To) -> f32 {
        1.0
    }
}

pub enum MixedDistribution<From: Index, To: Index> {
    Probabilistic(ProbabilisticDistribution<From, To>),
    Identity(IdentityDistribution<From, To>),
}

impl<From: Index, To: Index> Distribution<From, To> for MixedDistribution<From, To> {
    fn annotations_of_state(&self, state: From) -> IndexRange<To> {
        match self {
            MixedDistribution::Probabilistic(dist) => dist.annotations_of_state(state),
            MixedDistribution::Identity(dist) => dist.annotations_of_state(state),
        }
    }

    fn probability(&self, index: To) -> f32 {
        match self {
            MixedDistribution::Probabilistic(dist) => dist.probability(index),
            MixedDistribution::Identity(dist) => dist.probability(index),
        }
    }
}
