import { describe, expect, it } from "vitest";
import { checkForAppUpdate, mapPackageInspection } from "./api";

describe("package inspection wire contract", () => {
  it("preserves camelCase decision and bounded-loop metadata emitted by Rust", () => {
    const definition = mapPackageInspection({
      root: "/local/packages/release/1.0.0",
      digest: "sha256:abc",
      manifest: {
        protocol: "agent-pipeline.dev/v1alpha1",
        metadata: { name: "release", displayName: "Release", version: "1.0.0" },
        pipelines: [{ path: "pipelines/release.yaml" }]
      },
      pipelines: [{
        id: "release",
        context: {},
        nodes: [
          { id: "implement", type: "agent", outputs: {} },
          { id: "review", type: "gate", decisionSchema: { outcomes: ["approved", "changes_requested"] }, outputs: {} }
        ],
        edges: [{
          from: "review",
          to: "implement",
          when: "changes_requested",
          loop: { maxIterations: 3, onExhausted: "attention" }
        }]
      }]
    });

    expect(definition.nodes[1]?.policy).toBe("Decision: approved | changes_requested");
    expect(definition.edges[0]?.loop).toEqual({ maxIterations: 3, onExhausted: "attention" });
    expect(definition.digest).toBe("sha256:abc");
    expect(definition.source).toContain("/local/packages/");
  });
});

describe("app updater browser fallback", () => {
  it("does not contact or offer native updates in browser preview", async () => {
    expect(await checkForAppUpdate()).toBeNull();
  });
});
