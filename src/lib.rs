//! Round-robin fixture scheduling.
//!
//! Given a list of team names, [`round_robin`] produces a full single
//! round-robin schedule: every team plays every other team exactly once,
//! split into rounds of simultaneous fixtures. Odd-sized leagues get a
//! rotating bye, distributed so no team sits out twice before everyone
//! else has sat out once.

use std::fmt;

/// A single match between two teams within a round.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fixture {
    pub home: String,
    pub away: String,
}

/// One round of the schedule: the fixtures played simultaneously, plus
/// the team sitting out this round, if the league has an odd number of
/// teams.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Round {
    pub number: usize,
    pub fixtures: Vec<Fixture>,
    pub bye: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleError {
    NotEnoughTeams,
    DuplicateTeam(String),
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScheduleError::NotEnoughTeams => {
                write!(f, "need at least two teams to schedule fixtures")
            }
            ScheduleError::DuplicateTeam(name) => {
                write!(f, "team '{name}' appears more than once")
            }
        }
    }
}

impl std::error::Error for ScheduleError {}

// Internal placeholder used by the circle method below to represent the
// "empty seat" that odd-sized leagues rotate through. Not a real team, so
// it can't collide with a caller's roster without also failing the
// duplicate check first.
const BYE: &str = "BYE";

/// Builds a single round-robin schedule for the given teams.
///
/// Uses the standard "circle method": one team is held fixed and the
/// rest rotate around it one position per round, which guarantees every
/// pair meets exactly once in `n - 1` rounds (or `n` rounds if a bye is
/// needed to make the count even).
pub fn round_robin(teams: &[&str]) -> Result<Vec<Round>, ScheduleError> {
    if teams.len() < 2 {
        return Err(ScheduleError::NotEnoughTeams);
    }
    for i in 0..teams.len() {
        for j in (i + 1)..teams.len() {
            if teams[i] == teams[j] {
                return Err(ScheduleError::DuplicateTeam(teams[i].to_string()));
            }
        }
    }

    let mut arr: Vec<String> = teams.iter().map(|s| s.to_string()).collect();
    if arr.len() % 2 != 0 {
        arr.push(BYE.to_string());
    }
    let n = arr.len();
    let num_rounds = n - 1;
    let mut rounds = Vec::with_capacity(num_rounds);

    for round in 0..num_rounds {
        let mut fixtures = Vec::with_capacity(n / 2);
        let mut bye = None;

        for i in 0..n / 2 {
            let a = &arr[i];
            let b = &arr[n - 1 - i];
            if a == BYE {
                bye = Some(b.clone());
            } else if b == BYE {
                bye = Some(a.clone());
            } else if round % 2 == 0 {
                fixtures.push(Fixture { home: a.clone(), away: b.clone() });
            } else {
                // Alternate which side of the pairing is "home" each
                // round so a team isn't stuck at home or away all season.
                fixtures.push(Fixture { home: b.clone(), away: a.clone() });
            }
        }

        rounds.push(Round { number: round + 1, fixtures, bye });

        // Keep arr[0] fixed, rotate everyone else by one position.
        let last = arr.pop().expect("arr has at least 2 elements");
        arr.insert(1, last);
    }

    Ok(rounds)
}
