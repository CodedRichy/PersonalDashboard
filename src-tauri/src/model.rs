use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Due,
    Unpushed,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Item {
    pub kind: Kind,
    pub title: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Deadline {
    pub title: String,
    pub due: NaiveDate,
    pub project: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoInfo {
    pub name: String,
    pub unpushed: u32,
    pub last_commit_days: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScheduleEntry {
    pub title: String,
    pub due: String,
    pub days_left: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Stats {
    pub open: usize,
    pub due_soon: usize,
    pub unpushed: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DayView {
    pub today: String,
    pub items: Vec<Item>,
    pub schedule: Vec<ScheduleEntry>,
    pub stats: Stats,
    pub warnings: Vec<String>,
}
