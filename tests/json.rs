#![cfg(feature = "json")]

use fixture_scheduler::{round_robin, schedule_to_json, Fixture, Round};

#[test]
fn fixture_json_escapes_special_characters() {
    let fixture = Fixture {
        home: "Foo \"Bar\"".to_string(),
        away: "Back\\slash".to_string(),
    };
    assert_eq!(
        fixture.to_json(),
        "{\"home\":\"Foo \\\"Bar\\\"\",\"away\":\"Back\\\\slash\"}"
    );
}

#[test]
fn round_json_renders_bye_as_null_when_absent() {
    let round = Round {
        number: 1,
        fixtures: vec![Fixture { home: "A".to_string(), away: "B".to_string() }],
        bye: None,
    };
    assert_eq!(
        round.to_json(),
        "{\"number\":1,\"fixtures\":[{\"home\":\"A\",\"away\":\"B\"}],\"bye\":null}"
    );
}

#[test]
fn round_json_renders_bye_team_when_present() {
    let round = Round { number: 2, fixtures: vec![], bye: Some("C".to_string()) };
    assert_eq!(round.to_json(), "{\"number\":2,\"fixtures\":[],\"bye\":\"C\"}");
}

#[test]
fn schedule_json_matches_round_robin_output() {
    let rounds = round_robin(&["A", "B"]).unwrap();
    assert_eq!(
        schedule_to_json(&rounds),
        "[{\"number\":1,\"fixtures\":[{\"home\":\"A\",\"away\":\"B\"}],\"bye\":null}]"
    );
}
