//! Round-robin fixture scheduling.
//!
//! Given a list of team names, [`round_robin`] produces a full single
//! round-robin schedule: every team plays every other team exactly once,
//! split into rounds of simultaneous fixtures. Odd-sized leagues get a
//! rotating bye, distributed so no team sits out twice before everyone
//! else has sat out once.
//!
//! The core scheduling functions identify teams by a plain `&str`. When a
//! team's id (used for lookups, storage, external systems) needs to
//! differ from what gets shown on a fixture list, use [`Team`] and the
//! `_teams` variants ([`round_robin_teams`], [`double_round_robin_teams`],
//! and their seeded counterparts) instead.

use std::collections::HashMap;
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

/// A team with a stable id kept separate from the name shown on fixture
/// lists. Useful when the id is a database key, a slug, or anything else
/// that shouldn't change even if the team's display name does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Team {
    pub id: String,
    pub name: String,
}

impl Team {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Team { id: id.into(), name: name.into() }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<&str> for Team {
    /// Builds a team whose id and display name are the same string.
    fn from(name: &str) -> Self {
        Team { id: name.to_string(), name: name.to_string() }
    }
}

/// A single match between two teams within a round, identified by [`Team`]
/// rather than a raw string. Produced by [`round_robin_teams`] and its
/// siblings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamFixture {
    pub home: Team,
    pub away: Team,
}

/// One round of a [`Team`]-based schedule. See [`Round`] for the
/// string-based equivalent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamRound {
    pub number: usize,
    pub fixtures: Vec<TeamFixture>,
    pub bye: Option<Team>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleError {
    NotEnoughTeams,
    DuplicateTeam(String),
    SeedMismatch,
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
            ScheduleError::SeedMismatch => {
                write!(f, "seed must contain exactly the same teams as the schedule, each once")
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

fn validate_teams(teams: &[&str]) -> Result<(), ScheduleError> {
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
    Ok(())
}

/// Builds a single round-robin schedule for the given teams.
///
/// Uses the standard "circle method": one team is held fixed and the
/// rest rotate around it one position per round, which guarantees every
/// pair meets exactly once in `n - 1` rounds (or `n` rounds if a bye is
/// needed to make the count even).
///
/// The order of `teams` decides who plays whom in round one (the circle
/// method pairs position `i` against position `n - 1 - i`). To pick that
/// opening matchup deliberately without reshuffling your canonical team
/// list, use [`round_robin_seeded`] instead.
pub fn round_robin(teams: &[&str]) -> Result<Vec<Round>, ScheduleError> {
    validate_teams(teams)?;
    Ok(build_schedule(teams))
}

/// Like [`round_robin`], but schedules the teams in the order given by
/// `seed` rather than the order of `teams` itself.
///
/// `seed` must contain exactly the teams in `teams`, each appearing once,
/// in whatever order the caller wants round one to pair them. This lets a
/// caller keep `teams` in its natural order (alphabetical, by id, however
/// it's stored) while still controlling the opening-round matchups.
pub fn round_robin_seeded(teams: &[&str], seed: &[&str]) -> Result<Vec<Round>, ScheduleError> {
    validate_teams(teams)?;

    let mut teams_sorted = teams.to_vec();
    teams_sorted.sort_unstable();
    let mut seed_sorted = seed.to_vec();
    seed_sorted.sort_unstable();
    if teams_sorted != seed_sorted {
        return Err(ScheduleError::SeedMismatch);
    }

    Ok(build_schedule(seed))
}

fn build_schedule(teams: &[&str]) -> Vec<Round> {
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

    rounds
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
    Ok(mirror_second_leg(round_robin(teams)?))
}

/// Like [`double_round_robin`], but schedules the first leg with
/// [`round_robin_seeded`] so the caller controls the opening-round
/// matchups. See that function for what `seed` must contain.
pub fn double_round_robin_seeded(
    teams: &[&str],
    seed: &[&str],
) -> Result<Vec<Round>, ScheduleError> {
    Ok(mirror_second_leg(round_robin_seeded(teams, seed)?))
}

fn mirror_second_leg(first_leg: Vec<Round>) -> Vec<Round> {
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

    rounds
}

/// Like [`round_robin`], but takes and returns [`Team`] values so a team's
/// id (used for scheduling and lookups) can differ from its display name.
pub fn round_robin_teams(teams: &[Team]) -> Result<Vec<TeamRound>, ScheduleError> {
    let ids: Vec<&str> = teams.iter().map(|t| t.id.as_str()).collect();
    let rounds = round_robin(&ids)?;
    Ok(rounds.into_iter().map(|r| resolve_round(r, teams)).collect())
}

/// Like [`round_robin_seeded`], but takes and returns [`Team`] values. The
/// teams in `seed` are matched to `teams` by id, so `seed` may reuse the
/// same [`Team`] values or hand back different display names for the same
/// ids -- the ids are what decide the pairing.
pub fn round_robin_teams_seeded(
    teams: &[Team],
    seed: &[Team],
) -> Result<Vec<TeamRound>, ScheduleError> {
    let ids: Vec<&str> = teams.iter().map(|t| t.id.as_str()).collect();
    let seed_ids: Vec<&str> = seed.iter().map(|t| t.id.as_str()).collect();
    let rounds = round_robin_seeded(&ids, &seed_ids)?;
    Ok(rounds.into_iter().map(|r| resolve_round(r, teams)).collect())
}

/// Like [`double_round_robin`], but takes and returns [`Team`] values.
pub fn double_round_robin_teams(teams: &[Team]) -> Result<Vec<TeamRound>, ScheduleError> {
    let ids: Vec<&str> = teams.iter().map(|t| t.id.as_str()).collect();
    let rounds = double_round_robin(&ids)?;
    Ok(rounds.into_iter().map(|r| resolve_round(r, teams)).collect())
}

/// Like [`double_round_robin_seeded`], but takes and returns [`Team`]
/// values. See [`round_robin_teams_seeded`] for what `seed` must contain.
pub fn double_round_robin_teams_seeded(
    teams: &[Team],
    seed: &[Team],
) -> Result<Vec<TeamRound>, ScheduleError> {
    let ids: Vec<&str> = teams.iter().map(|t| t.id.as_str()).collect();
    let seed_ids: Vec<&str> = seed.iter().map(|t| t.id.as_str()).collect();
    let rounds = double_round_robin_seeded(&ids, &seed_ids)?;
    Ok(rounds.into_iter().map(|r| resolve_round(r, teams)).collect())
}

// The string-based scheduler already does all the real work; this just
// looks each id back up to the Team it came from so callers get their
// display names back.
fn resolve_round(round: Round, teams: &[Team]) -> TeamRound {
    let lookup: HashMap<&str, &Team> = teams.iter().map(|t| (t.id.as_str(), t)).collect();
    TeamRound {
        number: round.number,
        fixtures: round
            .fixtures
            .into_iter()
            .map(|f| TeamFixture {
                home: lookup[f.home.as_str()].clone(),
                away: lookup[f.away.as_str()].clone(),
            })
            .collect(),
        bye: round.bye.map(|id| lookup[id.as_str()].clone()),
    }
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
