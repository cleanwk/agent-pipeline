# Agent-native 时代的编排边界：DAG、Harness、Skills、MCP 与耐久执行

> 调研日期：2026-08-18  
> 范围：仅使用 OpenAI、Anthropic、Google ADK、Cloudflare、LangGraph、Dify、Model Context Protocol 的官方文档、官方工程文章和官方源码仓库。  
> 证据标记：**事实**表示可由紧邻的一手链接直接验证；**推论**表示基于多项事实的架构判断；**建议**表示面向 Agent Pipeline 的设计选择。

## 结论先行

用户的方向**大体正确，但“Dify 式强 DAG 已经被淘汰、Tools 根本不需要编排”说得过头**。

更准确的判断是：

1. **事实：开放式复杂任务的默认执行者正在从“人预先穷举步骤”转向“模型在 Harness 中动态规划并使用 Tools”。** Anthropic 把 Workflow 定义为预设代码路径、Agent 定义为模型动态决定过程和工具；OpenAI Agents SDK 同时支持 LLM 决策与代码编排；Cloudflare也把 Harness 明确定义为负责模型循环、工具选择和继续/停止判断的层。[Anthropic：Building effective agents](https://www.anthropic.com/engineering/building-effective-agents) [OpenAI：Agent orchestration](https://openai.github.io/openai-agents-python/multi_agent/) [Cloudflare Agents](https://developers.cloudflare.com/agents/)
2. **事实：拖拽画出完整流程图作为默认 Authoring 方式正在失去吸引力。** OpenAI 在 2025 年推出可视化 Agent Builder，随后于 2026-06 宣布将在 2026-11-30 下线，并把“继续作为代码的 workflow”迁移方向指向 Agents SDK、自然语言场景指向 Workspace Agents。这是一个很强的产品信号，但只是单一厂商决策，不能单独证明所有 Visual Workflow 都已死亡。[OpenAI：Introducing AgentKit（含 2026-06 更新）](https://openai.com/index/introducing-agentkit/)
3. **事实：主流实现没有取消编排，而是在形成混合架构。** Dify 1.16 同时引入 Linux sandbox、Skills 和可复用 Agent，并允许 Agent 独立工作或作为 Workflow Node；其官方文档仍建议在需要固定顺序、条件分支和多 Agent 交接时使用 Workflow。Google ADK 2.0 用 Graph/Dynamic Workflow 取代较死板的顺序/循环/并行模板，而不是取消 Workflow；Cloudflare把 Agent 与 Workflow 明确称为互补。[Dify Agent 概览](https://docs.dify.ai/en/self-host/use-dify/build/new-agent/overview) [Dify 1.16 Release](https://github.com/langgenius/dify/releases) [Google ADK Graph Workflows](https://adk.dev/graphs/) [Google ADK Dynamic Workflows](https://adk.dev/graphs/dynamic/) [Cloudflare：Using Agents with Workflows](https://developers.cloudflare.com/agents/concepts/workflows/)
4. **事实：Skills 已成为封装 SOP、领域知识、脚本和模板的跨平台方向。** Anthropic、Google ADK、Cloudflare 都采用按需发现/加载的 Skill 结构，以 progressive disclosure 避免把全部流程知识塞入每轮上下文；OpenAI 2026 Agents SDK 也把 Skills、MCP、AGENTS.md、Shell 和 Apply Patch 列为其 Model-native Harness 的标准原语。[Anthropic Agent Skills](https://www.anthropic.com/engineering/equipping-agents-for-the-real-world-with-agent-skills) [Google ADK Skills](https://adk.dev/skills/) [Cloudflare Agent Skills](https://developers.cloudflare.com/agents/runtime/execution/agent-skills/) [OpenAI：The next evolution of the Agents SDK](https://openai.com/index/the-next-evolution-of-the-agents-sdk/)
5. **事实：Skills、Tools 和更强模型没有消除耐久执行问题。** Anthropic 的长任务实验表明，即使 Frontier Model、Agent SDK 和 compaction 都存在，只给一个高层目标仍会出现一次做太多、跨上下文丢失进度和过早宣布完成；其解决方案依赖结构化 feature list、进度文件、Git、增量交接和验证。OpenAI、Cloudflare、LangGraph、Google ADK 均继续提供 checkpoint、resume、HITL、retry 或 durable state。[Anthropic：Effective harnesses for long-running agents](https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents) [OpenAI Agents SDK 2026](https://openai.com/index/the-next-evolution-of-the-agents-sdk/) [LangGraph Persistence](https://docs.langchain.com/oss/python/langgraph/persistence) [Google ADK Resume](https://adk.dev/runtime/resume/) [Cloudflare Durable Execution](https://developers.cloudflare.com/agents/runtime/execution/durable-execution/)

因此本报告的核心结论是：

> **DAG 没有消失；它应从“人类手工编程所有业务细节的主界面”，退到“少量确定性控制边界、耐久状态与运行投影”。开放式语义工作由 Agent + Harness + Skills + Tools/MCP 动态完成。**

这与 Agent Pipeline 当前倡导的边界相符：**Agent 拥有 Context 和语义规划，Runner 拥有耐久 Control，UI 拥有 Projection。** 需要修正的只是口径：不要宣称“无编排”，而应宣称“拒绝预写死的细粒度业务编排，只保留最小耐久控制面”。

## 1. 先拆开五个经常混用的概念

| 层 | 解决的问题 | 不应负责 |
| --- | --- | --- |
| Agent / Model | 理解目标、动态计划、选择工具、根据反馈改路 | exactly-once、进程崩溃恢复、权限事实 |
| Harness | Agent loop、上下文、工具路由、权限、sandbox、compaction、会话事件 | 固定业务部门流程、UI 布局 |
| Skill | 按需加载的 SOP、知识、脚本、模板、校验方法 | 长任务状态机、跨进程 checkpoint、事务语义 |
| MCP | Tools/Resources/UI/长任务句柄等跨产品协议 | 完整业务 Scheduler、产品级 Run 历史 |
| Durable Control Plane | retry、timeout、idempotency、HITL、checkpoint、审计、恢复 | 替模型决定每一个语义步骤 |

**事实：** Anthropic Managed Agents 把 Session、Harness、Sandbox 明确拆为可独立替换的接口；Session 是 append-only 事件日志，Harness 是调用 Claude 并路由 Tool 的 loop，Sandbox 是执行环境。Harness 或容器失败后，可以从外置 Session 的最后事件恢复。[Anthropic：Scaling Managed Agents](https://www.anthropic.com/engineering/managed-agents)

**推论：** “Harness + Tools 不需要编排”在术语上不成立，因为 Harness 本身就在编排 Agent loop 和 Tool 调用。真正可以被删除的是第二套由产品维护、试图替模型决定所有语义步骤的细粒度编排。

## 2. “强 DAG 被淘汰”到底对到什么程度

### 2.1 对的部分：静态细粒度 DAG 不适合开放任务

**事实：** Anthropic 认为 Agent 适合需要 flexibility 和 model-driven decision-making 的任务，而预定义 Workflow 适合可清晰分解、需要 predictability 和 consistency 的任务；其生产经验反而建议从最简单的组合模式开始，避免不必要的复杂框架。[Anthropic：Building effective agents](https://www.anthropic.com/engineering/building-effective-agents)

**事实：** OpenAI Agents SDK 把“让 LLM 决策”和“由代码编排”列为可混用的两类模式。开放任务中 Agent 可自主规划、调用 Tools、通过 Handoff 委派；代码编排则用于更确定的速度、成本和性能边界。[OpenAI：Agent orchestration](https://openai.github.io/openai-agents-python/multi_agent/)

**事实：** Google ADK 2.0 明确说静态 Graph 在复杂循环和复杂分支下可能变得难以管理，因此提供使用普通语言循环、条件和递归的 Dynamic Workflows；成功子节点仍自动 checkpoint，恢复时不重复执行。[Google ADK：Dynamic Workflows](https://adk.dev/graphs/dynamic/)

**事实：** LangGraph 也提供不要求把应用重构成显式 DAG 的 Functional API，用普通 Python 控制流保留 persistence、HITL、streaming 和 checkpoint；代价是运行图是动态生成的，不能像静态 Graph API 那样提前可视化。[LangGraph：Functional API](https://docs.langchain.com/oss/python/langgraph/functional-api)

**推论：** 对代码修改、调查、研究、设计等事先无法预测子任务数量和路线的工作，要求用户先画完整 DAG 会把模型已经能做的 planning 再手工实现一遍，并把变化频繁的模型能力冻结成产品配置。

### 2.2 过头的部分：确定性编排并未消失

**事实：** Dify 最新方向不是抛弃 Workflow，而是把新的 sandbox Agent 嵌入其中。官方明确建议：单个强 Agent 能自己到达目标时独立运行；需要固定顺序、条件分支、其他 Node 或多个专门 Agent 交接时，把 Agent 用作 Workflow 中的一步。[Dify Agent 概览](https://docs.dify.ai/en/self-host/use-dify/build/new-agent/overview)

**事实：** Dify 1.16 仍继续增强 Workflow Authoring、HITL 恢复、运行归档、可靠性、可观测性和“workflow as MCP server”；同时加入 Agent App、Skills、sandbox 和 Agent Node。这是**融合**，不是替代。[Dify 1.16 Release](https://github.com/langgenius/dify/releases)

**事实：** Cloudflare 列出的 Agent + Workflow 适用场景包括长时间任务、报告生成、HITL、保证交付、自动重试与多步操作；Workflow 的完成步骤永久化、失败自动 retry、等待事件最长可达一年，状态可在基础设施故障后恢复。[Cloudflare：Using Agents with Workflows](https://developers.cloudflare.com/agents/concepts/workflows/)

**事实：** Google ADK Graph Workflows 继续定位于需要精确 routing、分支、状态管理、非 AI 函数链和高可预测性的过程；ADK 2.0 淘汰的是较死板的 Template Workflow 作为默认形式，替代物是更灵活的 Graph 和 Dynamic Workflow。[Google ADK：Graph Workflows](https://adk.dev/graphs/) [Google ADK：Template Workflows](https://adk.dev/agents/workflow-agents/)

**推论：** 没有证据支持“DAG 作为执行/持久化结构已经淘汰”。证据支持的是：**固定图不再适合充当所有 Agent 语义的 Authoring DSL，但图、函数工作流或状态机仍是耐久控制与审计的有效实现。**

## 3. Skills 化是正确方向，但不是把整个系统写进 SKILL.md

### 3.1 Skills 适合承载什么

**事实：** Anthropic 将 Skill 定义为可动态发现的 instructions、scripts、resources 目录；启动时仅加载 name/description，命中后才读取完整 `SKILL.md` 和引用文件。Skill 还可以包含由 Agent 自主选择执行的确定性代码。Anthropic 明确提出 Skills 可以补充 MCP，教授 Agent 涉及外部工具和软件的复杂流程。[Anthropic Agent Skills](https://www.anthropic.com/engineering/equipping-agents-for-the-real-world-with-agent-skills)

**事实：** Google ADK 的 Skill 也是自包含的 instructions/resources/tools 单元，并以增量加载降低 context window 压力；该功能截至调研日仍标为 experimental。[Google ADK Skills](https://adk.dev/skills/)

**事实：** Cloudflare 的 Skill catalog 同样只把名称和描述放入 System Prompt，匹配后再激活；官方把 Skills 定位为“task-specific procedures, references, scripts, templates, assets”，并明确说不是 always-on System Prompt。脚本执行仍是 opt-in、experimental、默认受权限和超时限制。[Cloudflare Agent Skills](https://developers.cloudflare.com/agents/runtime/execution/agent-skills/)

**建议：** Agent Pipeline 的 Package 应优先把以下内容 Skills 化：

- 领域术语、成功标准、检查清单和常见失败模式；
- 如何发现并组合 MCP/本地 Tools；
- 任务特定的 prompt、schema、参考材料、脚本和测试；
- 如何生成 Handoff、Artifact，以及如何验证“完成”；
- Agent 能复盘后追加的经验，但必须走版本化 Change Proposal、评估和人工接受。

### 3.2 Skills 不能替代什么

**事实：** Anthropic 明确警告 Skill 可以携带指令和代码，恶意 Skill 可能导致数据外泄或非预期操作，因此应只安装可信来源并审计脚本和依赖。[Anthropic Agent Skills：Security](https://www.anthropic.com/engineering/equipping-agents-for-the-real-world-with-agent-skills)

**事实：** 更强模型会让某些 Harness 脚手架过时，但不会让所有控制都失去价值。Anthropic 在 Opus 4.6 上移除了不再必要的 sprint 分解，却保留 planner 与 evaluator；对于已落入模型可靠能力范围的任务，evaluator 成为开销，但边界外任务仍有明显收益。[Anthropic：Harness design for long-running apps](https://www.anthropic.com/engineering/harness-design-long-running-apps)

**推论：** Skill 是“可复用的做事方法”，不是“耐久运行事实”。它本身不提供 exactly-once、幂等 key、事务、重试预算、租约、进程恢复、版本冻结、Artifact lineage 或审批事实。

**建议：** 不要把 Pipeline Scheduler 变成一个巨型 Skill，也不要让 Skill 脚本直接改核心 Run 状态。Skill 可以建议计划、调用受控 Tool、输出 Proposal；Runner 才能提交状态转换。

## 4. MCP 很关键，但它不是完整 Orchestrator

### 4.1 MCP 的正确位置

**事实：** MCP 2026-07-28 的架构目标是让 Host 管理 Client、安全策略、授权和上下文聚合，让 Server 暴露专注、可组合的 Resources/Tools/Prompts。该协议把跨 Server 的复杂协调留给 Host，而不是让每个 Server 看见完整对话或彼此状态。[MCP Architecture](https://modelcontextprotocol.io/specification/2026-07-28/architecture)

**推论：** MCP 应是 Agent Pipeline 的 Capability/Tool/UI 互操作层，而不是 Pipeline Run、Attempt、Artifact lineage 和 Graph checkpoint 的唯一事实源。

### 4.2 MCP Apps 支持“Agent 执行 + 丰富渲染”

**事实：** MCP Apps 已在 2026-01 成为首个正式 MCP Extension。Tool 可以返回 sandboxed interactive UI，在对话内渲染 dashboard、form、visualization、document review 和 multi-step interaction；UI 与 Host 通过可审计 JSON-RPC 双向通信，Tool 调用仍可要求用户批准。[MCP Apps Announcement](https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/) [MCP Apps Overview](https://modelcontextprotocol.io/extensions/apps/overview)

**推论：** 这直接支持“Agent 产生结构化事实，UI 渲染可交互投影”的产品方向；但 MCP App 是表现/交互扩展，不应拥有 Scheduler。默认渲染器仍应覆盖 Markdown、JSON、Table、Diff、File、Graph、Timeline，只有需要领域交互时才启用 sandboxed MCP App。

### 4.3 MCP Tasks 支持长操作句柄，但不替你实现持久化

**事实：** 2026-07-28 版 MCP 把 Core 改为 stateless request/response，并把 Tasks 作为正式 Extension。Task 允许长操作返回 durable handle；Client 可断线后凭 ID 继续 poll、查看 progress、提交 `input_required` 回应或 cancel。官方实现指南要求 Client 持久保存 Task ID，Server 必须在返回前先耐久创建 Task。[MCP 2026-07-28](https://blog.modelcontextprotocol.io/posts/2026-07-28/) [MCP Tasks](https://modelcontextprotocol.io/extensions/tasks/overview)

**推论：** MCP Tasks 是跨 Client 的“远程 Operation Handle”，不是端到端 Workflow Engine。Server 仍要实现 durable task store，Host 仍要保存 Task ID 与本地 Attempt/Artifact/Attention 的关联；而且 Extension 需要 Client 与 Server 双方显式支持。

**建议：** `RuntimeAdapter`/`Action` 应 capability-negotiate MCP Apps 与 MCP Tasks；持久化 `server identity + tool + task_id + protocol/extension version + last observed state`，并将远程 Task 状态投影到 Attempt，而不是拿 Task 替代 Attempt。

## 5. “可断点续传”必须拆成四种不同承诺

| 恢复层级 | 最小事实 | 恢复含义 |
| --- | --- | --- |
| UI reconnect | durable event cursor | UI 重连并补齐 Activity，不代表 Agent 仍能续跑 |
| Agent Session resume | opaque runtime session handle | 恢复模型上下文/会话；能力取决于具体 Runtime |
| Orchestration resume | frozen Run Snapshot + checkpoint | 从已提交的 Node/Attempt/Artifact 边界继续调度 |
| Remote operation resume | MCP Task ID / provider job ID | 继续查询外部长操作，不重复提交副作用 |

**事实：** LangGraph 每步保存 checkpoint，支持 HITL、time travel、fault tolerance 和 pending writes recovery；其 Interrupt 恢复时会从 Node 开头重跑，所以 interrupt 前副作用必须幂等。[LangGraph Persistence](https://docs.langchain.com/oss/python/langgraph/persistence) [LangGraph Interrupts](https://docs.langchain.com/oss/python/langgraph/interrupts)

**事实：** Google ADK Resume 通过记录已完成 Agent workflow task 和 Event，从部分完成状态恢复；Custom Agent 默认不支持，需要自行实现增量 resume。[Google ADK Resume](https://adk.dev/runtime/resume/)

**事实：** OpenAI 2026 Agents SDK 把 Harness 与 Compute 分离，状态外置后可 snapshot/rehydrate，在原 sandbox 失败或过期时用新容器从 checkpoint 继续。[OpenAI Agents SDK 2026](https://openai.com/index/the-next-evolution-of-the-agents-sdk/)

**事实：** Cloudflare 的 Fiber 能把 Agent 内部工作和 checkpoint 存进 SQLite，但对独立、需逐步 retry 的多步操作仍建议使用 Workflow；这再次表明 Agent 内恢复与 Workflow 耐久性是不同层。[Cloudflare Durable Execution](https://developers.cloudflare.com/agents/runtime/execution/durable-execution/)

**建议：** 产品 UI 不应只显示一个模糊的 “Resumable”。应分别显示上述四种能力，并为无法原生 Session resume 的 Runtime 提供“从冻结输入、Handoff、Artifact 开新 Attempt”的明确降级路径。

## 6. 推荐的混合执行模型

```text
User intent / Package
        │
        ├── Skill catalog ──按需加载──> Agent Harness
        │                                  │
        │                                  ├── dynamic plan / tools / MCP / subagents
        │                                  └── runtime events / handoff / artifacts
        │
        └── Minimal durable control plane
                ├── coarse milestones / dependency edges
                ├── approval / typed human input
                ├── retry / timeout / loop ceiling / idempotency
                ├── frozen snapshot / checkpoint / immutable attempts
                └── events ──> Graph / Activity / Attention / Artifact projection
```

**推论：** 这里的 Graph 是“稳定责任边界和已发生因果关系”，不是 Agent 的完整思维链或预先穷举的 Todo 列表。Agent 在 Node 内产生的计划、子任务、Tool 调用属于动态 Activity；只有有独立输入输出契约、恢复/审批/资源边界的事项才值得升级为 Node。

### 何时应是 Skill/Agent，何时应是 Durable Node

| 问题 | Skill / Agent 内动态处理 | Durable Node / Gate |
| --- | --- | --- |
| 子步骤是否事先不可预测 | 是 | 否 |
| 是否只是语义推理或只读探索 | 是 | 通常否 |
| 是否包含不可逆/昂贵副作用 | 受控 Tool，但需外层边界 | 是 |
| 是否必须跨天等待人或外部事件 | 否 | 是 |
| 是否需要独立 retry/timeout/资源租约 | 通常否 | 是 |
| 是否需要正式 Artifact contract / lineage | 可生成候选 | 是 |
| 是否受合规顺序或职责分离约束 | 否 | 是 |

## 7. 面向 Agent Pipeline 的具体建议

### 7.1 保留并强化的设计

1. **建议：保留小型 Runner。** Runner 只拥有 Run Snapshot、Node/Attempt 状态、审批、循环上限、retry/timeout、Artifact、Attention、event log 和恢复边界；不拥有模型推理和 context compaction。
2. **建议：把复杂业务知识移入 Skills/Package。** Package 可声明 Skills、Prompts、Schemas、Assets、Tool/MCP requirements 和 coarse Graph；不要在 Core Enum 里加入 Jira、Deploy、Review、Research 等业务名。
3. **建议：自然语言 Authoring + 结构化 Proposal。** 模型根据真实流程生成/修改 Package；UI 展示 Graph、Diff、权限、验证和测试结果，用户接受后冻结版本。不要把拖拽连线做成主 Authoring 入口。
4. **建议：运行时可视化优先。** Graph Projection 展示粗粒度控制图，Activity Tree/Timeline 展示 Agent 实际动态路线；未知 Tool/Event 保留 raw payload，Renderer 只消费版本化事实。
5. **建议：恢复优先使用 Runtime 原生 Session；失败时创建新 Attempt。** 不自研通用 Transcript/Context 格式，不用回放 Tool 副作用来假装恢复 Agent。
6. **建议：MCP 是 Connector seam，不是 Core Domain。** Tools/Resources/Apps/Tasks 都经能力协商接入；Attempt、Artifact、Handoff 和 Attention 仍有产品自己的稳定协议。

### 7.2 需要警惕的实现反模式

- 每个 Agent 计划项都变成正式 Node，导致“双重 Scheduler”。
- Node prompt 写死 Tool 调用顺序，即使这些选择应由 Agent 根据观察动态决定。
- 让 Agent 动态增删正式 Graph Node 却不留下版本化 Proposal、因果关系和循环上限。
- 把 `SKILL.md` 当 checkpoint，或把对话 summary 当调度事实。
- 认为接入 MCP 就自然获得 auth、idempotency、resume、sandbox 和安全 UI。
- 将 UI 内存或 WebSocket 流当作 Run 的事实源。
- 把“进程还活着”“UI 已重连”“Session 可续”“远程 Job 可查”“Pipeline 可继续”合并成一个状态。
- 允许 Package/Skill/MCP App 未经审查地在 Tauri Host 或主 WebView 执行任意代码。

## 8. 可执行的架构验收问题

以下问题应全部能明确回答，而不是由 Prompt 暗示：

1. 一个 Node 内 Agent 临时规划出 30 个 Tool/子任务时，Core 是否仍只看到一个 Attempt 和动态 Activity，而不会创建 30 个正式 Node？
2. 去掉任何一个 Runtime Adapter 后，Runner 是否无需修改？
3. Skill 更新后，正在运行的 Run 是否仍可依据冻结版本解释和恢复？
4. Agent/Skill/MCP App 是否都无法直接提交核心状态转换？
5. Tool 产生副作用后 Harness 崩溃，是否能凭 idempotency key / remote task handle 判断该重试、查询还是人工介入？
6. UI 崩溃重开后，是否从事件游标恢复 Graph、Activity、Attention、Artifact，而不是从组件内存猜测？
7. Runtime 不支持 Session resume 时，是否诚实降级为新 Attempt，并携带显式 Handoff/Artifact，而不是宣称“原地续传”？
8. 用户是否可以只用自然语言创建流程，同时在应用前审阅结构化 Graph Diff、权限和风险变化？
9. Renderer 是否只改变表现，无法改变状态语义、审批结果和调度决定？
10. 每一个循环是否都有 termination condition、iteration ceiling 和 exhausted outcome？

## 9. 证据边界与最终判断

**事实边界：** 没有一篇权威论文或跨平台基准能证明“Visual DAG 已被整个行业淘汰”。现有一手证据主要是官方架构文档、产品路线和厂商实验。OpenAI 下线 Agent Builder 是强信号；Dify、Google、Cloudflare 和 LangGraph 继续投资 Workflow/Durable Runtime 则是同样强的反证。

**最终推论：** 截至 2026-08，行业共识不是“Agent 取代 Workflow”，而是：

- Model/Harness 负责开放世界中的动态路线；
- Skills 负责可组合、按需加载的领域 SOP；
- Tools/MCP 负责能力、数据、远程操作句柄与可选丰富 UI；
- Durable Runtime 负责系统不能交给概率模型的正确性；
- 可视化从“先画完再执行”转向“审阅意图、展示实际执行、处理 Attention、理解恢复边界”。

因此，Agent Pipeline 的理念应表述为：

> **我们不是一个 Dify 式低代码编排器。我们把复杂工作方法 Skills 化，让强 Agent 在原生 Harness 中动态工作；同时用一个最小、耐久、可审计的 Control Plane 保存人类真正需要控制和恢复的边界，并把运行事实渲染成可理解、可干预的 Projection。**

这比“完全不需要编排”更准确，也更能经受下一代模型继续变强：模型能力提升时，可以继续删掉过时的 Harness 脚手架；副作用、审批、持久化、权限和历史事实仍由稳定接口守住。
