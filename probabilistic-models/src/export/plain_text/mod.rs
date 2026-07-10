use crate::Model;
use crate::base_model::{BaseModel, Mdp};
use crate::traits::ReadStateSpace;
use crate::valuations::{Type, ValuationBits, Valuations};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use typed_index_collections::{Index, RawIndex};

impl<M: BaseModel, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
    Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
{
    #[must_use]
    pub fn tra_file(&self) -> TraFile<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals> {
        TraFile { model: self }
    }
}

pub struct TraFile<'a, M: BaseModel, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals> {
    model: &'a Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>,
}

impl<SI: Index, CI: Index, BI: Index, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
    TraFile<'_, Mdp<SI, CI, BI>, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
{
    pub fn write_to_file(&self, destination: impl AsRef<Path>) -> std::io::Result<()> {
        write!(File::create(destination)?, "{}", self)
    }
}

impl<SI: Index, CI: Index, BI: Index, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals> Display
    for TraFile<'_, Mdp<SI, CI, BI>, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.model.states().len())?;
        write!(f, " {}", self.model.choices().len())?;
        writeln!(f, " {}", self.model.branches().len())?;
        let mut prev_state = None;
        let mut prev_choice = Some(CI::from_raw(CI::RawType::zero()));
        let mut choice_counter = 0;
        for (state, choice, branch) in self.model.base.state_choice_branch_triples() {
            if prev_state != Some(state) {
                choice_counter = 0;
            }
            let destination = self.model.base.branch_destinations[branch];
            let probability = self.model.base.branch_probabilities[branch];
            writeln!(
                f,
                "{} {} {} {}",
                state.raw(),
                choice_counter,
                destination.raw(),
                probability
            )?;
            if prev_choice != Some(choice) {
                choice_counter += 1;
            }
            prev_state = Some(state);
            prev_choice = Some(choice);
        }
        Ok(())
    }
}

impl<
    SI: Index,
    CI: Index,
    CEI: Index,
    VI: Index,
    M: BaseModel,
    Ini,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
> Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Valuations<SI, CI, CEI, VI>>
{
    #[must_use]
    pub fn sta_file(
        &self,
    ) -> StaFile<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Valuations<SI, CI, CEI, VI>> {
        StaFile { model: self }
    }
}

pub struct StaFile<'a, M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals> {
    model: &'a Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>,
}
impl<
    SI: Index,
    CI: Index,
    CEI: Index,
    VI: Index,
    M: BaseModel<StateIndex = SI>,
    Ini,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
> StaFile<'_, M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Valuations<SI, CI, CEI, VI>>
{
    pub fn write_to_file(&self, destination: impl AsRef<Path>) -> std::io::Result<()> {
        write!(File::create(destination)?, "{}", self)
    }
}

impl<
    SI: Index,
    CI: Index,
    CEI: Index,
    VI: Index,
    M: BaseModel<StateIndex = SI>,
    Ini,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
> Display
    for StaFile<'_, M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Valuations<SI, CI, CEI, VI>>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let classes = self.model.state_valuations.classes();
        assert_eq!(
            classes.len(),
            1,
            "Can only print `.sta` file for models with a single valuation class"
        );
        let class = self.model.state_valuations.class(classes.index(0));

        let mut first = true;
        for variable in class.entries() {
            if !first {
                write!(f, " ")?;
            }
            write!(f, "{}", variable.name)?;
            first = false;
        }
        writeln!(f)?;

        for state in self.model.states() {
            write!(f, "{}:(", state.raw())?;
            let valuation = self.model.state_valuations.entry(state);
            let mut first = true;
            for (variable_index, variable) in class.entries().enumerate() {
                if !first {
                    write!(f, " ")?;
                }
                match variable.variable_type {
                    Type::Bool => {
                        write!(f, "{}", valuation.evaluate_bool(variable_index))?;
                    }
                    Type::Int => {
                        write!(f, "{}", valuation.evaluate_int(variable_index))?;
                    }
                    Type::Uint => {
                        write!(f, "{}", valuation.evaluate_int(variable_index))?;
                    }
                    Type::Double => {
                        write!(f, "{}", valuation.evaluate_double(variable_index))?;
                    }
                    Type::Rational => {
                        let (num, denom) = valuation.evaluate_rational(variable_index);
                        write!(f, "{num}/{denom}")?;
                    }
                    Type::String => {
                        write!(f, "\"{}\"", valuation.evaluate_string(variable_index))?;
                    }
                }
                first = false;
            }
            writeln!(f, ")")?;
        }
        Ok(())
    }
}
