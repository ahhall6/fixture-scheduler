# fixture-scheduler

A small Rust library for turning a list of team names into a single
round-robin fixture list: every team plays every other team exactly once,
grouped into rounds of simultaneous matches.

This comes up any time you're running a small league or tournament and
need to hand out a schedule - a five-a-side league, a chess club, an
office ping-pong ladder. The annoying part is never the happy path (even
number of teams, everyone plays every round); it's the edge cases: odd
numbers of teams needing a bye, making sure the bye rotates fairly
instead of always landing on the same team, and not silently producing a
broken schedule when someone hands you a duplicate team name.

## Usage

Add it as a path or git dependency (this crate is not published to
crates.io):

```toml
[dependencies]
fixture-scheduler = { path = "../fixture-scheduler" }
```

```rust
use fixture_scheduler::round_robin;

fn main() {
    let teams = ["Ospreys", "Kestrels", "Harriers", "Falcons", "Buzzards"];
    let rounds = round_robin(&teams).expect("valid schedule");

    for round in &rounds {
        println!("Round {}", round.number);
        for fixture in &round.fixtures {
            println!("  {} vs {}", fixture.home, fixture.away);
        }
        if let Some(bye) = &round.bye {
            println!("  {bye} has a bye");
        }
    }
}
```

With five teams (an odd number) this produces five rounds, two fixtures
per round, and one bye per round that rotates so each team sits out
exactly once over the course of the schedule.

If you need a home-and-away season instead of a single leg, use
[`double_round_robin`] - it runs the same schedule twice, with home and
away swapped on the second leg, and continues the round numbering from
where the first leg left off.

```rust
use fixture_scheduler::double_round_robin;

let teams = ["Ospreys", "Kestrels", "Harriers", "Falcons"];
let rounds = double_round_robin(&teams).expect("valid schedule");
assert_eq!(rounds.len(), 6); // 3 rounds per leg, two legs
```

By default, round one's pairings come from the order you pass `teams` in.
If you'd rather keep your team list in some other order (alphabetical, by
id) and still control who opens the season against whom, use
[`round_robin_seeded`] (or [`double_round_robin_seeded`]) and pass the
opening order separately:

```rust
use fixture_scheduler::round_robin_seeded;

let teams = ["Buzzards", "Falcons", "Harriers", "Kestrels", "Ospreys"];
let opening_order = ["Ospreys", "Kestrels", "Harriers", "Falcons", "Buzzards"];
let rounds = round_robin_seeded(&teams, &opening_order).expect("valid schedule");
```

`seed` must contain exactly the teams in `teams`, each once, or you'll get
`ScheduleError::SeedMismatch`.

## Optional features

- `json` - adds `schedule_to_json`, plus `to_json` methods on `Fixture` and
  `Round`, for rendering a schedule as JSON without pulling in serde.

```toml
[dependencies]
fixture-scheduler = { path = "../fixture-scheduler", features = ["json"] }
```

## What it does not do

- No fixture *dates* or venues - this only decides who plays whom, and
  in what round. Slotting rounds onto a calendar is a separate concern.
- No standings or results tracking. This library only produces the
  schedule.

## License

MIT, see [LICENSE](LICENSE).
