use crate::Model;
use crate::base_model::{BaseModel, ChoiceBranchGraph, SinglePlayer};
use crate::traits::ReadStateSpace;
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use typed_index_collections::{Index, RawIndex};

impl<M: ReadStateSpace, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, P>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, P>
{
    #[must_use]
    pub fn tra_file(
        &self,
    ) -> TraFile<'_, M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, P> {
        TraFile { model: self }
    }
}

pub struct TraFile<'a, M: ReadStateSpace, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, P> {
    model: &'a Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, P>,
}

impl<M: ReadStateSpace, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, P>
    TraFile<'_, M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, P>
where
    Self: Display,
{
    pub fn write_to_file(&self, destination: impl AsRef<Path>) -> std::io::Result<()> {
        write!(File::create(destination)?, "{}", self)
    }
}

// Accepts owned `Mdp`s as well as their `as_ref`/`as_mut` views
impl<G, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, P> Display
    for TraFile<
        '_,
        BaseModel<G, SinglePlayer>,
        Ini,
        ChLabel,
        BrLabel,
        Obs,
        APs,
        Rew,
        Ann,
        StateVals,
        P,
    >
where
    G: ReadStateSpace + Borrow<ChoiceBranchGraph<G::StateIndex, G::ChoiceIndex, G::BranchIndex>>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.model.states().len())?;
        write!(f, " {}", self.model.choices().len())?;
        writeln!(f, " {}", self.model.branches().len())?;
        let mut prev_state = None;
        let mut prev_choice = Some(G::ChoiceIndex::from_raw(
            <G::ChoiceIndex as Index>::RawType::zero(),
        ));
        let mut choice_counter = 0;
        for (state, choice, branch) in self.model.base.state_choice_branch_triples() {
            if prev_state != Some(state) {
                choice_counter = 0;
            } else if prev_choice != Some(choice) {
                choice_counter += 1;
            }
            let destination = self.model.base.graph.borrow().branch_destinations[branch];
            let probability = self.model.base.graph.borrow().branch_probabilities[branch];
            writeln!(
                f,
                "{} {} {} {}",
                state.raw(),
                choice_counter,
                destination.raw(),
                probability
            )?;
            prev_state = Some(state);
            prev_choice = Some(choice);
        }
        Ok(())
    }
}
