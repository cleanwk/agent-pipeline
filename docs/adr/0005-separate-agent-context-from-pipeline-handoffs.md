---
status: accepted
---

# Separate Agent context from Pipeline handoffs

Agent Runtime 继续负责 Session 内的上下文、压缩和原生恢复；Host App 不复制这套能力。跨 Node 的公共上下文通过每个 Attempt 发布的 Handoff 和可追溯 Run Brief 传递，后继节点默认读取公共上下文、直接依赖产出和用户附件，而不是无差别注入全部 Transcript，以免重现长任务上下文膨胀和信息丢失问题。

