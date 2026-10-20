use crate::traits::ReadStateSpace;
use crate::{BranchIndex, ChoiceIndex, StateIndex};

pub trait BuildStates: ReadStateSpace {
    fn add_state(&mut self) -> StateIndex<Self::Ix>;
}

macro_rules! derive_build_states {
    ($subcomponent:ident) => {
        fn add_state(&mut self) -> $crate::StateIndex<Self::Ix> {
            self.$subcomponent.add_state()
        }
    };
}
pub(crate) use derive_build_states;

pub trait BuildChoiceBranchGraph: ReadStateSpace {
    fn add_choice(&mut self) -> ChoiceIndex<Self::Ix>;
    fn add_branch(
        &mut self,
        probability: f64,
        destination: StateIndex<Self::Ix>,
    ) -> BranchIndex<Self::Ix>;

    fn add_choice_from_slice(
        &mut self,
        branches: &[(f64, StateIndex<Self::Ix>)],
    ) -> ChoiceIndex<Self::Ix> {
        let choice = self.add_choice();
        for &(probability, destination) in branches {
            self.add_branch(probability, destination);
        }
        choice
    }
}

macro_rules! derive_build_choice_branch_graph {
    ($subcomponent:ident) => {
        fn add_choice(&mut self) -> $crate::ChoiceIndex<Self::Ix> {
            self.$subcomponent.add_choice()
        }

        fn add_branch(
            &mut self,
            probability: f64,
            destination: $crate::StateIndex<Self::Ix>,
        ) -> $crate::BranchIndex<Self::Ix> {
            self.$subcomponent.add_branch(probability, destination)
        }
    };
}
pub(crate) use derive_build_choice_branch_graph;

pub trait BuildChoiceGraph: ReadStateSpace {
    fn add_choice(&mut self, destination: StateIndex<Self::Ix>) -> ChoiceIndex<Self::Ix>;
}

macro_rules! derive_build_choice_graph {
    ($subcomponent:ident) => {
        fn add_choice(
            &mut self,
            destination: $crate::StateIndex<Self::Ix>,
        ) -> $crate::ChoiceIndex<Self::Ix> {
            self.$subcomponent.add_choice(destination)
        }
    };
}
pub(crate) use derive_build_choice_graph;

pub trait BuildBranchGraph: ReadStateSpace {
    fn add_branch(
        &mut self,
        probability: f64,
        destination: StateIndex<Self::Ix>,
    ) -> BranchIndex<Self::Ix>;
}

macro_rules! derive_build_branch_graph {
    ($subcomponent:ident) => {
        fn add_branch(
            &mut self,
            probability: f64,
            destination: $crate::StateIndex<Self::Ix>,
        ) -> $crate::BranchIndex<Self::Ix> {
            self.$subcomponent.add_branch(probability, destination)
        }
    };
}
pub(crate) use derive_build_branch_graph;

pub trait ClearModel {
    fn clear(&mut self);
}

impl<T: BuildStates> BuildStates for &mut T {
    fn add_state(&mut self) -> StateIndex<Self::Ix> {
        (**self).add_state()
    }
}

impl<T: BuildChoiceBranchGraph> BuildChoiceBranchGraph for &mut T {
    fn add_choice(&mut self) -> ChoiceIndex<Self::Ix> {
        (**self).add_choice()
    }

    fn add_branch(
        &mut self,
        probability: f64,
        destination: StateIndex<Self::Ix>,
    ) -> BranchIndex<Self::Ix> {
        (**self).add_branch(probability, destination)
    }
}

impl<T: BuildChoiceGraph> BuildChoiceGraph for &mut T {
    fn add_choice(&mut self, destination: StateIndex<Self::Ix>) -> ChoiceIndex<Self::Ix> {
        (**self).add_choice(destination)
    }
}

impl<T: BuildBranchGraph> BuildBranchGraph for &mut T {
    fn add_branch(
        &mut self,
        probability: f64,
        destination: StateIndex<Self::Ix>,
    ) -> BranchIndex<Self::Ix> {
        (**self).add_branch(probability, destination)
    }
}

impl<T: ClearModel> ClearModel for &mut T {
    fn clear(&mut self) {
        (**self).clear();
    }
}
