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
