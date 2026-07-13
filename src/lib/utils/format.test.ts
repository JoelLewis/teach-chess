import { describe, expect, it } from "vitest";
import { humanizeLabel } from "./format";

describe("humanizeLabel", () => {
  it("turns camelCase theme names into title case", () => {
    expect(humanizeLabel("discoveredAttack")).toBe("Discovered Attack");
  });
});
