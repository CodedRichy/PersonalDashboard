# Dashboard v0 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A Tauri desktop app that shows, each morning, a ranked list of what to do today and why, from Rishi's git repos, vault project notes and a hand-edited `deadlines.yaml`.

**Architecture:** Rust owns all data: it scans repos with the `git` CLI, lists vault project note names, parses `deadlines.yaml`, and ranks with pure rules. One Tauri command `get_day` returns a `DayView` JSON. SolidJS renders it. No server, no network.

**Tech Stack:** Tauri 2, SolidJS + TypeScript + Vite, Vitest + @solidjs/testing-library + jsdom, Rust (serde, serde_yaml, chrono, tempfile for tests), tauri-plugin-autostart, @fontsource-variable/plus-jakarta-sans.

**Spec:** `docs/superpowers/specs/2026-10-08-dashboard-v0-design.md`

## Global Constraints
- Platform Windows 11; all file I/O UTF-8; ASCII only in CLI/log output.
- Repos root `~/Documents/GitHub`; vault notes `~/Documents/Vault/_brain/projects`; deadlines at `<app_data_dir>/deadlines.yaml`.
- Thresholds are constants: `DUE_WINDOW_DAYS = 3`, `STALE_DAYS = 14`.
- Ranking order: Due, then Unpushed, then Stale. No LLM. Every item carries a reason string.
- Font Plus Jakarta Sans (never Inter/Roboto/Arial). OKLCH tokens. `clamp()` type. Light theme only.
- No network access, no telemetry. Tauri CSP stays strict (no `connect-src` needed; data comes through `invoke`).
- Public repo: no personal data, real deadlines or vault content committed.
- Spec deviation: the spec's "dirty tree" input is dropped because no ranking rule uses it (YAGNI).

## Review Focus
1. Overdue deadline (due date before today): shows as "overdue N days", ranked first, not dropped.
2. `deadlines.yaml` missing: created with a commented sample, app shows an empty list, no crash.
3. `deadlines.yaml` malformed (bad YAML, or one entry with a bad date): other entries still load; a visible warning names the problem; screen is never blank.
4. Repo with no upstream branch, no commits, or a non-repo folder in the root: skipped or treated as 0 unpushed, never an error.
5. Nothing to do (zero items): a friendly "all clear" state, not an empty void.

---

### Task 1: Scaffold the Tauri + Solid app

**Files:**
- Create: whole scaffold in repo root (`package.json`, `src/`, `src-tauri/`, `vite.config.ts`, `index.html`)
- Modify: `vite.config.ts` (vitest), `.gitignore`

**Interfaces:**
- Produces: `npm run tauri dev`, `npm test` (Vitest), `cargo test` in `src-tauri`.

- [ ] **Step 1: Scaffold in a scratch dir (repo already has docs/)**

```bash
cd /c/Users/rishi/AppData/Local/Temp && rm -rf dash-scaffold
npm create tauri-app@latest dash-scaffold -- -m npm -t solid-ts --identifier com.codedrichy.personaldashboard -y
cp -r dash-scaffold/. /c/Users/rishi/Documents/GitHub/Dashboard/
cd /c/Users/rishi/Documents/GitHub/Dashboard && npm install
```
Expected: `package.json` and `src-tauri/` now exist in the repo; the scaffold's own `.git` (if any) is not copied over (delete `dash-scaffold/.git` before copying if present).

- [ ] **Step 2: Add test deps and Rust deps**

```bash
npm i -D vitest jsdom @solidjs/testing-library @testing-library/jest-dom
npm i @fontsource-variable/plus-jakarta-sans
cd src-tauri && cargo add serde_yaml@0.9 chrono --features chrono/serde && cargo add --dev tempfile && cd ..
```

- [ ] **Step 3: Configure Vitest.** Change the first import in `vite.config.ts` to `import { defineConfig } from "vitest/config";` and add to the config object:

```ts
  resolve: { conditions: ["browser"] },
  test: { environment: "jsdom", setupFiles: ["./src/test-setup.ts"] },
```
Create `src/test-setup.ts`:

```ts
import "@testing-library/jest-dom/vitest";
```
Add `"test": "vitest run"` to `package.json` scripts.

- [ ] **Step 4: Verify both toolchains run**

Run: `npx vitest run --passWithNoTests` -> exits 0. Run: `cd src-tauri && cargo test` -> exits 0.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "chore: scaffold Tauri 2 + Solid app with vitest" -m "Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Model and ranking rules (pure Rust)

**Files:**
- Create: `src-tauri/src/model.rs`, `src-tauri/src/rank.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod model; mod rank;`)

**Interfaces:**
- Produces (`model.rs`): `Kind {Due, Unpushed, Stale}`, `Item {kind, title, reason}`, `Deadline {title: String, due: NaiveDate, project: Option<String>}`, `RepoInfo {name: String, unpushed: u32, last_commit_days: i64}`, `ScheduleEntry {title, due: String, days_left: i64}`, `Stats {open, due_soon, unpushed: usize}`, `DayView {today: String, items, schedule, stats, warnings: Vec<String>}`.
- Produces (`rank.rs`): `DUE_WINDOW_DAYS`, `STALE_DAYS`, `build_day(today: NaiveDate, deadlines: &[Deadline], repos: &[RepoInfo], vault_notes: &[String], warnings: Vec<String>) -> DayView`.

- [ ] **Step 1: Write `model.rs`**

```rust
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
```

- [ ] **Step 2: Write the failing tests at the bottom of `rank.rs`** (file starts with only `use` lines and the tests; implementation comes in step 4)

```rust
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
}
```

- [ ] **Step 3: Run to verify failure.** Add `mod model; mod rank;` to `lib.rs`. Run: `cd src-tauri && cargo test rank` -> FAIL (`build_day` not found).

- [ ] **Step 4: Implement above the tests in `rank.rs`**

```rust
use crate::model::*;
use chrono::NaiveDate;

pub const DUE_WINDOW_DAYS: i64 = 3;
pub const STALE_DAYS: i64 = 14;

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
        .filter(|r| vault_notes.iter().any(|n| n.eq_ignore_ascii_case(&r.name)))
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
```

- [ ] **Step 5: Run to verify pass.** Run: `cd src-tauri && cargo test rank` -> 7 passed.

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat: ranking rules with reasons" -m "Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Deadlines file (parse, tolerate bad entries, create sample)

**Files:**
- Create: `src-tauri/src/deadlines.rs`
- Modify: `src-tauri/src/lib.rs` (`mod deadlines;`)

**Interfaces:**
- Consumes: `model::Deadline`.
- Produces: `parse(text: &str) -> (Vec<Deadline>, Vec<String>)`; `load(path: &Path) -> (Vec<Deadline>, Vec<String>)` (creates a commented sample file if missing).

- [ ] **Step 1: Write the failing tests** (bottom of `deadlines.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_entries() {
        let (ds, w) = parse("- title: Lab report\n  due: 2026-10-12\n  project: ktu\n- title: Quiz\n  due: 2026-10-09\n");
        assert_eq!(ds.len(), 2);
        assert_eq!(ds[0].project.as_deref(), Some("ktu"));
        assert!(w.is_empty());
    }

    #[test]
    fn bad_date_skips_only_that_entry_with_warning() {
        let (ds, w) = parse("- title: Good\n  due: 2026-10-12\n- title: Bad\n  due: next friday\n");
        assert_eq!(ds.len(), 1);
        assert_eq!(w.len(), 1);
        assert!(w[0].contains("Bad"));
    }

    #[test]
    fn malformed_yaml_returns_empty_with_warning() {
        let (ds, w) = parse("- title: [unclosed\n");
        assert!(ds.is_empty());
        assert_eq!(w.len(), 1);
        assert!(w[0].starts_with("deadlines.yaml"));
    }

    #[test]
    fn empty_or_comment_only_is_fine() {
        assert_eq!(parse("").1.len(), 0);
        assert_eq!(parse("# nothing yet\n").0.len(), 0);
        assert_eq!(parse("# nothing yet\n").1.len(), 0);
    }

    #[test]
    fn load_creates_sample_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("deadlines.yaml");
        let (ds, w) = load(&path);
        assert!(ds.is_empty() && w.is_empty());
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("# - title:"));
    }
}
```

- [ ] **Step 2: Run to verify failure.** `cd src-tauri && cargo test deadlines` -> FAIL.

- [ ] **Step 3: Implement above the tests**

```rust
use crate::model::Deadline;
use chrono::NaiveDate;
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize)]
struct Raw {
    title: String,
    due: String,
    #[serde(default)]
    project: Option<String>,
}

const SAMPLE: &str = "# Add deadlines as a list. Dates are YYYY-MM-DD.\n# - title: Submit lab report\n#   due: 2026-10-12\n#   project: ktu\n";

pub fn parse(text: &str) -> (Vec<Deadline>, Vec<String>) {
    if text.lines().all(|l| l.trim().is_empty() || l.trim_start().starts_with('#')) {
        return (vec![], vec![]);
    }
    let raws: Vec<Raw> = match serde_yaml::from_str(text) {
        Ok(r) => r,
        Err(e) => return (vec![], vec![format!("deadlines.yaml could not be read: {e}")]),
    };
    let mut out = Vec::new();
    let mut warnings = Vec::new();
    for r in raws {
        match NaiveDate::parse_from_str(r.due.trim(), "%Y-%m-%d") {
            Ok(due) => out.push(Deadline { title: r.title, due, project: r.project }),
            Err(_) => warnings.push(format!(
                "deadlines.yaml: \"{}\" has a bad date \"{}\" (use YYYY-MM-DD)",
                r.title, r.due
            )),
        }
    }
    (out, warnings)
}

pub fn load(path: &Path) -> (Vec<Deadline>, Vec<String>) {
    match fs::read_to_string(path) {
        Ok(text) => parse(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(path, SAMPLE);
            (vec![], vec![])
        }
        Err(e) => (vec![], vec![format!("deadlines.yaml could not be opened: {e}")]),
    }
}
```

- [ ] **Step 4: Run to verify pass.** `cargo test deadlines` -> 5 passed.

- [ ] **Step 5: Commit** `git add -A && git commit -m "feat: deadlines.yaml loader" -m "Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"`

---

### Task 4: Git scan and vault note names

**Files:**
- Create: `src-tauri/src/git.rs`, `src-tauri/src/vault.rs`
- Modify: `src-tauri/src/lib.rs` (`mod git; mod vault;`)

**Interfaces:**
- Consumes: `model::RepoInfo`.
- Produces: `git::inspect(path: &Path, now_epoch: i64) -> Option<RepoInfo>`; `git::scan(root: &Path, now_epoch: i64) -> Vec<RepoInfo>`; `vault::project_note_names(dir: &Path) -> Vec<String>` (lowercase file stems of `.md`, empty if dir missing).

- [ ] **Step 1: Write failing tests** (bottom of `git.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn run(dir: &Path, args: &[&str], date: Option<&str>) {
        let mut c = Command::new("git");
        c.current_dir(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args);
        if let Some(d) = date {
            c.env("GIT_AUTHOR_DATE", d).env("GIT_COMMITTER_DATE", d);
        }
        assert!(c.output().unwrap().status.success(), "git {:?} failed", args);
    }
    fn now() -> i64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64
    }

    #[test]
    fn repo_without_upstream_has_zero_unpushed() {
        let t = tempfile::tempdir().unwrap();
        let r = t.path().join("Alpha");
        std::fs::create_dir(&r).unwrap();
        run(&r, &["init", "-q"], None);
        run(&r, &["commit", "-q", "--allow-empty", "-m", "x"], None);
        let info = inspect(&r, now()).unwrap();
        assert_eq!(info.name, "Alpha");
        assert_eq!(info.unpushed, 0);
        assert_eq!(info.last_commit_days, 0);
    }

    #[test]
    fn counts_commits_ahead_of_upstream() {
        let t = tempfile::tempdir().unwrap();
        let bare = t.path().join("bare.git");
        std::fs::create_dir(&bare).unwrap();
        run(&bare, &["init", "-q", "--bare"], None);
        let r = t.path().join("Beta");
        std::fs::create_dir(&r).unwrap();
        run(&r, &["init", "-q", "-b", "main"], None);
        run(&r, &["commit", "-q", "--allow-empty", "-m", "one"], None);
        run(&r, &["remote", "add", "origin", bare.to_str().unwrap()], None);
        run(&r, &["push", "-q", "-u", "origin", "main"], None);
        run(&r, &["commit", "-q", "--allow-empty", "-m", "two"], None);
        run(&r, &["commit", "-q", "--allow-empty", "-m", "three"], None);
        assert_eq!(inspect(&r, now()).unwrap().unpushed, 2);
    }

    #[test]
    fn old_commit_gives_large_age() {
        let t = tempfile::tempdir().unwrap();
        let r = t.path().join("Old");
        std::fs::create_dir(&r).unwrap();
        run(&r, &["init", "-q"], None);
        run(&r, &["commit", "-q", "--allow-empty", "-m", "x"], Some("2020-01-01T00:00:00"));
        assert!(inspect(&r, now()).unwrap().last_commit_days > 365);
    }

    #[test]
    fn scan_skips_non_repos_and_commitless_repos() {
        let t = tempfile::tempdir().unwrap();
        std::fs::create_dir(t.path().join("plain-folder")).unwrap();
        std::fs::write(t.path().join("file.txt"), "x").unwrap();
        let empty = t.path().join("Empty");
        std::fs::create_dir(&empty).unwrap();
        run(&empty, &["init", "-q"], None);
        let ok = t.path().join("Ok");
        std::fs::create_dir(&ok).unwrap();
        run(&ok, &["init", "-q"], None);
        run(&ok, &["commit", "-q", "--allow-empty", "-m", "x"], None);
        let found = scan(t.path(), now());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Ok");
    }

    #[test]
    fn scan_of_missing_root_is_empty() {
        assert!(scan(Path::new("Z:/definitely/not/here"), now()).is_empty());
    }
}
```

- [ ] **Step 2: Run to verify failure.** `cargo test git::` -> FAIL.

- [ ] **Step 3: Implement above the tests in `git.rs`**

```rust
use crate::model::RepoInfo;
use std::{path::Path, process::Command};

fn git(path: &Path, args: &[&str]) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(path).args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: no console flash
    }
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub fn inspect(path: &Path, now_epoch: i64) -> Option<RepoInfo> {
    if !path.join(".git").exists() {
        return None;
    }
    let name = path.file_name()?.to_string_lossy().to_string();
    let unpushed = git(path, &["rev-list", "--count", "@{u}..HEAD"])
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let last: i64 = git(path, &["log", "-1", "--format=%ct"])?.parse().ok()?;
    Some(RepoInfo { name, unpushed, last_commit_days: ((now_epoch - last) / 86_400).max(0) })
}

pub fn scan(root: &Path, now_epoch: i64) -> Vec<RepoInfo> {
    let Ok(entries) = std::fs::read_dir(root) else { return vec![] };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| inspect(&e.path(), now_epoch))
        .collect()
}
```

- [ ] **Step 4: Write `vault.rs` with its test**

```rust
use std::path::Path;

pub fn project_note_names(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else { return vec![] };
    entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "md").unwrap_or(false))
        .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_lowercase_md_stems_only() {
        let t = tempfile::tempdir().unwrap();
        std::fs::write(t.path().join("Winnow.md"), "x").unwrap();
        std::fs::write(t.path().join("notes.txt"), "x").unwrap();
        assert_eq!(project_note_names(t.path()), vec!["winnow".to_string()]);
    }

    #[test]
    fn missing_dir_is_empty() {
        assert!(project_note_names(Path::new("Z:/nope")).is_empty());
    }
}
```

- [ ] **Step 5: Run to verify pass.** `cargo test` -> all tests pass (git 5, vault 2, plus earlier).

- [ ] **Step 6: Commit** `git add -A && git commit -m "feat: git scan and vault note names" -m "Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"`

---

### Task 5: `get_day` command

**Files:**
- Modify: `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml` (none beyond existing)

**Interfaces:**
- Consumes: `deadlines::load`, `git::scan`, `vault::project_note_names`, `rank::build_day`.
- Produces: Tauri command `get_day() -> DayView`, invoked from the frontend as `invoke("get_day")`.

- [ ] **Step 1: Replace the scaffold's greet command in `lib.rs`** (keep `mod` lines and `run()` entry the scaffold made):

```rust
mod deadlines;
mod git;
mod model;
mod rank;
mod vault;

use chrono::Local;
use model::DayView;
use tauri::Manager;

#[tauri::command]
fn get_day(app: tauri::AppHandle) -> DayView {
    let now = Local::now();
    let today = now.date_naive();
    let mut warnings = Vec::new();

    let home = app.path().home_dir().unwrap_or_default();
    let repos = git::scan(&home.join("Documents").join("GitHub"), now.timestamp());
    let notes = vault::project_note_names(
        &home.join("Documents").join("Vault").join("_brain").join("projects"),
    );

    let deadlines = match app.path().app_data_dir() {
        Ok(dir) => {
            let (d, w) = deadlines::load(&dir.join("deadlines.yaml"));
            warnings.extend(w);
            d
        }
        Err(e) => {
            warnings.push(format!("app data folder unavailable: {e}"));
            vec![]
        }
    };

    rank::build_day(today, &deadlines, &repos, &notes, warnings)
}
```
In `run()` use `.invoke_handler(tauri::generate_handler![get_day])` and delete the scaffold's `greet`.

- [ ] **Step 2: Verify it compiles.** `cd src-tauri && cargo check` -> no errors.

- [ ] **Step 3: Commit** `git add -A && git commit -m "feat: get_day command" -m "Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"`

---

### Task 6: Design tokens and DESIGN.md

**Files:**
- Create: `DESIGN.md`, `src/tokens.css`

- [ ] **Step 1: Read vault design references** (`_wiki/concepts/typography-pairings`, `color-palettes`, `layout-patterns`, `design-aesthetics-catalog`) via `mcp__vault__read_note`. If any contradicts the tokens below, adjust the tokens and say so in DESIGN.md.

- [ ] **Step 2: Write `src/tokens.css`**

```css
@import "@fontsource-variable/plus-jakarta-sans";

:root {
  --font: "Plus Jakarta Sans Variable", system-ui, sans-serif;
  --bg: oklch(0.985 0.004 290);
  --ink: oklch(0.22 0.02 280);
  --ink-2: oklch(0.5 0.02 280);
  --line: oklch(0.92 0.008 280);
  --card: oklch(1 0 0 / 0.8);
  --shadow: 0 1px 2px oklch(0.3 0.02 280 / 0.06), 0 8px 24px oklch(0.3 0.02 280 / 0.06);

  --due: oklch(0.955 0.04 55);      --due-ink: oklch(0.48 0.14 50);
  --work: oklch(0.955 0.035 295);   --work-ink: oklch(0.45 0.13 295);
  --stale: oklch(0.955 0.015 240);  --stale-ink: oklch(0.45 0.05 240);
  --clear: oklch(0.955 0.045 155);  --clear-ink: oklch(0.42 0.1 155);

  --radius: 22px;
  --gap: clamp(12px, 1vw + 8px, 20px);
  --fs-title: clamp(1.6rem, 1.2rem + 1.2vw, 2.2rem);
  --fs-stat: clamp(2rem, 1.4rem + 2vw, 3rem);
  --fs-body: clamp(0.85rem, 0.8rem + 0.2vw, 0.95rem);
}

* { box-sizing: border-box; }

body {
  margin: 0;
  font-family: var(--font);
  font-size: var(--fs-body);
  color: var(--ink);
  background:
    radial-gradient(60rem 40rem at 0% 0%, oklch(0.95 0.04 295 / 0.7), transparent 60%),
    radial-gradient(50rem 40rem at 100% 100%, oklch(0.95 0.05 155 / 0.6), transparent 60%),
    radial-gradient(40rem 30rem at 70% 90%, oklch(0.96 0.04 55 / 0.5), transparent 60%),
    var(--bg);
  min-height: 100vh;
}
```

- [ ] **Step 3: Write `DESIGN.md`** (30-50 lines): palette tokens above and their meaning (peach=due, lavender=unpushed, slate=stale, mint=all-clear), Plus Jakarta Sans only, 22px radius cards with hairline border and soft shadow, reference Dribbble shot 3 "My Design Tasks", the one deliberate risk (urgency tint instead of priority badge), light only, reserved empty pet slot bottom-right.

- [ ] **Step 4: Commit** `git add -A && git commit -m "docs: DESIGN.md and tokens" -m "Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"`

---

### Task 7: Frontend (types, api, components, tests)

**Files:**
- Create: `src/types.ts`, `src/format.ts`, `src/format.test.ts`, `src/fixture.ts`, `src/api.ts`, `src/components/StatTrio.tsx`, `src/components/TaskCard.tsx`, `src/components/Schedule.tsx`, `src/components/TaskCard.test.tsx`, `src/App.test.tsx`
- Modify: `src/App.tsx`, `src/index.tsx` (import `./tokens.css`, `./app.css`), delete scaffold `App.css`
- Create: `src/app.css`

**Interfaces:**
- Consumes: Rust `DayView` JSON (snake_case fields: `today`, `items[{kind,title,reason}]`, `schedule[{title,due,days_left}]`, `stats{open,due_soon,unpushed}`, `warnings[]`).
- Produces: `getDay(): Promise<DayView>`; `dayLabel(daysLeft: number): string`.

- [ ] **Step 1: `src/types.ts`**

```ts
export type Kind = "due" | "unpushed" | "stale";
export interface Item { kind: Kind; title: string; reason: string }
export interface ScheduleEntry { title: string; due: string; days_left: number }
export interface Stats { open: number; due_soon: number; unpushed: number }
export interface DayView {
  today: string;
  items: Item[];
  schedule: ScheduleEntry[];
  stats: Stats;
  warnings: string[];
}
```

- [ ] **Step 2: Failing test `src/format.test.ts`**

```ts
import { describe, it, expect } from "vitest";
import { dayLabel, chipLabel } from "./format";

describe("format", () => {
  it("labels days", () => {
    expect(dayLabel(-2)).toBe("Overdue");
    expect(dayLabel(0)).toBe("Today");
    expect(dayLabel(1)).toBe("Tomorrow");
    expect(dayLabel(5)).toBe("In 5 days");
  });
  it("labels chips", () => {
    expect(chipLabel("due")).toBe("Due");
    expect(chipLabel("unpushed")).toBe("Unpushed");
    expect(chipLabel("stale")).toBe("Stale");
  });
});
```
Run: `npx vitest run src/format.test.ts` -> FAIL.

- [ ] **Step 3: `src/format.ts`**

```ts
import type { Kind } from "./types";

export function dayLabel(daysLeft: number): string {
  if (daysLeft < 0) return "Overdue";
  if (daysLeft === 0) return "Today";
  if (daysLeft === 1) return "Tomorrow";
  return `In ${daysLeft} days`;
}

export function chipLabel(kind: Kind): string {
  return { due: "Due", unpushed: "Unpushed", stale: "Stale" }[kind];
}
```
Run -> PASS.

- [ ] **Step 4: `src/fixture.ts` and `src/api.ts`** (fixture is the browser-only dev fallback, used for Playwright screenshots)

```ts
// fixture.ts
import type { DayView } from "./types";
export const fixture: DayView = {
  today: "2026-10-08",
  items: [
    { kind: "due", title: "Lab report", reason: "due tomorrow" },
    { kind: "due", title: "OS quiz", reason: "due in 3 days" },
    { kind: "unpushed", title: "Winnow", reason: "3 commits unpushed" },
    { kind: "stale", title: "Deadwax", reason: "stale 21 days" },
  ],
  schedule: [
    { title: "Lab report", due: "2026-10-09", days_left: 1 },
    { title: "OS quiz", due: "2026-10-11", days_left: 3 },
    { title: "Project demo", due: "2026-10-20", days_left: 12 },
  ],
  stats: { open: 4, due_soon: 2, unpushed: 1 },
  warnings: [],
};
```

```ts
// api.ts
import { invoke } from "@tauri-apps/api/core";
import type { DayView } from "./types";
import { fixture } from "./fixture";

export async function getDay(): Promise<DayView> {
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) return fixture;
  return invoke<DayView>("get_day");
}
```

- [ ] **Step 5: Failing component tests.** `src/components/TaskCard.test.tsx`:

```tsx
// @vitest-environment jsdom
import { render, screen } from "@solidjs/testing-library";
import { describe, it, expect } from "vitest";
import TaskCard from "./TaskCard";

describe("TaskCard", () => {
  it("shows title, reason, chip and kind class", () => {
    const { container } = render(() => (
      <TaskCard item={{ kind: "due", title: "Lab report", reason: "due tomorrow" }} />
    ));
    expect(screen.getByText("Lab report")).toBeInTheDocument();
    expect(screen.getByText("due tomorrow")).toBeInTheDocument();
    expect(screen.getByText("Due")).toBeInTheDocument();
    expect(container.querySelector(".card--due")).not.toBeNull();
  });
});
```

`src/App.test.tsx`:

```tsx
// @vitest-environment jsdom
import { render, screen } from "@solidjs/testing-library";
import { describe, it, expect, vi } from "vitest";
import { fixture } from "./fixture";

vi.mock("./api", () => ({ getDay: vi.fn() }));
import { getDay } from "./api";
import App from "./App";

describe("App", () => {
  it("renders ranked items, stats and schedule", async () => {
    vi.mocked(getDay).mockResolvedValue(fixture);
    render(() => <App />);
    expect(await screen.findByText("Lab report", { selector: ".card h3" })).toBeInTheDocument();
    expect(screen.getByText("3 commits unpushed")).toBeInTheDocument();
    expect(screen.getByText("Tomorrow")).toBeInTheDocument();
  });

  it("shows an all-clear state when nothing to do", async () => {
    vi.mocked(getDay).mockResolvedValue({
      today: "2026-10-08", items: [], schedule: [],
      stats: { open: 0, due_soon: 0, unpushed: 0 }, warnings: [],
    });
    render(() => <App />);
    expect(await screen.findByText(/all clear/i)).toBeInTheDocument();
  });

  it("shows warnings without hiding the list", async () => {
    vi.mocked(getDay).mockResolvedValue({ ...fixture, warnings: ["deadlines.yaml: \"Bad\" has a bad date"] });
    render(() => <App />);
    expect(await screen.findByText(/has a bad date/)).toBeInTheDocument();
    expect(screen.getByText("3 commits unpushed")).toBeInTheDocument();
  });

  it("shows an error state when the backend call fails", async () => {
    vi.mocked(getDay).mockRejectedValue(new Error("boom"));
    render(() => <App />);
    expect(await screen.findByText(/could not load/i)).toBeInTheDocument();
  });
});
```
Run: `npx vitest run` -> FAIL.

- [ ] **Step 6: Components**

`src/components/TaskCard.tsx`:

```tsx
import type { Item } from "../types";
import { chipLabel } from "../format";

export default function TaskCard(props: { item: Item }) {
  return (
    <article class={`card card--${props.item.kind}`}>
      <h3>{props.item.title}</h3>
      <p class="reason">{props.item.reason}</p>
      <span class="chip">{chipLabel(props.item.kind)}</span>
    </article>
  );
}
```

`src/components/StatTrio.tsx`:

```tsx
import type { Stats } from "../types";

export default function StatTrio(props: { stats: Stats }) {
  const cells = () => [
    { n: props.stats.open, label: "Open" },
    { n: props.stats.due_soon, label: "Due soon" },
    { n: props.stats.unpushed, label: "Unpushed" },
  ];
  return (
    <div class="stats">
      {cells().map((c) => (
        <div class="stat">
          <span class="stat-n">{c.n}</span>
          <span class="stat-l">{c.label}</span>
        </div>
      ))}
    </div>
  );
}
```

`src/components/Schedule.tsx`:

```tsx
import { For } from "solid-js";
import type { ScheduleEntry } from "../types";
import { dayLabel } from "../format";

export default function Schedule(props: { entries: ScheduleEntry[] }) {
  return (
    <aside class="schedule">
      <h2>Schedule</h2>
      <For each={props.entries} fallback={<p class="reason">No deadlines yet. Add some to deadlines.yaml.</p>}>
        {(e) => (
          <div class={`sched sched--${e.days_left <= 3 ? "soon" : "later"}`}>
            <div class="sched-top"><strong>{e.title}</strong><span>{dayLabel(e.days_left)}</span></div>
            <p class="reason">{e.due}</p>
          </div>
        )}
      </For>
    </aside>
  );
}
```

`src/App.tsx`:

```tsx
import { createResource, For, Show } from "solid-js";
import { getDay } from "./api";
import StatTrio from "./components/StatTrio";
import TaskCard from "./components/TaskCard";
import Schedule from "./components/Schedule";

export default function App() {
  const [day] = createResource(getDay);
  return (
    <main class="shell">
      <header>
        <h1>My Day</h1>
        <Show when={day()}><p class="reason">{day()!.today}</p></Show>
      </header>
      <Show when={!day.error} fallback={<p class="banner">Could not load your day. Restart the app.</p>}>
        <Show when={day()} fallback={<p class="reason">Loading...</p>}>
          {(d) => (
            <>
              <For each={d().warnings}>{(w) => <p class="banner">{w}</p>}</For>
              <StatTrio stats={d().stats} />
              <div class="cols">
                <section class="list">
                  <For each={d().items} fallback={<div class="card card--clear"><h3>All clear</h3><p class="reason">Nothing due, nothing unpushed. Go build something.</p></div>}>
                    {(i) => <TaskCard item={i} />}
                  </For>
                </section>
                <Schedule entries={d().schedule} />
              </div>
              <div class="pet-slot" aria-hidden="true" />
            </>
          )}
        </Show>
      </Show>
    </main>
  );
}
```

- [ ] **Step 7: `src/app.css`** (uses tokens; import both in `src/index.tsx`: `import "./tokens.css"; import "./app.css";`)

```css
.shell { max-width: 1100px; margin: 0 auto; padding: var(--gap) calc(var(--gap) * 1.5); }
h1 { font-size: var(--fs-title); margin: 0; letter-spacing: -0.02em; }
h2 { font-size: 1.1rem; margin: 0 0 12px; }
.reason { color: var(--ink-2); margin: 4px 0 0; }
.banner { background: var(--due); color: var(--due-ink); border-radius: 14px; padding: 10px 14px; margin: 12px 0; }
.stats { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--gap); margin: var(--gap) 0; }
.stat { background: var(--card); border: 1px solid var(--line); border-radius: var(--radius); box-shadow: var(--shadow); padding: 14px 18px; display: flex; flex-direction: column; }
.stat-n { font-size: var(--fs-stat); font-weight: 700; font-variant-numeric: tabular-nums; line-height: 1; }
.stat-l { color: var(--ink-2); margin-top: 6px; }
.cols { display: grid; grid-template-columns: minmax(0, 1.6fr) minmax(0, 1fr); gap: var(--gap); align-items: start; }
.list { display: grid; gap: var(--gap); }
.card { position: relative; background: var(--card); border: 1px solid var(--line); border-radius: var(--radius); box-shadow: var(--shadow); padding: 18px 20px; animation: rise 0.35s ease-out both; }
.card h3 { margin: 0; font-size: 1.05rem; }
.card--due { background: var(--due); } .card--due .chip { color: var(--due-ink); }
.card--unpushed { background: var(--work); } .card--unpushed .chip { color: var(--work-ink); }
.card--stale { background: var(--stale); } .card--stale .chip { color: var(--stale-ink); }
.card--clear { background: var(--clear); } .card--clear h3 { color: var(--clear-ink); }
.chip { display: inline-block; margin-top: 12px; padding: 3px 12px; border-radius: 999px; background: oklch(1 0 0 / 0.7); font-weight: 600; font-size: 0.8rem; }
.schedule { background: var(--card); border: 1px solid var(--line); border-radius: var(--radius); box-shadow: var(--shadow); padding: 18px 20px; display: grid; gap: 12px; }
.sched { border-radius: 16px; padding: 12px 14px; background: var(--stale); }
.sched--soon { background: var(--due); }
.sched-top { display: flex; justify-content: space-between; gap: 8px; }
.pet-slot { position: fixed; right: 20px; bottom: 20px; width: 72px; height: 72px; pointer-events: none; }
@keyframes rise { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: none; } }
@media (max-width: 760px) { .cols { grid-template-columns: 1fr; } }
@media (prefers-reduced-motion: reduce) { .card { animation: none; } }
```

- [ ] **Step 8: Run to verify pass.** `npx vitest run` -> all pass (format 2, TaskCard 1, App 4). Remove the scaffold's `App.css`/logo leftovers if unused.

- [ ] **Step 9: Commit** `git add -A && git commit -m "feat: dashboard UI" -m "Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"`

---

### Task 8: Tray, close-to-tray, autostart, CSP

**Files:**
- Modify: `src-tauri/Cargo.toml` (tauri feature `tray-icon`; `tauri-plugin-autostart = "2"`), `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json` (add `"autostart:default"`), `src-tauri/tauri.conf.json` (CSP, window title "Dashboard", size 1100x760)

**Interfaces:**
- Consumes: scaffold's main window labelled `main`.

- [ ] **Step 1: Cargo.** In `src-tauri/Cargo.toml` set `tauri = { version = "2", features = ["tray-icon"] }` and run `cd src-tauri && cargo add tauri-plugin-autostart@2`.

- [ ] **Step 2: Set strict CSP** in `tauri.conf.json` under `app.security`:

```json
"csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self' data:; img-src 'self' data:"
```

- [ ] **Step 3: Add tray + autostart to `run()` in `lib.rs`**

```rust
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

fn show_main(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .invoke_handler(tauri::generate_handler![get_day])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let auto = CheckMenuItem::with_id(
                app, "autostart", "Start with Windows", true,
                app.autolaunch().is_enabled().unwrap_or(false), None::<&str>,
            )?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &auto, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, ev| match ev.id().as_ref() {
                    "open" => show_main(app),
                    "quit" => app.exit(0),
                    "autostart" => {
                        let al = app.autolaunch();
                        if al.is_enabled().unwrap_or(false) { let _ = al.disable(); } else { let _ = al.enable(); }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, ev| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Verify.** `cd src-tauri && cargo check` -> no errors. Run `npm run tauri dev`: window opens showing real data from your repos; close button hides to tray; tray left-click reopens; tray "Quit" exits; no console window flashes. Check `app/dist` for CSP-related blocking by confirming fonts render.

- [ ] **Step 5: Commit** `git add -A && git commit -m "feat: tray, close-to-tray, autostart toggle, strict CSP" -m "Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"`

---

### Task 9: Look verification, README, push

**Files:**
- Create: `README.md`
- Modify: UI CSS as the critique demands

- [ ] **Step 1: Screenshot the real UI.** Run `npm run dev` (browser fallback uses `fixture.ts`). Install Playwright outside the repo (`npm i -D playwright` is fine; do not commit browsers) and capture 1280x800 and 760x700 PNGs of `http://localhost:1420` into the scratchpad dir (not the repo).

- [ ] **Step 2: Critique against `DESIGN.md` and fix.** Check: card contrast of `--*-ink` text on tinted backgrounds (measure, do not eyeball), numerals are tabular, nothing overflows at 760px, the glass wash reads soft not muddy, no Inter fallback in use. Fix `app.css`/`tokens.css`, re-screenshot, repeat until clean.

- [ ] **Step 3: README.md** (human voice, short): what it is, "reads your repos, vault notes and deadlines.yaml, nothing leaves your machine", how to run (`npm install`, `npm run tauri dev`), where `deadlines.yaml` lives (`%APPDATA%\com.codedrichy.personaldashboard\deadlines.yaml`), the ranking rules, status: v0, personal use.

- [ ] **Step 4: Full verification.** `npm test` and `cd src-tauri && cargo test` both green; `npm run tauri build` completes.

- [ ] **Step 5: Commit and push**

```bash
git add -A && git commit -m "docs: README, polish after visual critique" -m "Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
git push
```

- [ ] **Step 6: One-week test.** Add real deadlines to `deadlines.yaml`, enable "Start with Windows" from the tray, and use it each morning for a week. Record in the vault whether you opened it unprompted.
