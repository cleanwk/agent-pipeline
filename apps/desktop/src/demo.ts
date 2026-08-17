import type { AgentProbe, RunProjection } from "./types";

export const demoAgents: AgentProbe[] = [
  { id: "pi", name: "Pi", path: "/opt/homebrew/bin/pi", version: "0.80.10", state: "ready", transport: "RPC" },
  { id: "codex", name: "Codex", path: "/opt/homebrew/bin/codex", version: "0.147.0", state: "ready", transport: "ACP / App Server" },
  { id: "claude", name: "Claude Code", path: "~/.local/bin/claude", version: "2.1.232", state: "ready", transport: "ACP" },
  { id: "opencode", name: "OpenCode", path: "~/.bun/bin/opencode", version: "1.18.18", state: "ready", transport: "ACP" }
];

export const demoRun: RunProjection = {
  id: "run_20260816_0017",
  title: "支付退款能力上线",
  status: "attention",
  startedAt: "2026-08-16 10:14:32",
  elapsed: "2h 34m 18s",
  workspace: "payments-platform",
  branch: "agent/refund-capability",
  brief: "退款能力已完成需求澄清、Ticket 与技术方案。Implement Attempt 1 已发布实现与测试结果；Review 正等待确认异步退款幂等边界。",
  eventCount: 47,
  definitionPackage: "seven-stage-product-delivery",
  definitionVersion: "0.2.0",
  definitionDigest: "sha256:c54ba184…e18708",
  selectedNodeId: "review",
  nodes: [
    { id: "grill", index: 1, title: "Grill", kind: "agent", status: "completed", attempt: 1, finishedAt: "10:15", duration: "8m", runtime: "Pi", artifactIds: ["grill-record"], activities: [
      { id: "g1", title: "梳理目标与边界", detail: "确认退款入口、权限和异常路径", status: "completed", time: "10:11" },
      { id: "g2", title: "关闭关键问题", detail: "12 个问题均已回答", status: "completed", time: "10:15" }
    ]},
    { id: "ticket", index: 2, title: "Ticket", kind: "action", status: "completed", attempt: 1, finishedAt: "10:17", duration: "2m", runtime: "Local action", artifactIds: ["ticket"], activities: [
      { id: "t1", title: "生成 Ticket", detail: "本地引用 PAY-2841", status: "completed", time: "10:17" }
    ]},
    { id: "spec", index: 3, title: "Spec", kind: "agent", status: "completed", attempt: 1, finishedAt: "10:28", duration: "11m", runtime: "Pi", artifactIds: ["spec"], activities: [
      { id: "s1", title: "建立技术约束", detail: "幂等键、状态机与补偿策略", status: "completed", time: "10:21" },
      { id: "s2", title: "发布技术方案", detail: "Spec revision 2", status: "completed", time: "10:28" }
    ]},
    { id: "implement", index: 4, title: "Implement", kind: "agent", status: "completed", attempt: 1, finishedAt: "11:02", duration: "34m", runtime: "Codex", artifactIds: ["patch", "test-report"], activities: [
      { id: "i1", title: "设计实现方案", detail: "映射 Spec 到模块改动", status: "completed", time: "10:31" },
      { id: "i2", title: "编码实现", detail: "7 files changed · +284 −19", status: "completed", time: "10:52" },
      { id: "i3", title: "单元测试", detail: "42 passed · 0 failed", status: "completed", time: "11:02" }
    ]},
    { id: "review", index: 5, title: "Review", kind: "gate", status: "attention", attempt: 1, startedAt: "11:02", duration: "10m", runtime: "Codex", artifactIds: ["review"], activities: [
      { id: "r1", title: "分析代码变更", detail: "检查退款状态机与数据一致性", status: "completed", time: "11:08" },
      { id: "r2", title: "等待人工确认", detail: "异步退款缺少强制幂等校验", status: "attention", time: "11:12" }
    ]},
    { id: "deploy", index: 6, title: "Deploy", kind: "action", status: "waiting", attempt: 0, runtime: "OCM adapter", artifactIds: [], activities: [] },
    { id: "smoke", index: 7, title: "Smoke Test", kind: "agent", status: "waiting", attempt: 0, runtime: "Pi", artifactIds: [], activities: [] }
  ],
  artifacts: [
    { id: "grill-record", title: "Grill 问答记录", mediaType: "Markdown", revision: 1, producerNodeId: "grill", producerAttempt: 1, createdAt: "10:15", size: "8.2 KB", summary: "12 个关键问题及已确认答案。" },
    { id: "ticket", title: "PAY-2841", mediaType: "Ticket reference", revision: 1, producerNodeId: "ticket", producerAttempt: 1, createdAt: "10:17", size: "Local ref", summary: "退款能力开发与上线追踪。" },
    { id: "spec", title: "退款技术方案", mediaType: "Markdown", revision: 2, producerNodeId: "spec", producerAttempt: 1, createdAt: "10:28", size: "24.6 KB", summary: "退款状态机、幂等、补偿与可观测性设计。" },
    { id: "patch", title: "实现 Patch", mediaType: "Git diff", revision: 1, producerNodeId: "implement", producerAttempt: 1, createdAt: "10:52", size: "18.3 KB", summary: "7 files changed · +284 −19。" },
    { id: "test-report", title: "单元测试报告", mediaType: "JSON", revision: 1, producerNodeId: "implement", producerAttempt: 1, createdAt: "11:02", size: "4.8 KB", summary: "42 passed · 0 failed。" },
    { id: "review", title: "Code Review", mediaType: "Markdown", revision: 1, producerNodeId: "review", producerAttempt: 1, createdAt: "11:12", size: "3.1 KB", summary: "发现异步退款幂等校验缺失，等待确认。" }
  ],
  attention: [
    { id: "attn-review", nodeId: "review", severity: "critical", title: "Review 需要确认", detail: "请检查 Review 的输出并决定是否打回", time: "10:16" },
    { id: "attn-deploy", nodeId: "deploy", severity: "info", title: "部署尚未执行", detail: "Deploy 等待 Review gate", time: "10:14" }
  ]
};
