import { createResource, For, onCleanup, onMount, Show } from "solid-js";
import { getDay } from "./api";
import StatTrio from "./components/StatTrio";
import TaskCard from "./components/TaskCard";
import Schedule from "./components/Schedule";

export default function App() {
  const [day, { refetch }] = createResource(getDay);
  const ready = () => day.state === "ready" || day.state === "refreshing";

  // The app lives in the tray for days: reload whenever the window is shown again.
  onMount(() => {
    const reload = () => { if (!document.hidden) refetch(); };
    window.addEventListener("focus", reload);
    document.addEventListener("visibilitychange", reload);
    onCleanup(() => {
      window.removeEventListener("focus", reload);
      document.removeEventListener("visibilitychange", reload);
    });
  });
  return (
    <main class="shell">
      <header>
        <h1>My Day</h1>
        <Show when={ready()}><p class="reason">{day()!.today}</p></Show>
      </header>
      <Show when={day.state !== "errored"} fallback={<p class="banner">Could not load your day. Restart the app.</p>}>
        <Show when={ready() ? day() : undefined} fallback={<p class="reason">Loading...</p>}>
          {(d) => (
            <>
              <For each={d().warnings}>{(w) => <p class="banner">{w}</p>}</For>
              <StatTrio stats={d().stats} />
              <div class="cols">
                <section class="list">
                  <For each={d().items} fallback={<div class="card card--clear"><h3>All clear</h3><p class="reason">Nothing due, nothing unpushed. Go build something.</p></div>}>
                    {(i) => <TaskCard item={i} />}
                  </For>
                </section>
                <Schedule entries={d().schedule} />
              </div>
              <div class="pet-slot" aria-hidden="true" />
            </>
          )}
        </Show>
      </Show>
    </main>
  );
}
