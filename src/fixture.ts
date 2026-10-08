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
