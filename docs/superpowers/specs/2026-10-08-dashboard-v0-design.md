# Dashboard v0 - design

Status: draft for Rishi's review. Date: 2026-10-08.

## Purpose
A personal dashboard for Rishi's own life. Opened each morning, it answers one question in two seconds: what should I do today, and why. v0 is for him only; commercial, pet, focus mode and screen helper are out of scope.

## Success test
Rishi opens it every morning for one week without being asked. If he stops, v0 failed and we learn why before building more.

## Shell
Tauri 2 + SolidJS desktop app. Tray icon, optional start-with-Windows. Rust side reads files and runs `git`; no server, no network, no accounts.

## Data (v0 reads only these)
1. Git repos under `C:\Users\rishi\Documents\GitHub` (via `git`): unpushed commits, dirty tree, last-commit age.
2. Vault project notes under `Documents\Vault\_brain\projects` (title, updated date).
3. `deadlines.yaml` in the app data dir, edited by hand: `title`, `due`, optional `project`.

## Ranking (plain rules, each item carries its reason string)
1. Deadline within 3 days (soonest first). Reason: "due in 2 days".
2. Repo with unpushed commits. Reason: "3 commits unpushed".
3. Project untouched 14+ days that has a vault note. Reason: "stale 21 days".
Everything else is hidden. No LLM, no scoring model.

## Layout (single window)
- Header: "My Day" title, date, search-free.
- Stat trio: total open / due soon / unpushed, big numerals.
- Centre column: ranked task cards (title, one-line reason, status chip: Due / Unpushed / Stale).
- Right column: "Schedule", the deadlines in date order with day labels.
- Corner slot reserved (empty) for the future pet.

## Look (reference: Dribbble shot 3, "My Design Tasks")
- Light, airy, soft glass cards on a faint pastel gradient wash (lavender, mint, peach at low chroma). Rounded 20-24px cards, hairline borders, soft shadow.
- Colour carries meaning: peach = due soon, lavender = work in progress, mint = done/clean.
- Small pill chips with icon and label; round icon buttons; rail of icon nav on the left.
- Font: Plus Jakarta Sans (Inter is banned by Rishi's rules). Large tabular numerals in the stat trio.
- OKLCH tokens, `clamp()` type. Light theme only in v0.
- The one deliberate risk: urgency tint on cards instead of a priority badge, so the page reads by colour before text.
- Before any frontend code: read vault `_wiki/concepts/` design notes and write `DESIGN.md`; screenshot the built UI with Playwright and critique before calling it done.

## Out of scope
Pet, focus mode, screen assistant, ActivityWatch, app/site tracking, LLM summary, portal scraping, accounts, sync, dark mode.

## Testing
Rust: unit tests for repo parsing and ranking rules (fixture repos in a temp dir). Frontend: Vitest + jsdom for the rank list (Solid needs `resolve.conditions: ['browser']`). Tauri CSP kept strict; file access only through invoke commands.

## Open assumptions to confirm
- Rishi approved the shot-3 look; the Tauri shell was proposed and not objected to, but never explicitly approved.
- 3-day and 14-day thresholds are guesses; make them constants.
