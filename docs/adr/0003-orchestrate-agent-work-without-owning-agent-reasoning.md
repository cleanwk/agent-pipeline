---
status: accepted
---

# Orchestrate Agent work without owning Agent reasoning

Host App 不重新实现 Agent 已经擅长的长任务推理、对话上下文管理或压缩机制，也不为了展示进度而把真实工作过度拆碎。它专注补足现有 Agent 的缺口：以优秀的 Graph 交互呈现执行过程，以持久化运行事实和明确交接产物抵抗上下文丢失，并为中断、人工干预和恢复提供稳定边界。Agent 仍拥有其 Session 内部上下文，Host App 只持有跨节点编排所必需的公共状态。
