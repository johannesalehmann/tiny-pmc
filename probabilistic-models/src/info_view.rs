use crate::annotations::{AtomicPropositions, RewardAnnotations};
use crate::base_model::{
    BaseModelEnum, Dtmc, Mdp, NonstochasticGame, ReadPlayers, StochasticGame, TransitionSystem,
};
use crate::initial_states::{InitialStates, SingleInitialState};
use crate::labels::Labels;
use crate::predecessors::Predecessors;
use crate::traits::ReadStateSpace;
use crate::valuations::Valuations;
use crate::{BranchIndex, ChoiceIndex, Ix, Model};
use private::WriteInfo;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use typed_index_collections::Index;

const MAX_LENGTH: usize = 19;
const BULLET_INDENT: usize = 2;

impl<
    B: WriteInfo,
    Ini: WriteInfo,
    ChLabel: WriteInfo,
    BrLabel: WriteInfo,
    Obs: WriteInfo,
    APs: WriteInfo,
    Rew: WriteInfo,
    Ann: WriteInfo,
    StateVals: WriteInfo,
    Preds: WriteInfo,
> Model<B, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    pub fn info(
        &self,
    ) -> InfoView<'_, B, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds> {
        InfoView { model: self }
    }
}
pub struct InfoView<'a, B, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds> {
    model: &'a Model<B, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>,
}

impl<
    B: WriteInfo,
    Ini: WriteInfo,
    ChLabel: WriteInfo,
    BrLabel: WriteInfo,
    Obs: WriteInfo,
    APs: WriteInfo,
    Rew: WriteInfo,
    Ann: WriteInfo,
    StateVals: WriteInfo,
    Preds: WriteInfo,
> Display for InfoView<'_, B, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.model.base.write_info(f)?;
        self.model.initial.write_info(f)?;
        self.model.choice_labels.write_info(f)?;
        self.model.branch_labels.write_info(f)?;
        self.model.observations.write_info(f)?;
        self.model.atomic_propositions.write_info(f)?;
        self.model.rewards.write_info(f)?;
        self.model.annotations.write_info(f)?;
        self.model.state_valuations.write_info(f)?;
        self.model.predecessors.write_info(f)?;
        Ok(())
    }
}

mod private {
    use std::fmt::Formatter;

    pub trait WriteInfo {
        fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
    }
}

impl<TS: WriteInfo, D: WriteInfo, M: WriteInfo, NG: WriteInfo, SG: WriteInfo> WriteInfo
    for BaseModelEnum<TS, D, M, NG, SG>
{
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseModelEnum::TransitionSystem(ts) => ts.write_info(f),
            BaseModelEnum::Dtmc(dtmc) => dtmc.write_info(f),
            BaseModelEnum::Mdp(mdp) => mdp.write_info(f),
            BaseModelEnum::NonstochasticGame(nsg) => nsg.write_info(f),
            BaseModelEnum::StochasticGame(sg) => sg.write_info(f),
        }
    }
}

impl<B: WriteInfo> WriteInfo for &B {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        B::write_info(self, f)
    }
}

impl<B: WriteInfo> WriteInfo for &mut B {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        B::write_info(self, f)
    }
}

impl WriteInfo for Infallible {
    fn write_info(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}

impl WriteInfo for () {
    fn write_info(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl<B: WriteInfo> WriteInfo for Option<B> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Some(b) => b.write_info(f),
            None => Ok(()),
        }
    }
}

impl<I: Ix> WriteInfo for Mdp<I> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_info(f, "Model type", "Markov decision process")?;
        write_info(f, "States", self.states().len())?;
        write_info(f, "Choices", self.choices().len())?;
        write_info(f, "Branches", self.branches().len())
    }
}

impl<I: Ix> WriteInfo for Dtmc<I> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_info(f, "Model type", "discrete-time Markov chain")?;
        write_info(f, "States", self.states().len())?;
        write_info(f, "Branches", self.branches().len())
    }
}

impl<I: Ix> WriteInfo for TransitionSystem<I> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_info(f, "Model type", "transition system")?;
        write_info(f, "States", self.states().len())?;
        write_info(f, "Choices", self.choices().len())
    }
}

impl<I: Ix> WriteInfo for StochasticGame<I> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_info(f, "Model type", "stochastic game")?;
        write_info(f, "States", self.states().len())?;
        write_info(f, "Choices", self.choices().len())?;
        write_info(f, "Branches", self.branches().len())?;
        write_players(f, self)
    }
}

impl<I: Ix> WriteInfo for NonstochasticGame<I> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_info(f, "Model type", "nonstochastic game")?;
        write_info(f, "States", self.states().len())?;
        write_info(f, "Choices", self.choices().len())?;
        write_players(f, self)
    }
}

impl<I: Ix> WriteInfo for InitialStates<I> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_info(f, "Initial states", self.true_values().into_iter().count())
    }
}
impl<I: Ix> WriteInfo for SingleInitialState<I> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_info(f, "Initial states", 1)
    }
}

pub trait PrintLabelInfo {
    const LABEL_INFO_NAME: &'static str;
}

impl PrintLabelInfo for String {
    const LABEL_INFO_NAME: &'static str = "String";
}

impl<I: Ix, AI: Index, E: PrintLabelInfo> WriteInfo for Labels<ChoiceIndex<I>, AI, E> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Choices are labelled (with `{}`)", E::LABEL_INFO_NAME)
    }
}
impl<I: Ix, AI: Index, E: PrintLabelInfo> WriteInfo for Labels<BranchIndex<I>, AI, E> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Branches are labelled (with `{}`)", E::LABEL_INFO_NAME)
    }
}

impl<I: Ix> WriteInfo for AtomicPropositions<I> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Atomic propositions:")?;
        for (name, values) in self {
            let count = values.true_values().into_iter().count();
            write_bullet(f, format_args!("{name} ({})", states(count)))?;
        }
        Ok(())
    }
}

impl<I: Ix> WriteInfo for RewardAnnotations<I> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Reward structures:")?;
        for (name, rewards) in self {
            let applies_to = match (rewards.states.is_some(), rewards.choices.is_some()) {
                (true, true) => "states, choices",
                (true, false) => "states",
                (false, true) => "choices",
                (false, false) => "does not apply to any entities",
            };
            write_bullet(f, format_args!("{name} ({applies_to})"))?;
        }
        Ok(())
    }
}

impl<I: Ix, EntityIdx: Index> WriteInfo for Valuations<I, EntityIdx> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Model stores valuations")
    }
}

impl<I: Ix> WriteInfo for Predecessors<I> {
    fn write_info(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Model stores predecessors")
    }
}

fn write_players(
    f: &mut Formatter<'_>,
    model: impl ReadPlayers + ReadStateSpace,
) -> std::fmt::Result {
    writeln!(f, "Players:")?;
    for player in model.players() {
        let count = model
            .states()
            .into_iter()
            .filter(|&state| model.state_owner(state) == player)
            .count();
        write_bullet(
            f,
            format_args!("{} ({})", model.player_name(player), states(count)),
        )?;
    }
    Ok(())
}

fn states(count: usize) -> String {
    match count {
        1 => "1 state".to_string(),
        _ => format!("{count} states"),
    }
}

fn write_info(f: &mut Formatter<'_>, label: impl Display, value: impl Display) -> std::fmt::Result {
    let label = label.to_string();
    let padding = MAX_LENGTH.saturating_sub(label.len());
    writeln!(f, "{label}: {:<padding$}{value}", "")
}

fn write_bullet(f: &mut Formatter<'_>, value: impl Display) -> std::fmt::Result {
    writeln!(f, "{:<width$}* {value}", "", width = BULLET_INDENT)
}

#[cfg(test)]
mod tests {
    use crate::annotations::{
        AtomicPropositions, EntityRewards, RewardAnnotations, StateChoiceRewards, TypedAnnotation,
    };
    use crate::initial_states::{InitialStates, SingleInitialState};
    use crate::labels::{BranchLabels, ChoiceLabels};
    use crate::{ChoiceIndex, Index, Model, StateIndex, Uniform, mdp, smg};
    use typed_index_collections::To1;

    type U = Uniform<usize>;

    #[test]
    fn markov_decision_process() {
        mdp!(base = { s0 -> 0.5: s0 & 0.5: s1, s1 -> 1.0: s0 });
        assert_eq!(
            Model::new(base).info().to_string(),
            "\
Model type:          Markov decision process
States:              2
Choices:             2
Branches:            3
"
        );
    }

    #[test]
    fn discrete_time_markov_chain() {
        mdp!(base = { s0 -> 1.0: s0 });
        let base = base.try_into_dtmc().unwrap();
        assert_eq!(
            Model::new(base).info().to_string(),
            "\
Model type:          discrete-time Markov chain
States:              1
Branches:            1
"
        );
    }

    #[test]
    fn transition_system() {
        mdp!(base = { s0 -> 1.0: s0, s0 -> 1.0: s0 });
        let base = base.try_into_transition_system().unwrap();
        assert_eq!(
            Model::new(base).info().to_string(),
            "\
Model type:          transition system
States:              1
Choices:             2
"
        );
    }

    #[test]
    fn nonstochastic_game() {
        smg!(base = { players: [p1], s0 (p1) -> 1.0: s0 });
        let base = base.try_into_nonstochastic_game().unwrap();
        assert_eq!(
            Model::new(base).info().to_string(),
            "\
Model type:          nonstochastic game
States:              1
Choices:             1
Players:
  * p1 (1 state)
"
        );
    }

    #[test]
    fn stochastic_game() {
        smg!(base = {
            players: [p1, p2],
            s0 (p1) -> 1.0: s1,
            s1 (p1) -> 1.0: s2,
            s2 (p2) -> 1.0: s0
        });
        assert_eq!(
            Model::new(base).info().to_string(),
            "\
Model type:          stochastic game
States:              3
Choices:             3
Branches:            3
Players:
  * p1 (2 states)
  * p2 (1 state)
"
        );
    }

    #[test]
    fn initial_state_counts() {
        for (initial, count) in [
            (vec![false, false, false], 0),
            (vec![false, true, false], 1),
            (vec![true, true, true], 3),
        ] {
            mdp!(base = { s0 -> 1.0: s0, s1 -> 1.0: s1, s2 -> 1.0: s2 });
            let model =
                Model::new(base).with_initial_states(InitialStates::<U>::with_entries(initial));
            assert!(
                model
                    .info()
                    .to_string()
                    .contains(&format!("Initial states:      {count}\n"))
            );
        }

        mdp!(base = { s0 -> 1.0: s0 });
        let model = Model::new(base).with_initial_state(s0);
        let _: &Model<_, SingleInitialState<U>, _, _, _, _, _, _, _, _> = &model;
        assert!(
            model
                .info()
                .to_string()
                .contains("Initial states:      1\n")
        );
    }

    #[test]
    fn absent_components_print_nothing() {
        mdp!(base = { s0 -> 1.0: s0 });
        let model = Model {
            base,
            initial: None::<InitialStates<U>>,
            choice_labels: (),
            branch_labels: (),
            observations: (),
            atomic_propositions: None::<AtomicPropositions<U>>,
            rewards: (),
            annotations: (),
            state_valuations: (),
            predecessors: (),
        };
        assert_eq!(model.info().to_string().lines().count(), 4);
    }

    fn full_model() -> impl std::fmt::Display {
        mdp!(base = { s0 -> 0.5: s0 & 0.5: s1, s1 -> 1.0: s0 });
        let mut aps = AtomicPropositions::<U>::new();
        for name in ["goal", "init"] {
            aps.add_entry(
                name.to_string(),
                TypedAnnotation::with_identity_distribution_and_entries(To1::with_entries(vec![
                    false, true,
                ])),
            );
        }
        let mut rewards = RewardAnnotations::<U>::new();
        let state_rewards = || -> EntityRewards<StateIndex<U>> {
            EntityRewards::with_identity_distribution_and_entries(To1::with_entries(vec![0.0; 2]))
        };
        let choice_rewards = || -> EntityRewards<ChoiceIndex<U>> {
            EntityRewards::with_identity_distribution_and_entries(To1::with_entries(vec![0.0; 2]))
        };
        rewards.add_entry(
            "steps".to_string(),
            StateChoiceRewards::with_state_rewards(state_rewards()),
        );
        rewards.add_entry(
            "time".to_string(),
            StateChoiceRewards::with_choice_rewards(choice_rewards()),
        );
        rewards.add_entry(
            "cost".to_string(),
            StateChoiceRewards::with_state_and_choice_rewards(state_rewards(), choice_rewards()),
        );
        rewards.add_entry("unused".to_string(), StateChoiceRewards::new());
        let model = Model {
            base,
            initial: SingleInitialState {
                index: StateIndex::<U>::from_raw(0),
            },
            choice_labels: ChoiceLabels::<U, String>::new(),
            branch_labels: BranchLabels::<U, String>::new(),
            observations: (),
            atomic_propositions: aps,
            rewards,
            annotations: (),
            state_valuations: (),
            predecessors: (),
        }
        .compute_predecessors();
        model.info().to_string()
    }

    // TODO: Also test the valuations line ("Model stores valuations").
    #[test]
    fn all_components() {
        assert_eq!(
            full_model().to_string(),
            "\
Model type:          Markov decision process
States:              2
Choices:             2
Branches:            3
Initial states:      1
Choices are labelled (with `String`)
Branches are labelled (with `String`)
Atomic propositions:
  * goal (1 state)
  * init (1 state)
Reward structures:
  * steps (states)
  * time (choices)
  * cost (states, choices)
  * unused (does not apply to any entities)
Model stores predecessors
"
        );
    }

    #[test]
    fn values_are_aligned() {
        let output = full_model().to_string();
        for line in output.lines() {
            if line.starts_with(' ') {
                assert_eq!(line.find('*'), Some(super::BULLET_INDENT), "`{line}`");
            } else if let Some(colon) = line.find(": ") {
                let column = colon + 2 + line[colon + 2..].find(|c| c != ' ').unwrap();
                assert_eq!(column, super::MAX_LENGTH + 2, "`{line}` is misaligned");
            }
        }
    }
}
