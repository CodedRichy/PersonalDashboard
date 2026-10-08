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
