import { invoke } from "@tauri-apps/api/core";
import type { DayView } from "./types";
import { fixture } from "./fixture";

export async function getDay(): Promise<DayView> {
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) return fixture;
  return invoke<DayView>("get_day");
}
