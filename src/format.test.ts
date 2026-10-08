import { describe, it, expect } from "vitest";
import { dayLabel, chipLabel } from "./format";

describe("format", () => {
  it("labels days", () => {
    expect(dayLabel(-2)).toBe("Overdue");
    expect(dayLabel(0)).toBe("Today");
    expect(dayLabel(1)).toBe("Tomorrow");
    expect(dayLabel(5)).toBe("In 5 days");
  });
  it("labels chips", () => {
    expect(chipLabel("due")).toBe("Due");
    expect(chipLabel("unpushed")).toBe("Unpushed");
    expect(chipLabel("stale")).toBe("Stale");
  });
});
