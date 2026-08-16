---
status: accepted
---

# Model feedback as new immutable Attempts

Pipeline 定义允许 Review 打回 Implement 一类反馈环，但运行历史只追加、不倒退：每次沿反馈边重新进入 Node 都创建新的 Attempt，旧 Session、日志和 Artifact 不被覆盖。Graph 可以折叠显示逻辑 Node 和反馈边，也可以展开每一轮 Attempt，从而同时满足直观观察、审计和恢复。

