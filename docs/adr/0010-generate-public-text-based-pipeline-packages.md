---
status: accepted
---

# Generate public text-based Pipeline Packages

Pipeline Authoring Skill 通过自然语言访谈或导入既有 Skill 和资料，生成一个真实、可独立建仓且可立即安装的 Package 目录；Host App 不保存另一套私有定义。Package 以公开 Schema 约束的 YAML、Markdown 和 JSON 资源描述一个或多个 Pipeline，并以 lockfile 固定外部依赖；模型提出变更，确定性工具完成初始化、校验、权限汇总、受限循环检查和测试，可执行代码必须经过开发者审查。首版从本地目录或固定 commit 与内容哈希的 Git 来源安装，Registry 与 Marketplace 后置。
