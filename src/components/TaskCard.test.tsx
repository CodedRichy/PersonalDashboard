// @vitest-environment jsdom
import { render, screen } from "@solidjs/testing-library";
import { describe, it, expect } from "vitest";
import TaskCard from "./TaskCard";

describe("TaskCard", () => {
  it("shows title, reason, chip and kind class", () => {
    const { container } = render(() => (
      <TaskCard item={{ kind: "due", title: "Lab report", reason: "due tomorrow" }} />
    ));
    expect(screen.getByText("Lab report")).toBeInTheDocument();
    expect(screen.getByText("due tomorrow")).toBeInTheDocument();
    expect(screen.getByText("Due")).toBeInTheDocument();
    expect(container.querySelector(".card--due")).not.toBeNull();
  });
});
