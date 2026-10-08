use crate::model::RepoInfo;
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const GIT_TIMEOUT: Duration = Duration::from_secs(8);

pub fn run_with_timeout(mut cmd: Command, timeout: Duration) -> Option<String> {
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: no console flash
    }
    let mut child = cmd.spawn().ok()?;
    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });
    let start = Instant::now();
    loop {
        match child.try_wait().ok()? {
            Some(status) => {
                let buf = reader.join().ok()?;
                return status
                    .success()
                    .then(|| String::from_utf8_lossy(&buf).trim().to_string());
            }
            None if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                return None;
            }
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    }
}

fn git(path: &Path, args: &[&str]) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(path).args(args).env("GIT_TERMINAL_PROMPT", "0");
    run_with_timeout(cmd, GIT_TIMEOUT)
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
    let dirs: Vec<PathBuf> =
        entries.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_dir()).collect();
    let mut repos: Vec<RepoInfo> = std::thread::scope(|s| {
        let handles: Vec<_> = dirs.iter().map(|d| s.spawn(move || inspect(d, now_epoch))).collect();
        handles.into_iter().filter_map(|h| h.join().ok().flatten()).collect()
    });
    repos.sort_by(|a, b| a.name.cmp(&b.name));
    repos
}

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

    #[cfg(windows)]
    #[test]
    fn hung_command_is_killed_after_timeout() {
        let mut c = Command::new("ping");
        c.args(["-n", "30", "127.0.0.1"]);
        let start = std::time::Instant::now();
        assert!(run_with_timeout(c, std::time::Duration::from_millis(300)).is_none());
        assert!(start.elapsed() < std::time::Duration::from_secs(5));
    }
}
