use crate::model::*;
use chrono::NaiveDate;

pub const DUE_WINDOW_DAYS: i64 = 3;
pub const STALE_DAYS: i64 = 14;

fn norm(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(|c| c.to_lowercase()).collect()
}

fn plural(n: i64, one: &str, many: &str) -> String {
    if n == 1 { format!("{n} {one}") } else { format!("{n} {many}") }
}

fn due_reason(days_left: i64) -> String {
    match days_left {
        d if d < 0 => format!("overdue {}", plural(-d, "day", "days")),
        0 => "due today".to_string(),
        1 => "due tomorrow".to_string(),
        d => format!("due in {d} days"),
    }
}

pub fn build_day(
    today: NaiveDate,
    deadlines: &[Deadline],
    repos: &[RepoInfo],
    vault_notes: &[String],
    warnings: Vec<String>,
) -> DayView {
    let mut dated: Vec<(i64, &Deadline)> =
        deadlines.iter().map(|d| ((d.due - today).num_days(), d)).collect();
    dated.sort_by_key(|(days, _)| *days);

    let mut items: Vec<Item> = dated
        .iter()
        .filter(|(days, _)| *days <= DUE_WINDOW_DAYS)
        .map(|(days, d)| Item { kind: Kind::Due, title: d.title.clone(), reason: due_reason(*days) })
        .collect();

    let mut unpushed: Vec<&RepoInfo> = repos.iter().filter(|r| r.unpushed > 0).collect();
    unpushed.sort_by(|a, b| b.unpushed.cmp(&a.unpushed).then(a.name.cmp(&b.name)));
    items.extend(unpushed.iter().map(|r| Item {
        kind: Kind::Unpushed,
        title: r.name.clone(),
        reason: format!("{} unpushed", plural(r.unpushed as i64, "commit", "commits")),
    }));

    let mut stale: Vec<&RepoInfo> = repos
        .iter()
        .filter(|r| r.unpushed == 0 && r.last_commit_days >= STALE_DAYS)
        .filter(|r| vault_notes.iter().any(|n| norm(n) == norm(&r.name)))
        .collect();
    stale.sort_by(|a, b| b.last_commit_days.cmp(&a.last_commit_days).then(a.name.cmp(&b.name)));
    items.extend(stale.iter().map(|r| Item {
        kind: Kind::Stale,
        title: r.name.clone(),
        reason: format!("stale {} days", r.last_commit_days),
    }));

    let schedule = dated
        .iter()
        .map(|(days, d)| ScheduleEntry {
            title: d.title.clone(),
            due: d.due.format("%Y-%m-%d").to_string(),
            days_left: *days,
        })
        .collect();

    let stats = Stats {
        open: items.len(),
        due_soon: items.iter().filter(|i| i.kind == Kind::Due).count(),
        unpushed: items.iter().filter(|i| i.kind == Kind::Unpushed).count(),
    };

    DayView { today: today.format("%Y-%m-%d").to_string(), items, schedule, stats, warnings }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }
    fn dl(title: &str, due: &str) -> Deadline {
        Deadline { title: title.into(), due: d(due), project: None }
    }
    fn repo(name: &str, unpushed: u32, days: i64) -> RepoInfo {
        RepoInfo { name: name.into(), unpushed, last_commit_days: days }
    }

    #[test]
    fn due_within_window_ranked_soonest_first_with_reasons() {
        let day = build_day(
            d("2026-10-08"),
            &[dl("Lab report", "2026-10-10"), dl("Quiz", "2026-10-09"), dl("Far", "2026-10-20")],
            &[], &[], vec![],
        );
        let titles: Vec<_> = day.items.iter().map(|i| i.title.as_str()).collect();
        assert_eq!(titles, vec!["Quiz", "Lab report"]);
        assert_eq!(day.items[0].reason, "due tomorrow");
        assert_eq!(day.items[1].reason, "due in 2 days");
    }

    #[test]
    fn overdue_and_today_are_kept_and_first() {
        let day = build_day(
            d("2026-10-08"),
            &[dl("Today", "2026-10-08"), dl("Late", "2026-10-06")],
            &[], &[], vec![],
        );
        assert_eq!(day.items[0].title, "Late");
        assert_eq!(day.items[0].reason, "overdue 2 days");
        assert_eq!(day.items[1].reason, "due today");
    }

    #[test]
    fn unpushed_repos_come_after_due_and_pluralise() {
        let day = build_day(
            d("2026-10-08"),
            &[dl("Quiz", "2026-10-09")],
            &[repo("Winnow", 3, 1), repo("Lot", 1, 1), repo("Clean", 0, 1)],
            &[], vec![],
        );
        let reasons: Vec<_> = day.items.iter().map(|i| i.reason.as_str()).collect();
        assert_eq!(reasons, vec!["due tomorrow", "3 commits unpushed", "1 commit unpushed"]);
    }

    #[test]
    fn stale_needs_vault_note_and_age_and_is_last() {
        let day = build_day(
            d("2026-10-08"),
            &[],
            &[repo("Deadwax", 0, 21), repo("NoNote", 0, 40), repo("Fresh", 0, 2)],
            &["deadwax".to_string(), "fresh".to_string()],
            vec![],
        );
        assert_eq!(day.items.len(), 1);
        assert_eq!(day.items[0].kind, Kind::Stale);
        assert_eq!(day.items[0].reason, "stale 21 days");
    }

    #[test]
    fn repo_already_listed_unpushed_is_not_also_stale() {
        let day = build_day(
            d("2026-10-08"), &[],
            &[repo("Deadwax", 2, 30)], &["deadwax".to_string()], vec![],
        );
        assert_eq!(day.items.len(), 1);
        assert_eq!(day.items[0].kind, Kind::Unpushed);
    }

    #[test]
    fn schedule_lists_all_deadlines_in_date_order_and_stats_count() {
        let day = build_day(
            d("2026-10-08"),
            &[dl("B", "2026-10-30"), dl("A", "2026-10-09")],
            &[repo("X", 2, 1)], &[], vec!["note".into()],
        );
        assert_eq!(day.schedule[0].title, "A");
        assert_eq!(day.schedule[0].days_left, 1);
        assert_eq!(day.schedule[1].title, "B");
        assert_eq!(day.stats, Stats { open: 2, due_soon: 1, unpushed: 1 });
        assert_eq!(day.warnings, vec!["note".to_string()]);
        assert_eq!(day.today, "2026-10-08");
    }

    #[test]
    fn nothing_to_do_gives_empty_items() {
        let day = build_day(d("2026-10-08"), &[], &[], &[], vec![]);
        assert!(day.items.is_empty());
        assert_eq!(day.stats.open, 0);
    }

    #[test]
    fn stale_matches_kebab_case_note_to_camel_case_repo() {
        let day = build_day(
            d("2026-10-08"), &[],
            &[repo("AnthropicSkillJar", 0, 30), repo("VeridockAI", 0, 30)],
            &["anthropic-skilljar".to_string(), "veridock-ai".to_string()], vec![],
        );
        assert_eq!(day.items.len(), 2);
    }
}
