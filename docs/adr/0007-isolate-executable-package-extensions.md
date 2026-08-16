---
status: accepted
---

# Isolate executable Package extensions

Pipeline Package 的核心协议保持声明式；Skill、MCP、Hook、脚本和 Adapter 分别声明来源与权限，可执行扩展只能通过隔离边界运行，不进入 Host App 或 WebView 进程。安装时必须展示其文件、命令、网络、环境和凭据能力，并由 Package 默认值、Workspace Policy 与本次 Run 选择共同形成最严格的有效策略。
