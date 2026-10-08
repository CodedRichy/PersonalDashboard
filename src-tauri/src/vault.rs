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
    use std::path::Path;

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
