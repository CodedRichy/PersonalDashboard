// @vitest-environment jsdom
import { render, screen } from "@solidjs/testing-library";
import { describe, it, expect, vi } from "vitest";
import { fixture } from "./fixture";

vi.mock("./api", () => ({ getDay: vi.fn() }));
import { getDay } from "./api";
import App from "./App";

describe("App", () => {
  it("renders ranked items, stats and schedule", async () => {
    vi.mocked(getDay).mockResolvedValue(fixture);
    render(() => <App />);
    expect(await screen.findByText("Lab report", { selector: ".card h3" })).toBeInTheDocument();
    expect(screen.getByText("3 commits unpushed")).toBeInTheDocument();
    expect(screen.getByText("Tomorrow")).toBeInTheDocument();
  });

  it("shows an all-clear state when nothing to do", async () => {
    vi.mocked(getDay).mockResolvedValue({
      today: "2026-10-08", items: [], schedule: [],
      stats: { open: 0, due_soon: 0, unpushed: 0 }, warnings: [],
    });
    render(() => <App />);
    expect(await screen.findByText(/all clear/i)).toBeInTheDocument();
  });

  it("shows warnings without hiding the list", async () => {
    vi.mocked(getDay).mockResolvedValue({ ...fixture, warnings: ['deadlines.yaml: "Bad" has a bad date'] });
    render(() => <App />);
    expect(await screen.findByText(/has a bad date/)).toBeInTheDocument();
    expect(screen.getByText("3 commits unpushed")).toBeInTheDocument();
  });

  it("shows an error state when the backend call fails", async () => {
    vi.mocked(getDay).mockRejectedValue(new Error("boom"));
    render(() => <App />);
    expect(await screen.findByText(/could not load/i)).toBeInTheDocument();
  });
});
