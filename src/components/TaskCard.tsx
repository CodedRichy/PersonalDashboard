import type { Item } from "../types";
import { chipLabel } from "../format";

export default function TaskCard(props: { item: Item }) {
  return (
    <article class={`card card--${props.item.kind}`}>
      <h3>{props.item.title}</h3>
      <p class="reason">{props.item.reason}</p>
      <span class="chip">{chipLabel(props.item.kind)}</span>
    </article>
  );
}
