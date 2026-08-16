import { demoRun } from "./demo";
import type { RunProjection } from "./types";

export type PipelineCommand =
  | { selectNode: { nodeId: string } }
  | { requestChanges: { nodeId: string; reason: string } }
  | { approve: { nodeId: string } }
  | { advance: Record<string, never> }
  | { resetDemo: Record<string, never> };

export function applyDemoCommand(source: RunProjection, command: PipelineCommand): RunProjection {
  const run = "resetDemo" in command ? structuredClone(demoRun) : structuredClone(source);
  if ("selectNode" in command) run.selectedNodeId = command.selectNode.nodeId;
  if ("requestChanges" in command) {
    const reason = command.requestChanges.reason;
    const review = run.nodes.find((node) => node.id === "review")!;
    const implement = run.nodes.find((node) => node.id === "implement")!;
    review.status = "completed";
    review.finishedAt = "11:14";
    implement.status = "running";
    implement.attempt = 2;
    implement.startedAt = "11:14";
    implement.finishedAt = undefined;
    implement.activities = [{ id: "i2-1", title: "读取 Review feedback", detail: "正在定位异步退款幂等校验路径", status: "running", time: "11:15" }];
    run.status = "running";
    run.selectedNodeId = "implement";
    run.attention = [{ id: "attn-implement-2", nodeId: "implement", severity: "info", title: "Implement · Attempt 2 正在运行", detail: "已把 Review feedback 作为显式 Handoff 注入", time: "11:15" }];
    run.artifacts.push({ id: "review-feedback-1", title: "Review feedback · Attempt 1", mediaType: "Markdown", revision: 1, producerNodeId: "review", producerAttempt: 1, createdAt: "11:14", size: "1.3 KB", summary: reason });
    run.brief = `Review 已打回 Attempt 1：${reason}。Implement Attempt 2 正在执行，并已读取冻结的 Spec、Patch、测试报告与 Review feedback。`;
    run.eventCount += 1;
  }
  if ("approve" in command) {
    const review = run.nodes.find((node) => node.id === "review")!;
    const deploy = run.nodes.find((node) => node.id === "deploy")!;
    review.status = "completed";
    deploy.status = "running";
    deploy.attempt = 1;
    deploy.activities = [{ id: "d1", title: "准备部署清单", detail: "解析 OCM 环境能力并检查权限", status: "running", time: "11:15" }];
    run.selectedNodeId = "deploy";
    run.status = "running";
    run.attention = [];
  }
  if ("advance" in command) {
    const implement = run.nodes.find((node) => node.id === "implement")!;
    if (implement.status === "running") {
      const review = run.nodes.find((node) => node.id === "review")!;
      implement.status = "completed";
      implement.finishedAt = "11:34";
      implement.activities.push({ id: "i2-2", title: "补齐幂等校验", detail: "新增唯一约束与重复请求回放测试", status: "completed", time: "11:34" });
      review.status = "attention";
      review.attempt = 2;
      review.activities = [{ id: "r2-1", title: "复核 Attempt 2", detail: "变更满足 feedback，等待最终批准", status: "attention", time: "11:39" }];
      run.selectedNodeId = "review";
      run.status = "attention";
      run.attention = [{ id: "attn-review-2", nodeId: "review", severity: "critical", title: "Review Attempt 2 需要确认", detail: "幂等修复已完成，请复核变更", time: "11:39" }];
    } else {
      const deploy = run.nodes.find((node) => node.id === "deploy")!;
      const smoke = run.nodes.find((node) => node.id === "smoke")!;
      if (deploy.status === "running") {
        deploy.status = "completed";
        deploy.finishedAt = "11:48";
        deploy.duration = "9m";
        deploy.artifactIds = ["deployment-receipt"];
        run.artifacts.push({ id: "deployment-receipt", title: "Deployment Receipt", mediaType: "Environment reference", revision: 1, producerNodeId: "deploy", producerAttempt: 1, createdAt: "11:48", size: "Local ref", summary: "测试环境 refund-test-42 部署成功。" });
        smoke.status = "running";
        smoke.attempt = 1;
        smoke.startedAt = "11:48";
        smoke.activities = [{ id: "sm1", title: "执行核心退款路径", detail: "创建退款、重复请求回放与状态查询", status: "running", time: "11:49" }];
        run.selectedNodeId = "smoke";
        run.brief = "Review Attempt 2 已批准，测试环境部署完成。Smoke Test 正依据 Deployment Receipt 验证退款主路径与幂等回放。";
      } else if (smoke.status === "running") {
        smoke.status = "completed";
        smoke.finishedAt = "11:56";
        smoke.duration = "8m";
        smoke.artifactIds = ["smoke-report"];
        smoke.activities.push({ id: "sm2", title: "发布 Smoke Test 报告", detail: "6 passed · 0 failed", status: "completed", time: "11:56" });
        run.artifacts.push({ id: "smoke-report", title: "Smoke Test Report", mediaType: "JSON", revision: 1, producerNodeId: "smoke", producerAttempt: 1, createdAt: "11:56", size: "3.6 KB", summary: "退款主路径、重复请求回放和状态查询共 6 项全部通过。" });
        run.status = "completed";
        run.attention = [];
        run.selectedNodeId = "smoke";
        run.brief = "七阶段 Pipeline 已完成。Review feedback 在 Attempt 2 关闭，部署与 Smoke Test 均已发布可追溯 Artifact。";
      }
    }
  }
  return run;
}
