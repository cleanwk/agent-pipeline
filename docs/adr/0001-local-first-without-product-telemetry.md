---
status: accepted
---

# Local-first without product telemetry

首个版本的 Host App 将全部 Pipeline、Pipeline Run、配置、Artifact 和审计数据保存在用户本机，不提供产品云端、同步或遥测。用户明确启动的外部 Agent、MCP 或业务系统集成仍可按各自协议访问网络；这是用户配置的执行能力，不属于 Host App 数据回传。架构需保留未来增加团队云端的边界，但不得以隐藏上传或云端依赖换取这种扩展性。

