use chrono::{Datelike, Duration, Local, NaiveDate};
use std::collections::HashMap;
use crate::gui::db::TimeEntry; 

pub type ProjectTimeEntries = HashMap<String, Vec<TimeEntry>>;

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

pub fn week_start(date: NaiveDate) -> NaiveDate {
    date - Duration::days(date.weekday().num_days_from_monday() as i64)
}

pub fn total_for_day(entries: &ProjectTimeEntries, date: NaiveDate) -> u64 {
    entries
        .values()
        .flat_map(|project_entries| project_entries.iter())
        .filter(|entry| entry.date.date_naive() == date)
        .map(|entry| u64::from(entry.seconds))
        .sum()
}

pub fn total_for_week(entries: &ProjectTimeEntries, date: NaiveDate) -> u64 {
    let start = week_start(date);
    let end = start + Duration::days(7);

    entries
        .values()
        .flat_map(|project_entries| project_entries.iter())
        .filter(|entry| {
            let entry_date = entry.date.date_naive();
            entry_date >= start && entry_date < end
        })
        .map(|entry| u64::from(entry.seconds))
        .sum()
}

pub fn focus_streak(entries: &ProjectTimeEntries, from: NaiveDate) -> u32 {
    let mut streak = 0;
    let mut date = from;

    loop {
        if total_for_day(entries, date) == 0 {
            break;
        }

        streak += 1;
        date -= Duration::days(1);
    }

    streak
}

pub fn project_totals_for_day(
    entries: &ProjectTimeEntries,
    date: NaiveDate,
) -> Vec<(String, u64)> {
    let mut totals: Vec<_> = entries
        .iter()
        .map(|(project, project_entries)| {
            let seconds: u64 = project_entries
                .iter()
                .filter(|entry| entry.project == *project && entry.date.date_naive() == date)
                .map(|entry| u64::from(entry.seconds))
                .sum();
            (project.clone(), seconds)
        })
        .filter(|(_, seconds)| *seconds > 0)
        .collect();

    totals.sort_by(|left, right| right.1.cmp(&left.1));
    totals
}

pub fn project_totals_for_month(
    entries: &ProjectTimeEntries,
    date: NaiveDate,
) -> Vec<(String, u64)> {
    project_totals_where(entries, |entry_date| {
        entry_date.year() == date.year() && entry_date.month() == date.month()
    })
}

pub fn project_totals_for_week(
    entries: &ProjectTimeEntries,
    date: NaiveDate,
) -> Vec<(String, u64)> {
    let start = week_start(date);
    project_totals_where(entries, |entry_date| {
        entry_date >= start && entry_date < start + Duration::days(7)
    })
}

pub fn project_totals_for_year(
    entries: &ProjectTimeEntries,
    date: NaiveDate,
) -> Vec<(String, u64)> {
    project_totals_where(entries, |entry_date| entry_date.year() == date.year())
}

fn project_totals_where(
    entries: &ProjectTimeEntries,
    matches_date: impl Fn(NaiveDate) -> bool,
) -> Vec<(String, u64)> {
    let mut totals: Vec<_> = entries
        .iter()
        .map(|(project, project_entries)| {
            let seconds: u64 = project_entries
                .iter()
                .filter(|entry| {
                    entry.project == *project && matches_date(entry.date.date_naive())
                })
                .map(|entry| u64::from(entry.seconds))
                .sum();
            (project.clone(), seconds)
        })
        .filter(|(_, seconds)| *seconds > 0)
        .collect();

    totals.sort_by(|left, right| right.1.cmp(&left.1));
    totals
}

pub fn daily_totals_for_week(
    entries: &ProjectTimeEntries,
    date: NaiveDate,
) -> Vec<(NaiveDate, u64)> {
    let start = week_start(date);

    (0..7)
        .map(|offset| {
            let day = start + Duration::days(offset);
            (day, total_for_day(entries, day))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(project: &str, date: NaiveDate, seconds: u32) -> TimeEntry {
        TimeEntry {
            project: project.to_string(),
            date: Local::now()
                .with_year(date.year())
                .unwrap()
                .with_month(date.month())
                .unwrap()
                .with_day(date.day())
                .unwrap(),
            seconds,
        }
    }

    #[test]
    fn aggregates_today_and_week() {
        let today = today();
        let mut entries = ProjectTimeEntries::new();
        entries.insert(
            "Focus".into(),
            vec![entry("Focus", today, 120), entry("Focus", today, 30)],
        );
        entries.insert("Other".into(), vec![entry("Other", today, 50)]);

        assert_eq!(total_for_day(&entries, today), 200);
        assert_eq!(total_for_week(&entries, today), 200);
        assert_eq!(project_totals_for_day(&entries, today)[0], ("Focus".into(), 150));
    }

    #[test]
    fn streak_stops_at_first_empty_day() {
        let today = today();
        let yesterday = today - Duration::days(1);
        let mut entries = ProjectTimeEntries::new();
        entries.insert("Focus".into(), vec![entry("Focus", today, 60), entry("Focus", yesterday, 60)]);

        assert_eq!(focus_streak(&entries, today), 2);
        assert_eq!(focus_streak(&entries, today - Duration::days(2)), 0);
    }

    #[test]
    fn week_starts_on_monday() {
        let sunday = NaiveDate::from_ymd_opt(2026, 9, 6).unwrap();
        assert_eq!(week_start(sunday), NaiveDate::from_ymd_opt(2026, 8, 31).unwrap());
    }

    #[test]
    fn daily_totals_cover_monday_to_sunday() {
        let sunday = NaiveDate::from_ymd_opt(2026, 9, 6).unwrap();
        let mut entries = ProjectTimeEntries::new();
        entries.insert("Focus".into(), vec![entry(
            "Focus",
            NaiveDate::from_ymd_opt(2026, 9, 3).unwrap(),
            90,
        )]);

        let totals = daily_totals_for_week(&entries, sunday);
        assert_eq!(totals.len(), 7);
        assert_eq!(totals[0].0, NaiveDate::from_ymd_opt(2026, 8, 31).unwrap());
        assert_eq!(totals[3].1, 90);
        assert_eq!(totals[6].0, sunday);
    }

    #[test]
    fn month_and_year_totals_use_calendar_boundaries() {
        let current = NaiveDate::from_ymd_opt(2026, 9, 6).unwrap();
        let mut entries = ProjectTimeEntries::new();
        entries.insert(
            "Focus".into(),
            vec![
                entry("Focus", NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(), 100),
                entry("Focus", NaiveDate::from_ymd_opt(2026, 8, 31).unwrap(), 200),
                entry("Focus", NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), 300),
            ],
        );

        assert_eq!(project_totals_for_month(&entries, current), vec![("Focus".into(), 100)]);
        assert_eq!(project_totals_for_year(&entries, current), vec![("Focus".into(), 600)]);
    }
}