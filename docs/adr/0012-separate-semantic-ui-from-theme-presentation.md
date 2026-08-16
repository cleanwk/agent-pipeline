---
status: accepted
---

# Separate semantic UI from Theme presentation

Host App 的 Graph、Attention、Review、Artifact 和运行状态使用稳定的语义模型与 Token，不在组件或 Pipeline Package 中写死颜色和表面样式。Theme Pack 可以替换明暗、色彩、字体角色、材质、线条、圆角、阴影和动效强度，但不得改变状态含义、隐藏证据或向 WebView 注入任意代码；同一条 Pipeline Run 因而可以在亮色、暗色和温暖纸张等主题下保持一致操作与可访问性。
