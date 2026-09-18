use fixture_scheduler::{slot_dates, Date};

#[test]
fn rejects_impossible_calendar_dates() {
    assert_eq!(Date::new(2026, 2, 30), None, "february never has 30 days");
    assert_eq!(Date::new(2026, 13, 1), None, "there is no month 13");
    assert_eq!(Date::new(2026, 0, 1), None, "there is no month 0");
    assert_eq!(Date::new(2026, 0, 0), None, "there is no day 0");
    assert!(Date::new(2026, 2, 28).is_some());
}

#[test]
fn accepts_leap_day_only_in_leap_years() {
    assert!(Date::new(2024, 2, 29).is_some(), "2024 is a leap year");
    assert_eq!(Date::new(2025, 2, 29), None, "2025 is not a leap year");
    assert!(Date::new(2000, 2, 29).is_some(), "2000 is divisible by 400, so it is a leap year");
    assert_eq!(Date::new(1900, 2, 29), None, "1900 is divisible by 100 but not 400");
}

#[test]
fn next_day_rolls_over_month_and_year() {
    assert_eq!(Date::new(2026, 1, 31).unwrap().next_day(), Date::new(2026, 2, 1).unwrap());
    assert_eq!(Date::new(2026, 12, 31).unwrap().next_day(), Date::new(2027, 1, 1).unwrap());
    assert_eq!(Date::new(2024, 2, 28).unwrap().next_day(), Date::new(2024, 2, 29).unwrap());
    assert_eq!(Date::new(2025, 2, 28).unwrap().next_day(), Date::new(2025, 3, 1).unwrap());
}

#[test]
fn add_days_matches_repeated_next_day() {
    let start = Date::new(2026, 1, 1).unwrap();
    let mut stepped = start;
    for _ in 0..40 {
        stepped = stepped.next_day();
    }
    assert_eq!(start.add_days(40), stepped);
}

#[test]
fn slots_rounds_at_the_requested_interval_with_no_blackouts() {
    let start = Date::new(2026, 3, 7).unwrap();
    let dates = slot_dates(4, start, 7, &[]);

    assert_eq!(
        dates,
        vec![
            Date::new(2026, 3, 7).unwrap(),
            Date::new(2026, 3, 14).unwrap(),
            Date::new(2026, 3, 21).unwrap(),
            Date::new(2026, 3, 28).unwrap(),
        ]
    );
}

#[test]
fn pushes_a_blacked_out_round_forward_without_shifting_the_rest() {
    let start = Date::new(2026, 3, 7).unwrap();
    // Round two would normally land on 2026-03-14; black that day out.
    let blackout = [Date::new(2026, 3, 14).unwrap()];
    let dates = slot_dates(3, start, 7, &blackout);

    assert_eq!(dates[0], Date::new(2026, 3, 7).unwrap());
    assert_eq!(dates[1], Date::new(2026, 3, 15).unwrap(), "should skip past the blackout day");
    assert_eq!(
        dates[2],
        Date::new(2026, 3, 21).unwrap(),
        "round three keeps its own base date rather than shifting with round two"
    );
}

#[test]
fn skips_over_multiple_consecutive_blackout_days() {
    let start = Date::new(2026, 3, 7).unwrap();
    let blackout = [
        Date::new(2026, 3, 7).unwrap(),
        Date::new(2026, 3, 8).unwrap(),
        Date::new(2026, 3, 9).unwrap(),
    ];
    let dates = slot_dates(1, start, 7, &blackout);

    assert_eq!(dates[0], Date::new(2026, 3, 10).unwrap());
}

#[test]
fn date_displays_as_iso_8601() {
    let date = Date::new(2026, 3, 7).unwrap();
    assert_eq!(date.to_string(), "2026-03-07");
}
