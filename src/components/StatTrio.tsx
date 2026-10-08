import { For } from "solid-js";
import type { Stats } from "../types";

export default function StatTrio(props: { stats: Stats }) {
  const cells = () => [
    { n: props.stats.open, label: "Open" },
    { n: props.stats.due_soon, label: "Due soon" },
    { n: props.stats.unpushed, label: "Unpushed" },
  ];
  return (
    <div class="stats">
      <For each={cells()}>
        {(c) => (
          <div class="stat">
            <span class="stat-n">{c.n}</span>
            <span class="stat-l">{c.label}</span>
          </div>
        )}
      </For>
    </div>
  );
}
