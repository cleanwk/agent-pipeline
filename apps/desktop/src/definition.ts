import type { PipelineDefinition } from "./types";

export const demoDefinition: PipelineDefinition = {
  protocol: "agent-pipeline.dev/v1alpha1",
  packageName: "seven-stage-product-delivery",
  displayName: "七阶段研发交付",
  version: "0.2.0",
  source: "~/Library/Application Support/dev.agentpipeline.desktop/packages/seven-stage-product-delivery/0.2.0",
  digest: "sha256:c54ba184fbdd7530db90e79a0ee9cb7f8c72c6d94612c448e0f0206419e18708",
  entrypoint: "pipelines/product-delivery.yaml",
  installedAt: "in the immutable local package store",
  contextPolicy: "Run Brief + direct dependency Artifacts + manual attachments",
  edges: [
    { from: "grill", to: "ticket" }, { from: "ticket", to: "spec" }, { from: "spec", to: "implement" },
    { from: "implement", to: "review" }, { from: "review", to: "deploy", when: "approved" },
    { from: "review", to: "implement", when: "changes_requested", loop: { maxIterations: 3, onExhausted: "attention" } },
    { from: "deploy", to: "smoke" }
  ],
  nodes: [
    {
      nodeId: "grill",
      summary: "通过追问关闭目标、边界、依赖与验收标准中的关键未知项。",
      promptRef: "prompts/grill.md",
      skillIds: ["grill-with-docs"],
      mcpServers: [],
      capabilities: ["artifact.publish", "activity.publish"],
      inputs: ["task", "attachments?"],
      outputs: ["grill-record · Markdown"],
      context: "Task + user attachments",
      policy: "Manual answers allowed · no automatic timeout",
      sandbox: "Workspace read-only"
    },
    {
      nodeId: "ticket",
      summary: "把澄清后的任务同步为团队可追踪的需求记录。",
      action: "ticket.create",
      skillIds: ["to-ticket"],
      mcpServers: [{ name: "Requirement Hub", transport: "http", tools: ["ticket.create", "ticket.link"], permission: "write" }],
      capabilities: ["ticket.write", "artifact.publish"],
      inputs: ["grill-record"],
      outputs: ["ticket-reference · Reference"],
      context: "Grill record + Run Brief",
      policy: "Ask before writing external system",
      sandbox: "No filesystem access"
    },
    {
      nodeId: "spec",
      summary: "基于冻结需求形成可实现、可评审的技术方案与约束。",
      promptRef: "prompts/spec.md",
      skillIds: ["to-spec"],
      mcpServers: [{ name: "Code Search", transport: "stdio", tools: ["search", "symbols"], permission: "read" }],
      capabilities: ["workspace.read", "artifact.publish", "activity.publish"],
      inputs: ["grill-record", "ticket-reference"],
      outputs: ["technical-spec · Markdown"],
      context: "Direct dependencies + repository map",
      policy: "Required output contract",
      sandbox: "Existing repo · read-only"
    },
    {
      nodeId: "implement",
      summary: "在隔离 Worktree 中实现 Spec，运行测试并发布 Patch 与测试证据。",
      promptRef: "prompts/implement.md",
      skillIds: ["implement", "tdd"],
      mcpServers: [{ name: "Code Search", transport: "stdio", tools: ["search", "symbols"], permission: "read" }],
      capabilities: ["workspace.read", "workspace.write", "process.git", "process.test", "artifact.publish"],
      inputs: ["technical-spec", "review-report?", "patch?"],
      outputs: ["patch · Git diff", "test-report · JSON"],
      context: "Spec + latest Review feedback + prior Attempt artifacts",
      policy: "Retry 2 · timeout 45m",
      sandbox: "New worktree per Attempt"
    },
    {
      nodeId: "review",
      summary: "检查代码、测试和风险；批准或携带结构化反馈返回 Implement。",
      promptRef: "prompts/review.md",
      skillIds: ["code-review"],
      mcpServers: [],
      capabilities: ["workspace.read", "process.git", "artifact.publish", "approval.request"],
      inputs: ["technical-spec", "patch", "test-report"],
      outputs: ["review-report · Markdown", "decision · approved | changes_requested"],
      context: "Frozen Spec + current Attempt evidence",
      policy: "Feedback loop max_iterations: 3",
      sandbox: "Implementation worktree · read-only"
    },
    {
      nodeId: "deploy",
      summary: "经高风险审批后，把当前构建部署到指定测试环境。",
      action: "environment.deploy",
      skillIds: ["deploy"],
      mcpServers: [{ name: "OCM Test Environment", transport: "http", tools: ["environment.inspect", "deployment.create", "deployment.status"], permission: "deploy" }],
      capabilities: ["environment.deploy", "artifact.publish", "approval.request"],
      inputs: ["patch", "test-report", "review-report"],
      outputs: ["deployment-receipt · Reference"],
      context: "Approved Attempt + environment snapshot",
      policy: "Human approval · idempotency key required",
      sandbox: "No source write"
    },
    {
      nodeId: "smoke",
      summary: "在部署目标上执行主链路、回归与幂等验证并生成报告。",
      promptRef: "prompts/smoke.md",
      skillIds: ["smoke-test"],
      mcpServers: [{ name: "OCM Test Environment", transport: "http", tools: ["environment.read", "logs.query"], permission: "read" }],
      capabilities: ["environment.read", "process.test", "artifact.publish", "activity.publish"],
      inputs: ["deployment-receipt", "technical-spec"],
      outputs: ["smoke-report · JSON"],
      context: "Deployment receipt + acceptance criteria",
      policy: "Failure creates Attention",
      sandbox: "Ephemeral test process"
    }
  ]
};
