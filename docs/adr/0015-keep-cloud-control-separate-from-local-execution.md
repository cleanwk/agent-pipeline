---
status: accepted
---

# Keep cloud control separate from local execution

未来云端是可选的 Control/Projection Plane，本地 Runner 仍是默认 Execution Plane。只有用户明确选择的 Pipeline Snapshot、状态事件与 Artifact 可以同步；代码、完整 Transcript、凭据和原始终端日志默认留在本机。云端操作形成需经本地能力与权限校验的 Command，未来 Remote Runner 复用相同 Pipeline 协议，因此本地模式永远不依赖云 API。

