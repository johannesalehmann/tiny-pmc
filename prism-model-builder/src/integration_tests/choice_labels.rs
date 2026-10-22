use super::*;
use crate::choice_labels::{CommandIndexComponent, CommandIndices};
use probabilistic_models::labels::ChoiceLabels;
use probabilistic_models::traits::ReadChoiceLabels;

/// An unlabelled choice, two synchronised choices with action `a` and three deadlock states.
const SOURCE: &str = r#"
    mdp
    module m1
        x : [0..2] init 0;
        [] x=0 -> (x'=1);
        [a] x=0 -> (x'=2);
        [a] x=0 -> (x'=1);
    endmodule
    module m2
        y : bool init false;
        [a] true -> (y'=true);
    endmodule"#;

/// Labels of all choices, in choice order.
fn labels<M: ReadStateSpace + ReadChoiceLabels<E: Clone>>(model: &M) -> Vec<M::E> {
    model
        .choices()
        .into_iter()
        .map(|c| model.choice_label(c).clone())
        .collect()
}

#[test]
fn action_name_labels() {
    let model = build!(SOURCE, .with_mdp_model_type()).unwrap();
    let _: &ChoiceLabels<Ix, Option<String>> = &model.choice_labels;
    assert_eq!(
        valuations(&model),
        ["x=0,y=false", "x=1,y=false", "x=2,y=true", "x=1,y=true"]
    );
    let a = Some("a".to_string());
    assert_eq!(labels(&model), [None, a.clone(), a, None, None, None]);
}

#[test]
fn command_index_labels() {
    let model =
        build!(SOURCE, .with_mdp_model_type().label_choices_with_command_indices()).unwrap();
    let _: &ChoiceLabels<Ix, CommandIndices> = &model.choice_labels;
    let component = |module_index, command_index| CommandIndexComponent {
        module_index,
        command_index,
    };
    let synchronised = |command_index| CommandIndices::Synchronised {
        components: vec![component(0, command_index), component(1, 0)],
    };
    assert_eq!(
        labels(&model),
        [
            CommandIndices::Unsynchronised {
                module_index: 0,
                command_index: 0
            },
            synchronised(1),
            synchronised(2),
            CommandIndices::DeadlockFix,
            CommandIndices::DeadlockFix,
            CommandIndices::DeadlockFix,
        ]
    );
}

#[test]
fn without_choice_labels() {
    let model = build!(SOURCE, .without_choice_labels()).unwrap();
    let _: &() = &model.choice_labels;
}

#[test]
fn choice_label_switching() {
    let model = build!(SOURCE, .without_choice_labels().label_choices_with_action_names()).unwrap();
    let _: &ChoiceLabels<Ix, Option<String>> = &model.choice_labels;
    let model =
        build!(SOURCE, .without_choice_labels().label_choices_with_command_indices()).unwrap();
    let _: &ChoiceLabels<Ix, CommandIndices> = &model.choice_labels;
    let model =
        build!(SOURCE, .label_choices_with_command_indices().without_choice_labels()).unwrap();
    let _: &() = &model.choice_labels;
    let model =
        build!(SOURCE, .label_choices_with_command_indices().label_choices_with_action_names())
            .unwrap();
    let _: &ChoiceLabels<Ix, Option<String>> = &model.choice_labels;
}
