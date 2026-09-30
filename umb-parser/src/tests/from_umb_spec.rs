use probabilistic_models::{StateIndex, mdp};

#[test]
fn parse_from_umb_spec() {
    let parsed = crate::parse_umb("examples/from_umb_spec/mdp-gzip.umb").unwrap();

    mdp!(expected_mdp = {
        x0false -> 0.2: x2false & 0.8: x2true,
        x0false -> 0.5: x1false & 0.5: x2false,

        x1false -> 0.9: x0false & 0.1: x1false,

        x2false -> 1.0: x2false,

        x2true -> 1.0: x2true
    });
    assert_eq!(parsed.base, expected_mdp);

    let initial = parsed.initial.unwrap();
    assert_eq!(initial.entries(), &[true, false, false, false]);

    let aps = parsed.atomic_propositions.unwrap();
    assert_eq!(aps.names().entries(), &["g"]);
    assert_eq!(aps["g"].values().as_slice(), &[false, false, false, true]);

    let rewards = parsed.rewards.unwrap();
    assert_eq!(rewards.names().entries(), &["r"]);
    assert_eq!(rewards["r"].states, None);
    assert_eq!(
        rewards["r"].choices.as_ref().unwrap().values().as_slice(),
        &[1.0, 0.0, 0.0, 0.0, 0.0]
    );

    // TODO: Check action labels once they are supported by UMB
}
