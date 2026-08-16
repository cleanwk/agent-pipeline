# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Rust、Tauri 2 与 Vue 3。首发运行环境为近三年内的 macOS，仅支持 Apple Silicon；界面运行在桌面 WebView 中。

## Users

主要用户是需要协作完成复杂、长周期研发任务的技术团队成员。团队共享和安装 Pipeline Package，每个人在自己的机器上使用已经安装并认证的 Agent Runtime 执行 Pipeline。

## Product Purpose

产品让由多个阶段组成的长时间 Agent 工作变得可观察、可干预、可恢复并可审计。成功意味着用户能清楚知道任务运行到了哪里、Agent 正在处理什么、为何等待、产生了哪些交付物，并能在上下文压缩、应用关闭或节点失败后从明确边界继续工作。

## Positioning

它不是低代码流程画布，也不重新实现 Agent 的推理和上下文系统。用户通过自然语言让模型创建和修改 Pipeline；产品以高质量 Graph、Activity、Timeline、Artifact 和恢复交互把 Agent 原本不可见、易丢失的长任务变成耐久的应用体验。

## Operating Context

首个示例 Pipeline 实现 Grill、Ticket、Spec、Implement、Code Review、Deploy 和 Smoke Test 七个研发阶段。Pipeline 可以包含分支、并行、汇合和受限反馈环；Review 打回会产生新的不可变 Attempt。节点可使用独立 worktree、Runtime、Skill、MCP 和业务平台集成。

团队通过本地目录或 Git 仓库分发 Pipeline Package。Host App 使用用户机器上已有的 Codex、Claude Code、OpenCode 和 Pi，并通过 Onboarding 与可重复运行的 Doctor 检查环境、认证和能力。

## Capabilities and Constraints

- Host App 与 Pipeline Package 分离，并交付一个独立的七阶段示例 Package。
- Host App 与七阶段示例分别发布为公开 GitHub 仓库 `cleanwk/agent-pipeline` 与 `cleanwk/agent-pipeline-example`，许可证均为 Apache-2.0。
- Package 使用公开、文本化、可版本化的协议，可以包含多个 Pipeline 和共享资源。
- Pipeline Authoring Skill 根据自然语言或现有资料生成、校验、测试并安装 Package；Graph 只用于解释、导航和审查，不提供拖拽编排。
- Pipeline Run 冻结定义、依赖与配置；运行历史、Attempt、Artifact Revision 和事件只追加、不覆盖。
- Agent Runtime 拥有 Session 内上下文、压缩和原生恢复；跨节点通过 Handoff、Run Brief、Artifact 和用户附件交接。
- 全部产品数据保存在本机，不提供产品云端、同步、遥测或崩溃上报。用户主动配置的 Agent、MCP 和业务系统可以访问网络。
- 一期不建设 Package Marketplace 或团队云；架构需允许未来云端渲染和操作 Pipeline 与选定产出。
- 默认视觉方向为“系统制图台”，批准构图为左侧 Attention、中间主 Graph、右侧固定 Inspector；节点可进一步进入 Node Focus。外观由可替换 Theme Pack 驱动。
- Vue UI、Tauri Host 与独立 Local Runner 分层；关闭窗口后运行可继续，UI 重启后重新连接。
- 本地 SQLite 同时保存可查询的当前状态与不可变 RunEvent，Artifact 内容以本地快照管理。
- ACP 提供 Agent 的统一基础能力，Runtime Adapter 保留 Codex、Claude Code、OpenCode 和 Pi 的原生增强与真实能力差异。
- Runtime 依次以 Pi RPC、Codex ACP/App Server、OpenCode ACP、Claude Code ACP Adapter 形成真实实现；第二个 Runtime 接入后才冻结公共 Interface。
- Agent CLI 持有自己的认证；业务集成 Secret 存入 macOS Keychain，Package、SQLite、日志、Artifact 与导出只保存引用。
- Host App 自身不包含遥测；由其启动的 Runtime 子进程默认注入该 Runtime 官方提供的隐私/自动分享关闭选项，但不修改用户的全局 Shell 配置，也不替第三方模型服务承诺离线。
- 一期内置 System、Draft Light、Night Ops 与 Warm Paper 主题入口；System 跟随 macOS 自动选择明暗模式。

## Brand Commitments

产品名称为 **Agent Pipeline**。首发界面以中文为主，Agent、Workflow、Pipeline、Runtime 等行业术语保留英文；代码应为后续国际化保留稳定的消息边界。

明确拒绝 Dify 式拖拽画布、传统低代码编排器和普通 Dashboard 卡片阵列。Craft Agents 与 Pi 可作为产品完成度参考，但不构成需要复制的视觉品牌。

## Evidence on Hand

当前仓库只有已确认的领域词汇和架构决定，没有现有 UI、品牌资产、用户数据、客户证明或性能指标；未来设计不得虚构这些内容。

## Product Principles

- 模型负责配置劳动，人通过自然语言表达意图并控制关键决定。
- Graph 和交互是核心产品机制，不是后台页面上的装饰。
- 观察 Agent，不干涉或复制 Agent 已经擅长的推理与上下文管理。
- 所有恢复、重试、审批和交付都以可追溯、不可覆盖的运行事实为依据。
- 核心保持简单稳定，Package、Runtime、Skill、MCP 和 Renderer 通过明确边界扩展。

## Delivery Acceptance

首个可验收版本必须完成真实纵向链路：Onboarding/Doctor 发现至少一个 Agent；安装七阶段示例 Package；运行 Grill、Ticket、Spec；在独立 worktree 实现代码；Review 打回并生成新 Attempt；通过本地参考 Adapter 执行 Deploy 与 Smoke Test；浏览 Run Brief、Activity、Artifact 和 Deliverables；关闭窗口后重新连接运行；通过 Pipeline Authoring Skill 生成并安装一个简单 Package。

交付物不是开发服务器或未打包源码。必须构建可运行的 macOS `.app`，安装到本机并从安装位置启动；随后用真实 UI 操作依次验证 Onboarding、Package 安装、Run 创建、Graph 更新、Node Focus 下钻、Attention/Review、反馈环、Artifact 汇总、主题切换和窗口重连。每个关键状态保存截图，并依据批准的 Run Graph 构图与 Theme 语义完成视觉审查。

完成本机验收后，通过版本 Tag 发布 Apple Silicon DMG 到公开 GitHub Release，并自动更新现有 `cleanwk/homebrew-tap` 中的 `Casks/agent-pipeline.rb`。用户安装命令为 `brew install --cask cleanwk/tap/agent-pipeline`。公开 Release 还必须通过 Developer ID 签名、Apple notarization、Cask audit/style、全新安装、启动、升级和卸载验证；Tauri updater 签名不能替代 macOS 分发签名与公证。
