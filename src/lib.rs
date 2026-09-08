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

/// Builds a double round-robin schedule: every pairing from
/// [`round_robin`], followed by a second leg with home and away swapped.
///
/// The second leg is a mirror of the first rather than an independently
/// rotated schedule, so a team that hosted another in round 3 of the
/// first leg will visit them in the equivalent round of the second leg.
/// Byes carry over unchanged, since the team sitting out a round in the
/// first leg sits out the same round again in the second.
pub fn double_round_robin(teams: &[&str]) -> Result<Vec<Round>, ScheduleError> {
    let first_leg = round_robin(teams)?;
    let legs_offset = first_leg.len();

    let mut rounds = first_leg.clone();
    for round in &first_leg {
        let fixtures = round
            .fixtures
            .iter()
            .map(|f| Fixture { home: f.away.clone(), away: f.home.clone() })
            .collect();
        rounds.push(Round {
            number: round.number + legs_offset,
            fixtures,
            bye: round.bye.clone(),
        });
    }

    Ok(rounds)
}

// Hand-rolled JSON output, no serde. The format is fixed and small enough
// (fixtures, a round number, an optional bye) that pulling in a dependency
// just to write a handful of string literals isn't worth it.
#[cfg(feature = "json")]
mod json {
    use super::{Fixture, Round};

    fn escape(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        out.push('"');
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out.push('"');
        out
    }

    impl Fixture {
        /// Renders this fixture as a JSON object: `{"home": "...", "away": "..."}`.
        pub fn to_json(&self) -> String {
            format!("{{\"home\":{},\"away\":{}}}", escape(&self.home), escape(&self.away))
        }
    }

    impl Round {
        /// Renders this round as a JSON object with `number`, `fixtures`, and `bye` fields.
        pub fn to_json(&self) -> String {
            let fixtures: Vec<String> = self.fixtures.iter().map(Fixture::to_json).collect();
            let bye = match &self.bye {
                Some(name) => escape(name),
                None => "null".to_string(),
            };
            format!(
                "{{\"number\":{},\"fixtures\":[{}],\"bye\":{}}}",
                self.number,
                fixtures.join(","),
                bye
            )
        }
    }

    /// Renders a full schedule, as returned by [`super::round_robin`] or
    /// [`super::double_round_robin`], as a JSON array of rounds.
    pub fn schedule_to_json(rounds: &[Round]) -> String {
        let rounds: Vec<String> = rounds.iter().map(Round::to_json).collect();
        format!("[{}]", rounds.join(","))
    }
}

#[cfg(feature = "json")]
pub use json::schedule_to_json;
