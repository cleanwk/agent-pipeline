# Agent Pipeline 的可扩展内核：以 Pi Agent 为主轴的架构研究

> 调研日期：2026-08-16  
> 范围：只使用官方文档、官方仓库、规范与作者一方文章。文中“事实”可由紧邻链接验证；“建议/推断”是面向 Agent Pipeline 的设计判断。

## 结论先行

Agent Pipeline 不应成为另一个 Agent，也不应成为一个通用低代码引擎。它的最小职责是：**冻结一条长任务的控制图，把非确定性的 Agent 工作放进可追溯 Attempt，持续投影运行进度，保存交付物，并在人工介入或进程中断后从明确边界继续。**

Pi 最值得学习的是“适配工作流，而不是让工作流适配内核”：它把 agent loop、Session、资源发现、UI 分层，默认只提供很少的工具，把扩展放在窄而明确的入口。Pi 官方把自己定义为 minimal harness，默认四个工具，刻意不内置 sub-agent、plan mode 等能力；扩展由 Extensions、Skills、Prompt Templates、Themes 与 Packages 提供。[Pi README / Philosophy](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/README.md#philosophy)

对应到 Agent Pipeline：

1. Runtime 拥有模型上下文、Session、compaction 与原生恢复；Pipeline 不复制这些能力。
2. Runner 拥有 Graph traversal、Attempt、审批、重试、超时、Artifact 与恢复边界；Package 不能接管调度器。
3. UI 只消费结构化事实并形成 Graph/Activity/Attention/Artifact 投影；不要求模型为 UI 生成布局。
4. 首期真正需要的扩展缝只有 `RuntimeAdapter` 与声明式 Package 解析。存储、Renderer、云同步、Connector、Executor SDK 都不应提前抽象成框架。
5. 直接复用 ACP Rust SDK、各 Agent 官方接口、Vue Flow/ELK 一类表现库；不要直接依赖 pre-1.0 的 acpx Flow runtime，也不要引入 Temporal 级别的服务型基础设施。

## 1. 第一性原理：产品缺的不是 Agent 能力

长任务失败通常不是因为模型不会把一个大任务做完，而是应用层缺少四种耐久事实：

- **控制事实**：当前可运行哪个 Node、走过哪条 edge、循环了几次、在等谁。
- **执行事实**：哪个 Runtime Session 执行了哪个 Attempt，是否仍可 resume。
- **观察事实**：Agent 正在计划、调用工具、改文件还是等待输入。
- **交付事实**：哪些结果已经正式发布、被哪个后继读取、哪个 revision 被取代。

因此最重要的边界是：

```text
Agent Runtime                         Agent Pipeline
--------------                        ------------------------------
reasoning / model loop                graph cursor / scheduling
session context                       NodeRun / Attempt history
native compaction                     approval / retry / loop ceiling
tool execution detail   --events-->   Activity projection
native resume handle                  recovery capability + checkpoint
working files            --publish--> immutable ArtifactRevision
```

这不是语义洁癖。若 Pipeline 同时接管模型上下文，必须追赶 Codex、Claude、OpenCode、Pi 各自的 signed reasoning、tool message、compaction 和恢复格式；最终既不可靠，也会污染 Agent 本来已经有效的工作方式。

## 2. Pi Agent：当前身份与设计主轴

### 2.1 官方身份已经迁移

Pi 当前官方仓库是 [`earendil-works/pi`](https://github.com/earendil-works/pi)，npm scope 是 `@earendil-works/*`；`@earendil-works/pi-coding-agent` 从 0.74.0 开始发布。旧 `@mariozechner/*` 在 0.73.1 后弃用但未下架，CLI 仍叫 `pi`，既有配置与 Session 路径不变。[官方迁移公告](https://pi.dev/news/2026/5/7/pi-has-a-new-home)

实现与文档不应再把 `badlogic/pi-mono` 或 `@mariozechner/pi-coding-agent` 写成当前安装目标。

### 2.2 极简不是“功能少”，而是核心不替用户决定工作流

Pi 的作者将原则概括为“如果我不需要，就不构建”，并认为精确控制进入模型的 Context 与能检查每次交互非常关键。[作者设计复盘](https://mariozechner.at/posts/2025-11-30-pi-coding-agent/) Pi 当前 README 进一步明确：核心跳过 sub-agent、plan mode 等功能，用户可让 Pi 自己构建或安装 Package。[Pi README](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/README.md)

Pi 的默认 agent 只有 `read/write/edit/bash` 四个工具。其价值不在“四”这个数字，而在一个判断标准：**模型已经能通过组合基础能力完成的工作，不进入内核。**

对 Agent Pipeline 的推断：

- 不内置“研发需求”“Jira”“OCM”“代码评审”等业务 Node 类型；它们属于 Package。
- 不给每种 Agent 的每种 tool call 建核心表；归一化为 Activity，同时保留 vendor raw payload。
- 不内置 Agent Todo/子任务调度。Node 下的 Activity Tree 是 Runtime 事件的投影，不是第二套 Scheduler。
- 不做拖拽式 Graph Authoring。自然语言修改声明式 Graph，UI 负责预览、解释和审查 Change Proposal。

### 2.3 分层比类层级更重要

Pi monorepo 把多模型 API、agent loop、coding session/资源、TUI 分开；`AgentSession` 管生命周期、消息、模型状态、compaction 与事件流，替换 Session 的 new/resume/fork/import 则上移到 `AgentSessionRuntime`。[Pi SDK](https://pi.dev/docs/latest/sdk)

这里值得复制的原则是**所有权单一**：

- `AgentSession` 不负责 Workflow。
- `AgentSessionRuntime` 负责 cwd-bound 服务在 Session 替换时重建。
- UI 订阅事件，不成为 Session 状态来源。

Agent Pipeline 应采取同样方式：`RunEngine` 不知道 Codex Thread 或 Pi JSONL 细节；`RuntimeAdapter` 不决定 Graph 下一条边；Vue 不直接驱动进程状态机。

### 2.4 ResourceLoader 是一个“深模块”

Pi 的 `ResourceLoader` 用一个边界提供 extensions、skills、prompts、themes 与 AGENTS context；传入自定义 Loader 后，默认的 cwd/agentDir 发现规则不再决定资源来源。[ResourceLoader / SDK](https://pi.dev/docs/latest/sdk#resourceloader)

它扩展性好的原因不是资源类型多，而是调用者只依赖“解析后的有效资源”，不用知道资源来自全局目录、项目目录还是 Package。Skills 还采用渐进披露：启动时只暴露名称与描述，命中后模型再读取完整 `SKILL.md`，避免把所有知识预先塞入 Context。[Pi Skills](https://pi.dev/docs/latest/skills)

对 Agent Pipeline 的迁移方式应更窄：

```text
Package source (local/git)
        ↓ resolve + validate + lock
ResolvedPackage
        ├─ GraphDefinition
        ├─ skills/prompts/schemas/assets
        ├─ runtime requirements
        ├─ capability/permission declarations
        └─ content hashes
```

不要创建一个可以运行任意代码的万能 `ResourceLoader`。Package 解析只产出冻结数据；代码型扩展若以后确有需要，必须在隔离子进程和明确权限下运行。

### 2.5 Extension 的优点与不能照搬的信任模型

Pi Extension 能注册工具、命令、provider、UI，订阅 Session/Turn/Tool 生命周期，并定制 compaction；有状态 Extension 应从持久 Session branch 重建，而不是只依赖内存。[Pi Extensions](https://pi.dev/docs/latest/extensions)

但 Pi 明确声明其进程默认拥有启动用户的文件、进程、网络和凭据权限，强边界应通过容器或沙箱获得。[Pi 权限与容器边界](https://github.com/earendil-works/pi#permissions--containerization) Pi Package 也提醒安装者：Extension 是任意代码并拥有完整系统访问。[Pi Packages](https://pi.dev/docs/latest/packages)

因此：**学习 Pi 的 Extension seam，不复制 Pi 的 Extension trust model。** 团队一键安装的 Pipeline Package 默认只能包含声明式 Graph、Schema、Prompt、Skill、Theme metadata；Hook/Adapter/脚本必须显式声明、审查并隔离执行，不能注入 Tauri Host 或 WebView。

### 2.6 Session persistence 与 compaction：历史不覆盖，上下文由 Agent 管

Pi Session 使用按 cwd 组织的 append-only JSONL tree；entry 以稳定 `id/parentId` 连接，支持 resume、tree navigation、fork 与 clone。[Pi Session Format](https://pi.dev/docs/latest/session-format) RPC 的 `get_entries(since: id)` 可以用 entry id 作为增量游标，适合客户端重连后补投影。[Pi RPC](https://pi.dev/docs/latest/rpc)

Pi compaction 会保留最近内容，为旧跨度产生 summary 并追加 `CompactionEntry`；原历史不被删除。分支还有独立 Branch Summary，Extension 可替换 summarization 行为。[Pi Compaction](https://pi.dev/docs/latest/compaction)

这直接支持当前产品边界：

- Pipeline 保存 Runtime 的 `session_id/session_file` 与最后观察游标，不复制对话为自己的“标准上下文”。
- Resume 优先调用 Runtime 原生能力；不支持时，从冻结输入、Handoff 和 Artifact 创建新 Attempt。
- Run Brief 不是 Session compaction。它是跨 Node 的显式交接投影，不能冒充完整对话。
- 完整 Transcript 可以索引和浏览，但不默认注入每个后继 Node。

### 2.7 Pi 接入：Rust Runner 首选 RPC 子进程

Pi 对 Node/TypeScript 宿主推荐 SDK，对其他语言提供严格 JSONL 的 RPC；RPC 覆盖 prompt、steer、follow-up、abort、new/switch/fork/clone、compact、state/messages/entries 与完整事件流。[Pi SDK](https://pi.dev/docs/latest/sdk) [Pi RPC](https://pi.dev/docs/latest/rpc)

对 Rust Runner，最合适的是启动用户已经安装并认证的 Pi RPC 子进程：

- 不嵌入 Node runtime，也不接管用户凭据。
- 直接消费结构化事件，不解析 TUI/PTY。
- 保存 Session handle 与 entry cursor，实现 UI/Runner 重连后的增量恢复。
- 以 RPC 的 `agent_settled` 判定完整工作已结束，而不是看到一次 `agent_end` 就提前完成 Node。

PTY 只应是明确标注能力降级的兼容后备。

Pi 目前还包含版本检查与可关闭的安装/更新 telemetry；官方提供 `PI_SKIP_VERSION_CHECK`、`PI_TELEMETRY=0` 和 offline 设置。[Pi Telemetry and update checks](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/README.md#telemetry-and-update-checks) Agent Pipeline 的“自身无遥测”承诺不能替第三方 CLI 作虚假保证；Onboarding 应展示该差异，并允许用户选择给子进程注入官方 opt-out 设置。

## 3. 生态对照：复用什么，不复用什么

### 3.1 ACP 与官方 Rust SDK：公共底座，不是 Pipeline 协议

ACP v1 通过 JSON-RPC 定义 initialize、session/new/load/resume、prompt、update、permission 与 cancellation；能力在初始化时协商，Session load/resume 等能力均不能假定存在。[ACP v1 Overview](https://agentclientprotocol.com/protocol/v1/overview) [ACP Session Setup](https://agentclientprotocol.com/protocol/v1/session-setup) ACP 官方明确区分 wire `protocolVersion` 与 SDK/package 版本，兼容性应以前者和 capabilities 判断。[ACP Protocol Repository](https://github.com/agentclientprotocol/agent-client-protocol)

官方 Rust SDK 已提供 client/agent/proxy/conductor；protocol v2 与 MCP-over-ACP 仍有 unstable feature，不适合作为 MVP 必需条件。[ACP Rust SDK](https://github.com/agentclientprotocol/rust-sdk)

建议：

- 直接采用官方 Rust SDK 实现 ACP client。
- 每个 Session 固化 `runtime_session_id + negotiated_capabilities + adapter_version`。
- ACP 只承载 Agent ↔ Client；Graph、Attempt、Artifact、Attention 均使用我们自己的小协议。
- 首期锁定稳定 v1；v2/MCP-over-ACP 放在实验 feature 后。

### 3.2 acpx Flows：最接近问题，但不作为稳定内核

acpx 是 headless ACP client，已有 persistent session、NDJSON event envelope、queue/cancel、实验性 Flow 与 worktree isolation；官方同时明确标为 alpha，接口可能变化。[acpx README](https://github.com/openclaw/acpx)

其 Flow 设计把 runtime 的 graph/liveness/persistence/routing 与 Agent 的 reasoning/judgment/code changes 分开；Graph 是 plain tagged objects，validation 与 semantic validation 分层，Viewer 是只读 projection。官方 non-goals 包括不修改 ACP、不造大型 DSL、不把 GitHub/PR 业务写进 core、不重复 Transcript、不做 visual builder。[acpx Flow Guide](https://github.com/openclaw/acpx/blob/main/docs/flows.md) [acpx Flow Architecture](https://github.com/openclaw/acpx/blob/main/docs/2026-03-25-acpx-flows-architecture.md)

建议复用它的测试思想和数据形状，不直接嵌入 runtime：

- 用 plain versioned data 表达 Graph。
- Runner 独占 traversal、retry、timeout 与 liveness。
- 建立 branch、same-session multi-turn、worktree、checkpoint、immutable replay 的回归场景。
- 不照搬它的五类 Node；首期 `agent | action | gate` 三种执行语义足够。

### 3.3 Codex App Server：第一个 native enhancement

Codex app-server 是官方 rich client 接口，使用 JSON-RPC/JSONL stdio，WebSocket 仍标 experimental/unsupported。其模型为 `Thread → Turn → Item`，流式事件有稳定 item 生命周期、Diff、工具活动、审批、用户输入和 token usage；还可根据本机 Codex 版本生成对应 JSON Schema。[Codex App Server](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md)

建议：ACP 提供公共能力，Codex native adapter 额外提供更高质量的 Activity Tree、Diff、Attention、fork/resume。Adapter 把 vendor event 映射为平台事件，但保存原始 payload 与 schema fingerprint；`item/completed` 是 Activity 结果，stream delta 是可丢的观察数据，不是 Artifact。

### 3.4 OpenCode：ACP 基线，HTTP Server 后加增强

OpenCode 官方直接提供 `opencode acp`（stdin/stdout ND-JSON），也提供带 OpenAPI 3.1 的 headless HTTP server；TUI 本身就是 server client。[OpenCode CLI](https://dev.opencode.ai/docs/cli/) [OpenCode Server](https://dev.opencode.ai/docs/server/)

首期使用 ACP。todo、child session、diff、fork 等 HTTP 能力以后作为同一个 OpenCode adapter 的 enhancement，不能在核心中伪装成第二种 Runtime。

### 3.5 Craft Agents OSS：学习 Agent-native UX 与 client/server 边界

Craft Agents 把 Workspace、Session、Source、Skill、permission mode、theme 和 background task 分开；Electron UI 可作为 thin client 连接 headless server。其产品入口强调自然语言添加 Source、迁移/创建 Skill，而不是配置表单或拖拽 Builder。[Craft Agents OSS](https://github.com/craft-ai-agents/craft-agents-oss)

值得借鉴：Attention/Inbox、自然语言配置、UI/Runner 分离、Theme 级联、三档权限 UX。不要复用它的 Session JSONL 作为 Pipeline 存储，也不要让模糊的 `Source` 吞掉 MCP、Connector、Secret Reference 与权限边界。

## 4. 成熟编排系统给出的最低限度

### Temporal：借原则，不引擎

Temporal 用 append-only Event History 重建 Workflow Execution；Workflow code 必须 deterministic，外部 API 等副作用放进 Activity，Activity 必须可幂等重试或明确不可重试。[Temporal Architecture](https://github.com/temporalio/temporal/blob/main/docs/architecture/README.md)

适用推断：

- `GraphDefinition ≠ PipelineRun`；启动时冻结 Run Snapshot。
- Scheduler 必须确定性，Agent/Shell/MCP 是不可确定 effect。
- 不通过 replay 重演 Agent；恢复应 native resume 或创建新 Attempt。
- 事件日志是事实，Run Brief 是展示/上下文投影，后者不能控制调度。

不应引入 Temporal Server/Worker 与确定性 replay 编程模型；对本地单机 MVP，SQLite 当前状态 + append-only RunEvent 已足够。

### LangGraph：借 checkpoint、interrupt 与 loop 约束

LangGraph 在每个 step 保存 checkpoint，支持 history、replay/fork 与 HITL；人工 `update_state` 会创建新 checkpoint而不修改旧值。[LangGraph Persistence](https://docs.langchain.com/oss/python/langgraph/persistence) Interrupt 会保存状态等待，恢复时实际上从 Node 开头重跑，因此 interrupt 前副作用必须幂等。[LangGraph Interrupts](https://docs.langchain.com/oss/python/langgraph/interrupts)

适用推断：

- Review → Implement 是反馈 edge，每次经过都创建新 Attempt。
- 每个环必须有 condition、max iterations 与 exhausted outcome。
- Connector/Action 要带 idempotency key；恢复不能假定从某行代码继续。
- Checkpoint 只存 graph cursor、Node/Attempt 状态、branch/gate decision、Artifact refs 与 runtime resume handle。
- Agent Pipeline 比 LangGraph 更严格：暂停数天后仍使用冻结的 Graph Snapshot，而非最新定义。

### Argo 与 Prefect：Artifact、暂停与资源并发

Argo 原生区分 DAG/Steps、input/output artifacts、loops、suspend/resume、retry 和 mutex/semaphore。[Argo Overview](https://argo-workflows.readthedocs.io/en/latest/) [Argo Synchronization](https://argo-workflows.readthedocs.io/en/latest/synchronization/) Prefect 把 Artifact 定义为供人消费的持久输出并保留 lineage/version history，而 Event 有小型 envelope 与时序关系。[Prefect Events](https://docs.prefect.io/v3/concepts/events) [Prefect Artifacts](https://docs.prefect.io/v3/concepts/artifacts)

适用推断：Artifact 与 checkpoint 必须分离；Attention 应是可暂停、可恢复的结构化输入请求；首期资源控制只实现全局/Workspace/Run 并发上限与 worktree lease，等真实环境冲突出现再扩展通用 Lease 语法。

## 5. 推荐的最小核心

核心只保留九个概念：

1. **GraphDefinition**：plain versioned data；Node、Edge、condition、loop ceiling、输入输出 contract。
2. **RunSnapshot**：冻结 Package、Graph、resolved dependencies、policy 与 hashes。
3. **NodeRun / Attempt**：逻辑节点与不可变执行尝试分开；retry/feedback 永不覆盖。
4. **RuntimeAdapter**：ACP baseline + 可选 vendor-native enhancement。
5. **RunEvent**：append-only observable fact。
6. **Projection Tables**：与 RunEvent 在同一 SQLite transaction 更新，服务高频 UI 查询。
7. **ArtifactRevision**：typed、immutable、lineage-aware blob/metadata。
8. **AttentionRequest**：结构化 schema、evidence refs、allowed actions、resolution。
9. **OrchestrationCheckpoint**：只保存继续调度所需的 cursor/decision/ref。

执行类别不要设计类层级，使用 sealed enum：

```rust
enum NodeExecution {
    Agent(AgentSpec),   // Runtime Session
    Action(ActionSpec), // 本地 process 或已解析 connector capability
    Gate(GateSpec),     // approval / typed human input
}
```

子 Pipeline 是 Graph 组合，不需要第四个插件执行框架；确定性 compute/decision 先用内置 Action/Edge condition 表达。

## 6. 真正需要的 seams/interfaces

### 6.1 RuntimeAdapter：现在就需要

已有 Codex、Claude、OpenCode、Pi 四个实现，已满足“第二个 adapter”原则。接口只覆盖跨 Runtime 共同且对编排必要的行为：

```rust
trait RuntimeAdapter {
    async fn probe(&self) -> RuntimeReport;
    async fn start(&self, spec: StartSession) -> SessionHandle;
    async fn resume(&self, handle: &SessionHandle) -> ResumeResult;
    async fn prompt(&self, handle: &SessionHandle, input: PromptInput) -> TurnHandle;
    async fn interrupt(&self, handle: &SessionHandle) -> InterruptResult;
    fn events(&self, handle: &SessionHandle) -> RuntimeEventStream;
}
```

关键约束：

- `RuntimeReport` 暴露 capability，不承诺最低公分母之外的同等能力。
- `SessionHandle` 是 opaque vendor data + adapter version，不由 core 解码。
- `RuntimeEvent` 是稳定 envelope + common projection + raw vendor payload。
- native enhancement 仍在同一个 adapter 内，不产生 `CodexAcpRuntime`/`CodexNativeRuntime` 两个用户概念。

### 6.2 PackageResolver：现在需要，但保持函数式

首期已有 local directory 与 Git 两个来源，因此需要 source resolution seam；它应是“输入 source，输出 immutable resolved tree/diagnostics”，而不是 lifecycle-heavy plugin API。解析后生成 lock、hash、权限清单与兼容性报告。

### 6.3 RunStore 与 ArtifactStore：先做具体实现，不做 trait

只有 SQLite 与本地 content-addressed directory 一个实现。直接写具体模块与事务边界；未来真正出现 cloud projection 或另一种 blob store 时，再从已验证的调用面提取接口。现在建立 repository/provider/DAO 泛型层只会隐藏 SQL 事务与 lineage 约束。

### 6.4 Projection：协议稳定，Renderer 不开放代码

Activity、Attention、Artifact、Graph state 使用版本化 DTO。主题只映射 semantic tokens；Package 可推荐 Theme/Accent 和 Artifact schema，但不能注入 Vue component。只有出现至少两个无法用 Markdown/JSON/Diff/Table/File 组合表达的真实 Artifact 后，才设计 sandboxed renderer extension。

## 7. RunEvent 与状态推进

推荐最小 envelope：

```text
id, occurred_at, type, subject,
run_id, node_run_id?, attempt_id?, session_id?,
correlation_id?, causation_id?,
payload_version, payload
```

一次状态推进在同一 SQLite transaction 中：

1. 校验 expected current state；
2. 更新 current projection；
3. append RunEvent；
4. 写 durable outbox（只有未来 cloud sync 真正实现时才启用发送器）。

不要做纯 Event Sourcing replay framework。事件用于审计、Timeline、UI 重连与未来云投影；当前表用于调度与查询。不要把高频 text delta 全部升级成关键 RunEvent：可批量落 Activity chunk，关键生命周期事件必须耐久。

## 8. Package 协议的最低形状

Package 是可版本化目录，不是 Host 进程插件：

```text
package.yaml
pipelines/*.yaml
skills/*/SKILL.md
prompts/*.md
schemas/*.json
assets/*
tests/*
```

Graph 使用 plain data；Schema validation 与 semantic validation 分开。安装必须检查：未知版本、缺失 Node/edge、不可达 Node、无上限循环、缺失 output contract、权限/Runtime 不满足、source/hash 不固定。Run 创建后冻结 resolved tree。

Package 只能声明：所需 Runtime capability、Skill/MCP/Connector 配置、Artifact contracts、Gate/risk 与推荐 Theme。它不能：注册 Scheduler callback、运行 WebView 代码、覆盖核心状态机、读取 Secret value、动态改写运行中的正式 Graph。

## 9. 必须推迟到第二个真实实现出现的抽象

| 推迟项 | 现在怎么做 | 何时抽象 |
|---|---|---|
| Storage backend trait | 直接 SQLite + 本地 blob | 真正实现 cloud/remote store 时 |
| Cloud sync provider | 表中预留可同步分类；不写 provider framework | 第一种服务端协议确定后 |
| Generic Connector SDK | Action 内先支持 process 与 MCP config | 至少两个非 MCP 业务平台出现后 |
| Arbitrary Executor plugin | sealed `agent/action/gate` | 两种无法表达的执行机制出现后 |
| Renderer plugin | 通用安全 renderer + JSON Schema | 至少两个无法声明式渲染的 Artifact 后 |
| 通用 Resource Lease DSL | 并发上限 + worktree lease | 端口和远程环境冲突真实发生后 |
| Expression language | 少量 typed conditions | 条件组合超出现有 schema 后 |
| Dynamic Graph mutation | 固定 Graph + 动态 Activity | 明确的 map/fan-out 用例出现后 |
| Universal Transcript model | Activity projection + raw vendor payload | 两个消费方需要稳定跨 Agent transcript 后 |
| Context manager/compactor | 完全委托 Runtime | 不应因第二个 adapter 而抽象；这是非目标 |

原则：**不要为“可能有”建立 seam；只在两个真实实现已经迫使调用者产生分支时提取 seam。**

## 10. 删除清单

以下内容不进入 MVP；若代码设计开始需要它们，应优先删除而非完善：

- 自研模型 agent loop、Session 格式或 context compaction。
- ACP 替代协议，或基于 ACP v2/unstable MCP-over-ACP 的必选能力。
- Dify 式拖拽 Graph Builder。
- 任意 Package 代码控制 Scheduler 或进入 WebView/Host。
- Jira、Linear、OCM、GitHub、Deploy 等业务名称进入 core enum。
- “所有 Agent 都支持 resume/fork/plan/diff”的虚假统一层。
- 用 PTY 文本解析作为正常结构化接入路径。
- 把每个 tool delta、日志、临时文件自动提升为 Artifact。
- 把 Run Brief 当作 checkpoint 或唯一控制状态。
- 覆盖旧 Attempt/Artifact；所有修订和反馈都追加。
- 纯 Event Sourcing reducer framework。
- 首期嵌入 Temporal、LangGraph 或 acpx runtime。
- 首期多数据库、分布式调度、远程 Runner、Marketplace、签名基础设施。
- Renderer/plugin 级自定义 Vue 页面。
- 允许 Agent 在运行时任意增加/删除正式 Node。
- 通用 policy language；先实现明确的风险等级、Gate 和最严格优先级。

## 11. 复用矩阵

| 需求 | 首选复用 | 使用方式 | 不采用 |
|---|---|---|---|
| Agent 公共接入 | [ACP Rust SDK](https://github.com/agentclientprotocol/rust-sdk) | v1 client + capability negotiation | 自研统一 Agent wire protocol |
| Pi | [Pi RPC](https://pi.dev/docs/latest/rpc) | 用户 CLI 子进程、结构化事件、native resume | TUI scraping、嵌入 TS SDK 到 Rust core |
| Codex | [Codex app-server](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md) | ACP baseline + native rich events | experimental WebSocket 作为 MVP 依赖 |
| OpenCode | [`opencode acp`](https://dev.opencode.ai/docs/cli/) | ACP baseline；HTTP 后加 enhancement | 同一产品暴露成两个 Runtime |
| Claude | ACP 官方生态适配 + Claude 自有能力探测 | 以实际 capability 为准 | 假设所有版本均可原生 resume |
| Flow 语义 | [acpx Flows](https://github.com/openclaw/acpx/blob/main/docs/flows.md) | 参考数据形状、测试与 replay viewer | 直接绑定 pre-1.0 runtime |
| 长任务原则 | [Temporal Architecture](https://github.com/temporalio/temporal/blob/main/docs/architecture/README.md) | definition/run、effect、idempotency | 引入 Temporal 服务栈 |
| checkpoint/HITL | [LangGraph Persistence](https://docs.langchain.com/oss/python/langgraph/persistence) | checkpoint 内容与 immutable fork 语义 | 让 graph framework管理 Agent context |
| 产品/远程边界 | [Craft Agents OSS](https://github.com/craft-ai-agents/craft-agents-oss) | Attention、Agent-native UX、thin client | 复制其存储模型或模糊 Source 边界 |

## 12. 建议实现顺序与架构验收

### 依赖采用策略

依赖应按“协议优先、库其次、运行时最后”的顺序选择，并逐层限制耦合面：

1. **稳定协议直接采用**：ACP v1、JSON Schema、JSON-RPC/JSONL 等进入边界契约；所有 capability 在运行时协商，不靠版本号猜测。
2. **官方 SDK 包在窄适配层内**：ACP Rust SDK 只出现在 `runtime/acp`；Vue Flow 与 ELK 只出现在 Graph projection；SQLite driver 只出现在持久化模块。核心领域对象不暴露第三方类型。
3. **用户 CLI 作为外部 Runtime**：Pi、Codex、Claude Code、OpenCode 由 Onboarding 探测并由 Runner 启动；不把它们打包进 App，不接管认证，不假设相同版本或相同恢复能力。
4. **实验能力必须可拔除**：ACP v2、MCP-over-ACP、Codex WebSocket、acpx Flow 只能放在 feature flag 或研究分支后，不能成为 Run Snapshot 可读、Runner 可启动或 UI 可重连的必要条件。
5. **锁定来源而非冻结整个生态**：Rust/JS 构建依赖使用 lockfile；Git Package 固定 commit SHA 与内容哈希；Runtime CLI 记录探测到的版本、协议版本和 capability snapshot。不要把用户 CLI 复制进 Package lockfile。
6. **接受性门槛**：新增依赖必须替代一块我们不具差异化价值的代码，并具备维护主体、许可证可接受、可离线失败、可被 adapter 隔离、不会把遥测或云依赖带进 Host 五项条件。

建议的 MVP 直接依赖只有：Tauri/Vue 桌面栈、SQLite Rust driver、ACP Rust SDK、Vue Flow、ELK，以及序列化/Schema 校验等基础库。Pi/Codex/OpenCode/Claude Code 是外部可探测 Runtime，不是链接进核心的库；acpx、Temporal、LangGraph、Argo、Prefect 和 Craft Agents 是设计与测试参考，不是产品运行依赖。

### 主要风险与控制

| 风险 | 已知事实 | 面向本产品的推断/控制 |
|---|---|---|
| Runtime 能力碎片化 | ACP 的 load/resume 等能力需协商，各 Agent 的 native event 也不同 | UI 按 capability 展示恢复等级；不把 native 增强压进最低公共接口 |
| Pi 包名与生态迁移 | Pi 已迁移到 `earendil-works/pi` 与 `@earendil-works/*` | Doctor 同时识别旧安装，但文档与新安装只指向当前官方身份 |
| 第三方 CLI 网络行为 | Pi 官方存在可关闭的版本检查/telemetry；其他 CLI 也有各自策略 | “Agent Pipeline 无遥测”只覆盖自身；Onboarding 显示外部边界并提供官方 opt-out 配置入口 |
| Package 供应链 | Pi Extension/Package 可执行任意代码并继承用户权限 | Pipeline Package 默认声明式；Git 固定 SHA/hash；代码扩展单独审查、授权和隔离 |
| Alpha 依赖漂移 | acpx 明确处于 alpha，ACP v2/MCP-over-ACP 仍不稳定 | 只借数据形状和测试案例，不把它们放入持久格式或启动关键路径 |
| “恢复”承诺过度 | 有些 Runtime 只能重连进程，有些能 resume Session，有些只能重试 | 分开标示 UI reconnect、native session resume、new Attempt retry、manual recovery |
| 事件量膨胀 | Agent 会产生大量 token delta、日志和工具流事件 | 生命周期事实耐久保存；高频 delta 合并为 Activity chunk；Artifact 必须显式发布 |
| Graph/上下文双重编排 | Runtime 已经负责模型 Context 与 compaction | Runner 只保存 handle、cursor、Handoff 和 Artifact refs；Run Brief 不参与调度判断 |
| 抽象提前固化 | 只有一个实现时无法验证通用接口是否真实 | 除 Runtime/Package source 外先写具体模块；第二个真实实现出现后再提取 seam |
| 本地 Runner 与未来云耦合 | 当前产品要求本地独立、未来可能增加云投影和远程命令 | 事件 envelope 保持可投影，云只消费显式允许的数据；本地执行永不依赖云可用性 |

### Slice 1：Pi 单 Runtime 走通真实恢复

先用 Pi RPC 做 `probe → start → prompt → event projection → interrupt → process/UI reconnect → resume/get_entries(since)`。这能最早验证产品真正的难点，而不是先写四个空 Adapter。

验收：关闭 UI 不丢 Runner；重开后 Graph、Activity、Session 与最后 cursor 一致；Pi compaction 后仍能继续；Node 完成以 settled 状态判断。

### Slice 2：最小循环与交付

实现 `Spec → Implement → Review → feedback edge`，每轮新 Attempt；发布 immutable ArtifactRevision/Handoff，生成 Run Brief projection；验证循环上限与 Attention。

### Slice 3：第二 Runtime 迫使 seam 成形

接 Codex ACP/App Server。只有此时才固定 `RuntimeAdapter` 公共 DTO；比较 Pi 和 Codex 的 resume、diff、plan、permission 差异，确认没有最低公分母泄漏。

### Slice 4：Package 自举

让 Pipeline Authoring Skill 生成 plain Package，执行 validate、preview、permission review、sandbox test、install。用户只通过自然语言修改，Graph 始终只读投影。

### 架构验收问题

- 删除任一 Runtime Adapter，RunEngine 是否无需修改？
- 新增 Jira/OCM 集成，核心 enum 是否无需增加业务名？
- Agent Session 丢失时，能否从 Run Snapshot + Handoff + Artifact 开新 Attempt？
- UI 崩溃时，Runner 是否不依赖 Vue 内存状态？
- Runtime 发出未知 event 时，是否能保存 raw payload 而不让 Run 失败？
- Package 更新后，旧 Run 是否仍使用原 snapshot？
- Review 循环十次后，是否能查看每个 Attempt 和 Artifact lineage 而无覆盖？
- Theme/Package 是否无法改变状态语义和 Review evidence contract？

如果这些问题的答案都是“是”，内核已经足够深；如果答案依赖更多框架层，优先回看删除清单。

## 最终建议

Agent Pipeline 应成为一个**小型、耐久、可观察的 control plane**：它知道任务的结构和事实，却不知道模型如何思考；它能显示 Agent 的活动，却不要求 Agent 管 UI；它能恢复 Pipeline，却不伪造 Runtime 不具备的 Session 恢复；它允许团队分发能力，却不让 Package 代码接管 Host。

Pi 的真正启发是：扩展性来自内核拒绝拥有不属于它的职责。对本项目而言，最重要的不是再增加一个抽象，而是持续守住三条边界：**Agent 拥有 Context，Runner 拥有 Control，UI 拥有 Projection。**
