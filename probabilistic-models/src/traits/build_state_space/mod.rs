use crate::traits::ReadStateSpace;

pub trait BuildStates: ReadStateSpace {
    fn add_state(&mut self) -> Self::StateIndex;
}

macro_rules! derive_build_states {
    ($subcomponent:ident) => {
        fn add_state(&mut self) -> Self::StateIndex {
            self.$subcomponent.add_state()
        }
    };
}
pub(crate) use derive_build_states;

pub trait BuildChoiceBranchGraph: ReadStateSpace {
    fn add_choice(&mut self) -> Self::ChoiceIndex;
    fn add_branch(&mut self, probability: f64, destination: Self::StateIndex) -> Self::BranchIndex;

    fn add_choice_from_slice(&mut self, branches: &[(f64, Self::StateIndex)]) -> Self::ChoiceIndex {
        let choice = self.add_choice();
        for &(probability, destination) in branches {
            self.add_branch(probability, destination);
        }
        choice
    }
}

macro_rules! derive_build_choice_branch_graph {
    ($subcomponent:ident) => {
        fn add_choice(&mut self) -> Self::ChoiceIndex {
            self.$subcomponent.add_choice()
        }

        fn add_branch(
            &mut self,
            probability: f64,
            destination: Self::StateIndex,
        ) -> Self::BranchIndex {
            self.$subcomponent.add_branch(probability, destination)
        }
    };
}
pub(crate) use derive_build_choice_branch_graph;

pub trait BuildChoiceGraph: ReadStateSpace {
    fn add_choice(&mut self, destination: Self::StateIndex) -> Self::ChoiceIndex;
}

macro_rules! derive_build_choice_graph {
    ($subcomponent:ident) => {
        fn add_choice(&mut self, destination: Self::StateIndex) -> Self::ChoiceIndex {
            self.$subcomponent.add_choice(destination)
        }
    };
}
pub(crate) use derive_build_choice_graph;

pub trait BuildBranchGraph: ReadStateSpace {
    fn add_branch(&mut self, probability: f64, destination: Self::StateIndex) -> Self::BranchIndex;
}

macro_rules! derive_build_branch_graph {
    ($subcomponent:ident) => {
        fn add_branch(
            &mut self,
            probability: f64,
            destination: Self::StateIndex,
        ) -> Self::BranchIndex {
            self.$subcomponent.add_branch(probability, destination)
        }
    };
}
pub(crate) use derive_build_branch_graph;

pub trait ClearModel {
    fn clear(&mut self);
}

impl<T: BuildStates> BuildStates for &mut T {
    fn add_state(&mut self) -> Self::StateIndex {
        (**self).add_state()
    }
}

impl<T: BuildChoiceBranchGraph> BuildChoiceBranchGraph for &mut T {
    fn add_choice(&mut self) -> Self::ChoiceIndex {
        (**self).add_choice()
    }

    fn add_branch(&mut self, probability: f64, destination: Self::StateIndex) -> Self::BranchIndex {
        (**self).add_branch(probability, destination)
    }
}

impl<T: BuildChoiceGraph> BuildChoiceGraph for &mut T {
    fn add_choice(&mut self, destination: Self::StateIndex) -> Self::ChoiceIndex {
        (**self).add_choice(destination)
    }
}

impl<T: BuildBranchGraph> BuildBranchGraph for &mut T {
    fn add_branch(&mut self, probability: f64, destination: Self::StateIndex) -> Self::BranchIndex {
        (**self).add_branch(probability, destination)
    }
}

impl<T: ClearModel> ClearModel for &mut T {
    fn clear(&mut self) {
        (**self).clear();
    }
}
