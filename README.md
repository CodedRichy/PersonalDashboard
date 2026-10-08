# PersonalDashboard

A small desktop app that tells me what to do today, and why.

It reads three things on my own machine: my git repos, my notes about each project, and a hand-edited list of deadlines. It ranks them with plain rules and shows the result. Nothing leaves the computer, there is no account, and there is no AI in the loop.

## What it shows

Items are ranked in this order, and each one says why it is there:

1. A deadline within 3 days, or already overdue. ("due tomorrow", "overdue 2 days")
2. A repo with commits that were never pushed. ("3 commits unpushed")
3. A project with a note that has not been touched for 14 days. ("stale 21 days")

Anything else stays hidden. A Schedule column lists every deadline in date order.

## Run it

```
npm install
npm run tauri dev
```

You need Node, and Rust on the MSVC toolchain (the repo pins it). Tests: `npm test` and `cargo test` in `src-tauri`.

## Your data

- Repos: every folder under `Documents\GitHub` that is a git repo.
- Project notes: `Documents\Vault\_brain\projects\*.md`, matched to repos by name.
- Deadlines: `%APPDATA%\com.codedrichy.personaldashboard\deadlines.yaml`, created on first run with a commented example.

```yaml
- title: Submit lab report
  due: 2026-10-12
```

A bad date or broken YAML shows a warning on screen and the rest still loads.

## Tray

Closing the window hides it to the tray. Left-click the tray icon to reopen it. The tray menu has "Start with Windows" and "Quit".

## Status

v0, built for one person. The test is whether I open it every morning for a week. The look is in `DESIGN.md`; the plan and spec are in `docs/superpowers`.
