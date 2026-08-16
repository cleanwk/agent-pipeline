---
status: accepted
---

# Keep the Pipeline execution kernel small

Runner 核心只拥有稳定 Graph、Node 状态机、不可变 Attempt、依赖调度和事件发布；Node 的执行语义只分为 `agent`、`action` 与 `gate`，受控进程和外部连接都是 Action，子 Pipeline 通过 Graph 组合，而不是继续增加并列 Executor 类型。运行中的正式 Graph 保持冻结，Agent 可自由发布 Activity；反馈环必须声明条件、最大次数和耗尽行为，失败、重试、超时与人工处理由 Package 默认值和 Workspace Policy 合成。首期只为已有多个真实 Adapter 的 Runtime，以及有本地目录和 Git 两种来源的 Package resolution 建立扩展 seam；SQLite、Artifact、Connector、Renderer、云同步和 Lease 先保持具体 Module，等第二个真实实现迫使调用方产生分支后再提取 Interface。
