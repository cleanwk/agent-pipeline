---
status: accepted
---

# Run work independently from the desktop window

Pipeline 执行生命周期不依附于桌面窗口：关闭窗口时本地 Runner 可继续工作，重新打开后 Host App 重连并恢复可视化；明确退出时由用户选择继续后台运行或停止。不同 Agent Runtime 的恢复能力必须如实分级展示，不能用统一的“断点续传”承诺掩盖只能重试或人工恢复的情况。

