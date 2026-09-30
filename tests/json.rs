#![cfg(feature = "json")]

use fixture_scheduler::{
    round_robin, round_robin_teams, schedule_to_json, team_schedule_to_json, Fixture, Round, Team,
};

#[test]
fn team_json_includes_id_and_escaped_name() {
    let team = Team::new("osp", "The \"Ospreys\"");
    assert_eq!(team.to_json(), "{\"id\":\"osp\",\"name\":\"The \\\"Ospreys\\\"\"}");
}

#[test]
fn team_schedule_json_keeps_ids_and_names() {
    let teams = [Team::new("a", "Alpha"), Team::new("b", "Bravo")];
    let rounds = round_robin_teams(&teams).unwrap();
    assert_eq!(
        team_schedule_to_json(&rounds),
        "[{\"number\":1,\"fixtures\":[{\"home\":{\"id\":\"a\",\"name\":\"Alpha\"},\
         \"away\":{\"id\":\"b\",\"name\":\"Bravo\"}}],\"bye\":null}]"
    );
}

#[test]
fn team_round_json_renders_bye_team() {
    let rounds = round_robin_teams(&[
        Team::new("a", "Alpha"),
        Team::new("b", "Bravo"),
        Team::new("c", "Charlie"),
    ])
    .unwrap();
    let json = rounds[0].to_json();
    assert!(json.contains("\"bye\":{\"id\":"), "{json}");
}

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
