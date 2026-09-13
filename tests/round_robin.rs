use fixture_scheduler::{
    double_round_robin, double_round_robin_seeded, round_robin, round_robin_seeded,
    ScheduleError,
};
use std::collections::HashSet;

struct ErrorCase {
    name: &'static str,
    teams: &'static [&'static str],
    expected: ScheduleError,
}

#[test]
fn rejects_invalid_input() {
    let cases = [
        ErrorCase {
            name: "no teams",
            teams: &[],
            expected: ScheduleError::NotEnoughTeams,
        },
        ErrorCase {
            name: "one team",
            teams: &["Falcons"],
            expected: ScheduleError::NotEnoughTeams,
        },
        ErrorCase {
            name: "duplicate team",
            teams: &["Falcons", "Falcons"],
            expected: ScheduleError::DuplicateTeam("Falcons".to_string()),
        },
        ErrorCase {
            name: "duplicate team later in the list",
            teams: &["A", "B", "C", "B"],
            expected: ScheduleError::DuplicateTeam("B".to_string()),
        },
    ];

    for case in cases {
        let got = round_robin(case.teams);
        assert_eq!(
            got,
            Err(case.expected.clone()),
            "case '{}' produced {:?}",
            case.name,
            got
        );
    }
}

struct ShapeCase {
    name: &'static str,
    teams: &'static [&'static str],
    expected_rounds: usize,
    expected_fixtures_per_round: usize,
    expects_bye: bool,
}

#[test]
fn produces_the_right_shape() {
    const SIX: &[&str] = &["A", "B", "C", "D", "E", "F"];
    const SEVEN: &[&str] = &["A", "B", "C", "D", "E", "F", "G"];

    let cases = [
        ShapeCase {
            name: "two teams",
            teams: &["A", "B"],
            expected_rounds: 1,
            expected_fixtures_per_round: 1,
            expects_bye: false,
        },
        ShapeCase {
            name: "three teams needs a rotating bye",
            teams: &["A", "B", "C"],
            expected_rounds: 3,
            expected_fixtures_per_round: 1,
            expects_bye: true,
        },
        ShapeCase {
            name: "four teams, no bye",
            teams: &["A", "B", "C", "D"],
            expected_rounds: 3,
            expected_fixtures_per_round: 2,
            expects_bye: false,
        },
        ShapeCase {
            name: "five teams needs a rotating bye",
            teams: &["A", "B", "C", "D", "E"],
            expected_rounds: 5,
            expected_fixtures_per_round: 2,
            expects_bye: true,
        },
        ShapeCase {
            name: "six teams, no bye",
            teams: SIX,
            expected_rounds: 5,
            expected_fixtures_per_round: 3,
            expects_bye: false,
        },
        ShapeCase {
            name: "seven teams needs a rotating bye",
            teams: SEVEN,
            expected_rounds: 7,
            expected_fixtures_per_round: 3,
            expects_bye: true,
        },
    ];

    for case in cases {
        let rounds = round_robin(case.teams).unwrap_or_else(|e| {
            panic!("case '{}' failed to schedule: {e}", case.name)
        });

        assert_eq!(
            rounds.len(),
            case.expected_rounds,
            "case '{}': wrong round count",
            case.name
        );

        for round in &rounds {
            assert_eq!(
                round.fixtures.len(),
                case.expected_fixtures_per_round,
                "case '{}' round {}: wrong fixture count",
                case.name,
                round.number
            );
            assert_eq!(
                round.bye.is_some(),
                case.expects_bye,
                "case '{}' round {}: unexpected bye state",
                case.name,
                round.number
            );
        }
    }
}

// The part that's easy to get subtly wrong: every pair of teams must meet
// exactly once, no team ever plays itself, and (for odd-sized leagues)
// every team sits out exactly once before any team sits out twice.
#[test]
fn covers_every_pair_exactly_once() {
    let cases: &[&[&str]] = &[
        &["A", "B"],
        &["A", "B", "C"],
        &["A", "B", "C", "D"],
        &["A", "B", "C", "D", "E"],
        &["A", "B", "C", "D", "E", "F", "G"],
    ];

    for teams in cases {
        let rounds = round_robin(teams).unwrap();

        let mut seen_pairs: HashSet<(String, String)> = HashSet::new();
        let mut bye_counts: std::collections::HashMap<String, usize> =
            teams.iter().map(|t| (t.to_string(), 0)).collect();

        for round in &rounds {
            let mut playing_this_round: HashSet<String> = HashSet::new();

            for fixture in &round.fixtures {
                assert_ne!(fixture.home, fixture.away, "a team was scheduled against itself");

                let pair = if fixture.home < fixture.away {
                    (fixture.home.clone(), fixture.away.clone())
                } else {
                    (fixture.away.clone(), fixture.home.clone())
                };
                assert!(
                    seen_pairs.insert(pair.clone()),
                    "pair {pair:?} scheduled more than once for {teams:?}"
                );

                assert!(playing_this_round.insert(fixture.home.clone()));
                assert!(playing_this_round.insert(fixture.away.clone()));
            }

            if let Some(bye) = &round.bye {
                *bye_counts.get_mut(bye).unwrap() += 1;
                assert!(!playing_this_round.contains(bye), "bye team also has a fixture");
            }
        }

        let expected_pairs = teams.len() * (teams.len() - 1) / 2;
        assert_eq!(seen_pairs.len(), expected_pairs, "missing pairings for {teams:?}");

        if teams.len() % 2 != 0 {
            for (team, count) in &bye_counts {
                assert_eq!(*count, 1, "team {team} did not get exactly one bye in {teams:?}");
            }
        }
    }
}

#[test]
fn seeded_schedule_uses_seed_order_for_round_one() {
    let teams = ["A", "B", "C", "D"];

    let default_first_round = round_robin(&teams).unwrap()[0].clone();
    let seeded_first_round =
        round_robin_seeded(&teams, &["D", "C", "B", "A"]).unwrap()[0].clone();

    assert_ne!(
        default_first_round, seeded_first_round,
        "reversing the seed should change who plays whom in round one"
    );

    // The seeded schedule should still be a full, valid round-robin: same
    // round count, same fixture shape, just a different starting pairing.
    let unseeded = round_robin(&teams).unwrap();
    let seeded = round_robin_seeded(&teams, &["D", "C", "B", "A"]).unwrap();
    assert_eq!(seeded.len(), unseeded.len());
    for round in &seeded {
        assert_eq!(round.fixtures.len(), 2);
        assert!(round.bye.is_none());
    }
}

#[test]
fn seeded_schedule_rejects_mismatched_seed() {
    let teams = ["A", "B", "C"];

    assert_eq!(
        round_robin_seeded(&teams, &["A", "B", "D"]),
        Err(ScheduleError::SeedMismatch),
        "seed with a team not in the roster should be rejected"
    );
    assert_eq!(
        round_robin_seeded(&teams, &["A", "B"]),
        Err(ScheduleError::SeedMismatch),
        "seed missing a team should be rejected"
    );
    assert_eq!(
        round_robin_seeded(&teams, &["A", "A", "C"]),
        Err(ScheduleError::SeedMismatch),
        "seed with a duplicate standing in for a missing team should be rejected"
    );
}

#[test]
fn seeded_schedule_still_checks_the_roster_first() {
    assert_eq!(
        round_robin_seeded(&["Falcons"], &["Falcons"]),
        Err(ScheduleError::NotEnoughTeams)
    );
    assert_eq!(
        round_robin_seeded(&["A", "A"], &["A", "A"]),
        Err(ScheduleError::DuplicateTeam("A".to_string()))
    );
}

#[test]
fn double_round_robin_seeded_mirrors_the_seeded_first_leg() {
    let teams = ["A", "B", "C", "D"];
    let seed = ["D", "C", "B", "A"];

    let single = round_robin_seeded(&teams, &seed).unwrap();
    let double = double_round_robin_seeded(&teams, &seed).unwrap();

    assert_eq!(double.len(), single.len() * 2);
    let (first_leg, second_leg) = double.split_at(single.len());
    assert_eq!(first_leg, single.as_slice());
    for (first_round, second_round) in first_leg.iter().zip(second_leg) {
        for (first_fixture, second_fixture) in
            first_round.fixtures.iter().zip(&second_round.fixtures)
        {
            assert_eq!(second_fixture.home, first_fixture.away);
            assert_eq!(second_fixture.away, first_fixture.home);
        }
    }
}

#[test]
fn double_round_robin_propagates_errors() {
    assert_eq!(double_round_robin(&["Falcons"]), Err(ScheduleError::NotEnoughTeams));
    assert_eq!(
        double_round_robin(&["A", "B", "A"]),
        Err(ScheduleError::DuplicateTeam("A".to_string()))
    );
}

#[test]
fn double_round_robin_mirrors_the_first_leg() {
    let cases: &[&[&str]] = &[&["A", "B", "C", "D"], &["A", "B", "C", "D", "E"]];

    for teams in cases {
        let single = round_robin(teams).unwrap();
        let double = double_round_robin(teams).unwrap();

        assert_eq!(double.len(), single.len() * 2, "wrong round count for {teams:?}");

        let (first_leg, second_leg) = double.split_at(single.len());

        assert_eq!(first_leg, single.as_slice(), "first leg should match a single round-robin");

        for (first_round, second_round) in first_leg.iter().zip(second_leg) {
            assert_eq!(
                second_round.number,
                first_round.number + single.len(),
                "second leg round numbers should continue from the first"
            );
            assert_eq!(
                second_round.bye, first_round.bye,
                "the same team should sit out the mirrored round"
            );
            assert_eq!(
                second_round.fixtures.len(),
                first_round.fixtures.len(),
                "mirrored round should have the same number of fixtures"
            );
            for (first_fixture, second_fixture) in
                first_round.fixtures.iter().zip(&second_round.fixtures)
            {
                assert_eq!(second_fixture.home, first_fixture.away, "home/away should be swapped");
                assert_eq!(second_fixture.away, first_fixture.home, "home/away should be swapped");
            }
        }
    }
}
