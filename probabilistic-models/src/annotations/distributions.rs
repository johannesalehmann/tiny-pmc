use crate::Index;
use crate::csr::{Csr, CsrRange};
use crate::to1::To1;
use std::marker::PhantomData;

pub trait DeltaDistribution<From: Index, To: Index> {
    fn annotation_of_state(&self, state: From) -> To;
}

pub trait Distribution<From: Index, To: Index> {
    fn annotations_of_state(&self, state: From) -> CsrRange<To>;
    fn probability(&self, index: To) -> f32;
}

pub struct ProbabilisticDistribution<From: Index, To: Index> {
    entity_to_annotations: Csr<From, To>,
    probabilities: To1<To, f32>,
}

impl<From: Index, To: Index> Distribution<From, To> for ProbabilisticDistribution<From, To> {
    fn annotations_of_state(&self, state: From) -> CsrRange<To> {
        self.entity_to_annotations.get(state).unwrap()
    }

    fn probability(&self, index: To) -> f32 {
        self.probabilities[index]
    }
}

pub struct IdentityDistribution<From: Index, To: Index> {
    phantom_data: PhantomData<(From, To)>,
}

impl<From: Index, To: Index> Distribution<From, To> for IdentityDistribution<From, To> {
    fn annotations_of_state(&self, state: From) -> CsrRange<To> {
        CsrRange::identity(state)
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
    fn annotations_of_state(&self, state: From) -> CsrRange<To> {
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
