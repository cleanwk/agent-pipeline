# Agent-native 架构与当前实现审阅（2026-08）

## 结论

方向基本正确，表述需要收窄。

- **已经过时的**不是所有编排，而是把模型能够自行规划、选工具、反思和修正的过程拆成大量固定 LLM/Tool 节点，再要求人维护连线、变量映射和 Prompt plumbing。
- **仍然必要的**是 harness 级协调：Session 生命周期、工具暴露、权限与审批、并发/预算、幂等副作用、耐久事件、Artifact、人工中断和恢复。这些是模型能力增强后仍不会自动消失的系统责任。
- **Skill** 适合承载领域方法、工作步骤、脚本、参考资料和验证方式；**MCP** 适合承载工具/资源/交互 UI 的标准连接。两者都不等于 durable execution，也不能替代权限、状态和恢复内核。
- 最合适的形态不是 Dify 式强 DAG，也不是“完全没有编排”，而是：**冻结 Mission Contract，Agent 动态维护 PlanRevision，Harness 执行并记录，Renderer 投影真实事件和产物；只有审批、风险、副作用和交付契约保持确定性。**

配套的一手资料调研见 [agent-native-orchestration-boundaries-2026.md](agent-native-orchestration-boundaries-2026.md)。

## 对产品理念的判断

当前仓库的正式理念其实已经大体站在正确一侧：

- [README](../../README.md) 明确“不重新实现 Agent 推理和上下文管理、不提供拖拽式流程画布”。
- [ADR 0003](../adr/0003-orchestrate-agent-work-without-owning-agent-reasoning.md) 把 Host 的责任限定为可观察、持久事实、交接、中断和恢复。
- [ADR 0009](../adr/0009-author-pipelines-through-intent-not-a-visual-editor.md) 规定自然语言生成变更提案，Graph 只用于解释、审查和控制。
- [ADR 0017](../adr/0017-keep-the-pipeline-execution-kernel-small.md) 要求内核只保留稳定状态机、Attempt、依赖、事件和有限 gate。

问题不是顶层理念选错，而是领域模型还残留了传统 workflow 假设：`PipelineDefinition.nodes + edges` 同时承担了任务合同、Agent 计划、调度规则和 UI 拓扑四种责任。真正 Agent-native 的实现应把它们拆开：

| 概念 | 是否冻结 | 谁产生 | 用途 |
| --- | --- | --- | --- |
| `MissionContract` | Run 启动时冻结 | Package/用户/Policy | 目标、边界、允许能力、必需产物、风险 gate、预算 |
| `PlanRevision` | 只追加新版本 | Lead Agent | 当前计划、动态分解、并行/委派关系、完成标准 |
| `ExecutionEvent` | 永远追加 | Harness/Runtime/Tool | 真实调用、状态、Attention、恢复事实 |
| `ArtifactRevision` | 永远追加 | Agent/Tool | 可交付证据及 lineage |
| `RunProjection` | 可重建/更新 | Projector | Graph、Timeline、Inspector、Deliverables 的 UI 读模型 |

这样 Graph 可以显示 Agent 的真实计划和实际执行，而不是成为执行本身。确定性 macro-flow 只在确有必要时存在，例如“生产部署前必须批准”“付款工具最多执行一次”“最终产物必须通过 schema/eval”。

## 当前实现的优点

1. **Host、Runner、Package 已有正确的物理分层。** Tauri 窗口关闭后 Runner 可独立存在，Unix Socket 权限收紧，本地 SQLite 使用 WAL。
2. **声明协议是文本化且可审查的。** 路径逃逸和 symlink 安装已有防护，节点类型和边引用有基础验证。
3. **UI 的核心信息架构是对的。** Attention、Graph、Activity、Attempt、Artifact、Definition 被明确区分，Graph 已开始消费 Package edges。
4. **文档没有掩盖纵向切片的性质。** [README 第 23 行](../../README.md#L23) 已说明工作节点仍由确定性示例驱动，真实 Prompt 尚未执行。
5. **现有测试全部通过。** `pnpm test` 通过 3 个 Vitest 与 9 个 Rust 测试；它们能证明演示状态持久化、基础 Package 校验和 socket 往返正常。

这些都是可保留的资产，不需要推倒 UI 或 Package/Runner 分层。

## 实现问题（按优先级）

### P0：Package 能安装和展示，但不能驱动运行

`LoadedPackage::load` 会解析 `PipelineDefinition`，但 `Engine` 从未接收或保存它。`Engine::open` 总是播种 `demo_projection()`；`Engine::dispatch` 直接判断 `review`，并直接修改 `implement`、`deploy`、`smoke`。见 [pipeline-core/src/lib.rs 第 513 行](../../crates/pipeline-core/src/lib.rs#L513) 和 [第 563 行](../../crates/pipeline-core/src/lib.rs#L563)。

因此当前存在两个不相交的模块：

- Package 模块：解析、验证、安装、Definition UI；
- Demo 模块：真正响应按钮并推进固定七阶段状态。

这意味着任意新 Package 即使安装成功也无法创建或执行 Run。它是当前最重要的真实性缺口。

**建议：** 不要先实现通用 DAG scheduler。先让 `start_run(MissionSnapshot)` 启动一个真实 Lead Agent Session；Package 提供 entry skill、能力/风险/产物合同，Agent 通过结构化旁路事件发布 `plan.revised`、`activity.*`、`artifact.published` 和 `attention.requested`。只有明确声明的 gate/effect 由内核做确定性校验。

### P0：Skill 和 MCP 只是 UI 字符串，不是可用能力

`NodeDefinition.skills` 是 `Vec<String>`，`mcp` 是任意 `serde_yaml::Value`。验证只确认 `prompt` 和 `schema` 文件存在，不解析 skill、不检查 MCP 字段、不做工具发现/能力匹配，也不生成 Runtime 配置。Runner 中唯一的 Runtime 代码是使用 `--no-session --no-extensions` 的 Pi `get_state` 探针，见 [pipeline-runner/src/lib.rs 第 35 行](../../crates/pipeline-runner/src/lib.rs#L35)。

独立示例仓库声明了 `grill-with-docs`、`to-ticket`、`to-spec`、`implement`、`tdd`、`code-review`、`deploy`、`smoke-test`，但没有任何 `skills/*/SKILL.md`；这些 ID 在本仓库也没有 resolver。所谓 MCP server 也只有显示逻辑。

**建议：** 把 Package 主体从 “nodes/prompts” 转为可实际挂载的能力包：

```text
agent-pipeline.package.yaml
mission.yaml
skills/*/SKILL.md
skills/*/scripts/*
schemas/*
renderers/* (先只允许声明式 metadata)
evals/scenarios.yaml
```

安装阶段解析并锁定 skill source/version/digest、MCP server identity/tool schemas、权限和 Runtime capability；启动时只给 Agent 暴露本次 Mission 需要的最小能力集合。

### P0：当前“断点续传”只是恢复一个 JSON 画面

SQLite 表只有 singleton `current_run(projection_json)` 和无 run identity 的 `run_events`；没有 Run/Attempt/Session/ToolCall/Attention/Checkpoint 表，也没有 Runtime handle、entry cursor、pending approval、幂等 key 或租约。见 [pipeline-core/src/lib.rs 第 515 行](../../crates/pipeline-core/src/lib.rs#L515)。Pi 探针读到的 `session_id/session_file` 不会入库，也没有 `start/resume/prompt/events/interrupt`。

所以当前能恢复的是“上次 UI 选中了哪个节点、演示走到哪一步”，不能恢复：

- 正在进行的 Agent Session；
- 已发出但结果未确认的 Tool/MCP call；
- 等待用户批准的调用；
- Runtime 事件增量游标；
- crash 后是否应重试、重新附着或创建新 Attempt。

**建议：** 将“恢复”定义为可测试的协议，而不是数据库存在即可：`session_handle + runtime_cursor + pending_effect + idempotency_key + last_durable_event`。至少覆盖 Runner 在 tool call 前后各崩溃一次、UI 关闭重连、审批等待数小时后恢复、Runtime 无原生 resume 时创建新 Attempt 四个场景。

### P1：冻结定义的完整性摘要不覆盖实际执行内容

Package digest 只序列化 `manifest + parsed pipelines`，见 [pipeline-runner/src/lib.rs 第 380 行](../../crates/pipeline-runner/src/lib.rs#L380)。Prompt、JSON Schema、未来 SKILL.md、脚本和 assets 的文件内容都未进入摘要。修改 `prompts/implement.md` 不会改变 digest。

此外，同一 `name/version` 已存在时安装函数直接返回成功，不比较 source digest，也不确认目录内容，见 [pipeline-runner/src/lib.rs 第 435 行](../../crates/pipeline-runner/src/lib.rs#L435)。这与“不可变安装、Run 冻结定义和依赖”的产品承诺不一致。

**建议：** 对规范化后的完整文件树做 content-addressed digest；安装目标使用 digest 或在 `name/version` 冲突时要求 digest 完全相同；Run 保存 package digest、entry mission id、skill/MCP/tool schema locks 和 Runtime adapter version。

### P1：协议校验不足以保障一个动态 Agent Run

当前校验没有检查不可达节点、空输出合同、handoff 引用是否存在、skill 是否解析、MCP tool 是否存在、capability 是否可满足、decision condition 是否有效、approval 与副作用风险是否匹配。`tests/scenarios.yaml` 没有任何 consumer。

有界循环检查也有逻辑漏洞：它收集全图所有 cycle edge，只要其中**任意一条**有 loop policy，就放过全图；两个互不相干的 cycle 中，一个有界、一个无界也会通过。见 [pipeline-core/src/lib.rs 第 338 行](../../crates/pipeline-core/src/lib.rs#L338)。

**建议：** 如果新架构仍保留 deterministic macro-flow，只校验少量 gate/effect 关系，并按 SCC 或枚举 back-edge 保证每个可循环区域都有统一 budget；更重要的是执行 Package 自带 eval/scenario，而不是只验证 YAML 形状。

### P1：Attempt、Event 和多 Run 还不是真实领域模型

`attempt` 只是 `PipelineNode` 上的计数器；打回时直接替换 Implement activities，旧 Attempt 没有独立 identity、状态、输入快照和 session lineage。RunEvent 只有 sequence/type/payload/time；缺少 run/attempt/session/correlation/causation/payload version。`event_count` 统计整张全局表，UI selection 也写成 durable domain event。系统只能有一个 Run。

**建议：** 让 `Attempt` 成为不可变输入快照 + 可变生命周期的独立记录；event envelope 至少包含 `run_id, attempt_id?, session_id?, correlation_id?, causation_id?, payload_version`。UI selection 留在客户端；事件只记录可审计的运行事实。

### P1：Renderer 仍绑定七阶段演示

Graph 节点/edges 已部分数据驱动，但位置按数组每四个换行，edge path 使用固定 `680/696` 坐标，历史区只渲染 `nodes.slice(3, 7)`。见 [RunGraph.vue 第 10 行](../../apps/desktop/src/components/RunGraph.vue#L10)。Node Inspector 的 Review diff、日志、打回文案和按钮是固定数据，见 [NodeInspector.vue 第 43 行](../../apps/desktop/src/components/NodeInspector.vue#L43)。Deliverables 的四个分组和 node id 过滤也硬编码在 [App.vue 第 223 行](../../apps/desktop/src/App.vue#L223)，没有使用 Package 的 `deliverySlots`。

**建议：** Renderer 只消费版本化 projection：

- Graph renderer：`PlanRevision + ExecutionEvent`；
- Activity renderer：统一 trace/span envelope；
- Artifact renderer：先做安全的 Markdown、JSON、Diff、Table、File、Image；
- Interactive renderer：以后通过 sandboxed UI resource/MCP Apps 类协议接入，不允许 Package 注入主 WebView 代码。

### P2：多 Pipeline Package 的 entry identity 丢失

Runner inspection 支持多个 pipelines，但 `RunProjection` 只保存 package/version/digest，没有 pipeline/mission id；前端固定取 `inspected.pipelines[0]`，见 [api.ts 第 76 行](../../apps/desktop/src/api.ts#L76)。一旦 Package 真包含多个入口，Run 无法证明自己冻结的是哪一个。

**建议：** 新 RunSnapshot 显式保存 `package_digest + mission_id + mission_schema_version`，UI 严格按该 identity 投影。

## 推荐目标架构

用一个深模块隐藏复杂性，避免把 Runtime、Skill、MCP、Store 的细节泄漏到 UI 或 Package 作者：

```text
RunCoordinator interface
  start(MissionSnapshot) -> RunId
  steer(RunId, UserInput) -> Accepted
  resolve_attention(RunId, AttentionId, Decision) -> Accepted
  interrupt(RunId) -> Accepted
  snapshot(RunId) -> RunProjection
  events(RunId, after_cursor) -> EventPage

RunCoordinator implementation
  RuntimeAdapter(s)
  SkillResolver + ToolBroker(MCP/local/agent-as-tool)
  Permission/Budget/Idempotency policy
  Session/Attempt/Effect recovery
  Event normalization + Artifact publication
  Plan projector + UI projectors
```

这是一个深模块：调用者只学习 6 个稳定操作，内部可以容纳多个 Runtime 的差异、工具审批、重连、事件归一化和恢复。`RuntimeAdapter` 在第二个真实 Runtime 到来后才冻结；SQLite Store 仍可保持具体实现，不必先造泛型 repository 层。

## 建议的实施顺序

1. **暂停扩展 YAML DAG 和演示 UI。** 保留现有 demo 作为明确标记的 fixture，不再让它代表 Engine。
2. **完成一个真实 Lead Agent 纵切。** 用 Pi 或 Codex 中任一个实现 `start/resume/prompt/events/interrupt`，真实挂载一个 Package 内 SKILL.md 和一个 MCP/local tool，真实发布 Artifact。
3. **把 Mission/Plan/Trace 分开。** 冻结 MissionContract；允许 Agent 追加 PlanRevision；UI Graph 改为计划/执行投影。
4. **先做恢复再做第二 Runtime。** 建 crash-point 集成测试，确保 pending effect 不重复、Session 能重新附着、不能 resume 时形成新 Attempt。
5. **补齐 content-addressed Package。** 全树 digest、skill/tool locks、同版本冲突检测、Package eval runner。
6. **把 UI 去七阶段化。** gate 决策由 schema 渲染，delivery slot 来自定义，日志来自 event stream，Artifact 由 media type renderer 选择。
7. **接第二个 Runtime 后冻结 seam。** 用真实差异反推最小 `RuntimeAdapter`，避免先设计最低公分母。

## 必须新增的验收测试

- 任意三节点 Mission（节点名不含 review/implement/deploy/smoke）能够启动并完成。
- Agent 自行新增/删除/重排计划步骤时，旧 PlanRevision 仍可查看，Run 不丢失。
- Package 引用不存在的 Skill/MCP tool 时安装失败，并给出可操作诊断。
- 修改任意 prompt/SKILL/script/schema 后 Package digest 必变。
- Runner 在副作用提交前、提交后但回执入库前分别崩溃，恢复后副作用最多一次。
- 等待 approval 时关闭 UI/Runner，重启后同一 Attention 与证据可继续。
- Runtime 支持 resume 时复用原 Session；不支持时创建新 Attempt 并显式 handoff。
- 两个并发 Run 的 event count、artifact、attention 和 selected UI state 不串线。
- Package 自带 scenarios 真正执行，并覆盖失败、budget exhausted、tool denied 和 output schema invalid。

## 最终判断

项目不需要改成 Dify，也不应继续把七阶段 DAG 做得更强。当前最有价值的产品机会是 **Agent harness 的 mission control**：Skill 让 Agent 学会组织复杂工作，MCP/Tools 让它获得能力，Renderer 让人看懂和干预，durable kernel 让长任务可恢复且副作用可控。

但要兑现这个定位，下一里程碑必须从“可展示的 Pipeline demo”切到“真实 Session + 真实 Skill + 真实 Tool + 真实恢复”的最小闭环。否则文档讲的是 Agent-native，运行时仍然是传统固定编排的演示状态机。
