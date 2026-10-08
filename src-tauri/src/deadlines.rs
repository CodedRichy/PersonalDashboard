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
