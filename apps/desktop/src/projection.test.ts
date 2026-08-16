import { describe, expect, it } from "vitest";
import { demoRun } from "./demo";
import { applyDemoCommand } from "./projection";

describe("RunProjection command seam", () => {
  it("projects Review feedback as a new Implement Attempt without erasing history", () => {
    const before = structuredClone(demoRun);
    const after = applyDemoCommand(before, {
      requestChanges: { nodeId: "review", reason: "补齐幂等校验" }
    });

    expect(after.nodes.find((node) => node.id === "implement")?.attempt).toBe(2);
    expect(after.selectedNodeId).toBe("implement");
    expect(after.artifacts[after.artifacts.length - 1]?.title).toBe("Review feedback · Attempt 1");
    expect(demoRun.nodes.find((node) => node.id === "implement")?.attempt).toBe(1);
  });
});
