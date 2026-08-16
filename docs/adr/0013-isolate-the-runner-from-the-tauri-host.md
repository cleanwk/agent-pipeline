---
status: accepted
---

# Isolate the Runner from the Tauri Host

桌面应用分为 Vue UI、受控 Tauri Host 与独立 Local Runner：UI 表达用户意图和渲染投影，Host 提供最小桌面与 IPC 边界，Runner 持有 Pipeline 状态机、Agent/PTY 进程、Artifact 与恢复。关闭窗口不会隐式结束任务，重新打开 UI 通过本地 Socket 重连；这一边界也允许未来把同一 Runner 协议部署到远程执行环境。

