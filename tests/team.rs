use fixture_scheduler::{
    double_round_robin_teams, double_round_robin_teams_seeded, round_robin_teams,
    round_robin_teams_seeded, ScheduleError, Team,
};

#[test]
fn schedules_by_id_but_returns_display_names() {
    let teams = vec![
        Team::new("fal", "Falcons"),
        Team::new("hwk", "Hawks"),
        Team::new("owl", "Owls"),
        Team::new("lrk", "Larks"),
    ];

    let rounds = round_robin_teams(&teams).unwrap();

    assert_eq!(rounds.len(), 3);
    for round in &rounds {
        assert_eq!(round.fixtures.len(), 2);
        for fixture in &round.fixtures {
            // The display name should never equal the id: each team here
            // was constructed with a distinct id and name, so getting the
            // id back on `name` would mean the lookup lost the team.
            assert_ne!(fixture.home.name, fixture.home.id);
            assert_ne!(fixture.away.name, fixture.away.id);
            assert_ne!(fixture.home.id, fixture.away.id);
        }
    }
}

#[test]
fn duplicate_id_is_rejected_even_with_different_names() {
    let teams = vec![Team::new("a", "Alpha"), Team::new("a", "Alpha Two")];

    assert_eq!(round_robin_teams(&teams), Err(ScheduleError::DuplicateTeam("a".to_string())));
}

#[test]
fn odd_sized_league_byes_carry_the_full_team() {
    let teams = vec![Team::new("a", "Alpha"), Team::new("b", "Bravo"), Team::new("c", "Charlie")];

    let rounds = round_robin_teams(&teams).unwrap();

    let mut byes: Vec<&Team> = rounds.iter().filter_map(|r| r.bye.as_ref()).collect();
    byes.sort_by(|a, b| a.id.cmp(&b.id));
    let ids: Vec<&str> = byes.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "b", "c"]);
}

#[test]
fn seeded_schedule_matches_by_id_for_round_one() {
    let teams = vec![
        Team::new("a", "Alpha"),
        Team::new("b", "Bravo"),
        Team::new("c", "Charlie"),
        Team::new("d", "Delta"),
    ];
    let seed = vec![
        Team::new("d", "Delta"),
        Team::new("c", "Charlie"),
        Team::new("b", "Bravo"),
        Team::new("a", "Alpha"),
    ];

    let default_first = round_robin_teams(&teams).unwrap()[0].clone();
    let seeded_first = round_robin_teams_seeded(&teams, &seed).unwrap()[0].clone();

    assert_ne!(default_first, seeded_first);
}

#[test]
fn double_round_robin_teams_mirrors_home_and_away() {
    let teams = vec![Team::new("a", "Alpha"), Team::new("b", "Bravo")];

    let double = double_round_robin_teams(&teams).unwrap();

    assert_eq!(double.len(), 2);
    let first = &double[0].fixtures[0];
    let second = &double[1].fixtures[0];
    assert_eq!(first.home.id, second.away.id);
    assert_eq!(first.away.id, second.home.id);
}

#[test]
fn double_round_robin_teams_seeded_propagates_seed_mismatch() {
    let teams = vec![Team::new("a", "Alpha"), Team::new("b", "Bravo")];
    let bad_seed = vec![Team::new("a", "Alpha"), Team::new("z", "Zulu")];

    assert_eq!(
        double_round_robin_teams_seeded(&teams, &bad_seed),
        Err(ScheduleError::SeedMismatch)
    );
}

#[test]
fn team_from_str_uses_the_same_string_as_id_and_name() {
    let team: Team = "Falcons".into();
    assert_eq!(team.id, "Falcons");
    assert_eq!(team.name, "Falcons");
    assert_eq!(team.to_string(), "Falcons");
}
