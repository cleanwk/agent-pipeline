---
status: accepted
---

# Persist current state and an immutable event log

Local Runner 在同一 SQLite 事务中更新可高效查询的当前状态并追加不可变 RunEvent，而不是选择纯 Event Sourcing 或只有可覆盖状态表。Graph、Attention 和普通列表读取当前投影，Timeline、审计、UI 重连与未来云端同步可读取事件；原始 Runtime 事件可以留存，但公开投影只依赖带版本的标准事件信封。

