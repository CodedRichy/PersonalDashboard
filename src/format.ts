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
