# Agent Pipeline Platform

本上下文描述可安装、可执行并可审计的 Agent 工作流水线，以及承载它们的本地应用。

## Language

**Workspace**:
用户本机上的长期工作边界，汇集代码资源、已安装 Package、策略、集成配置和多个 Pipeline Run。
_Avoid_: Repository, Worktree, Run

**Host App**:
安装、配置、运行和观察 Pipeline Package 的产品本体；它不包含某一条具体业务流程。
_Avoid_: Platform, Client, Pipeline App

**Pipeline Package**:
可独立版本化、分发和安装的工作流产品单元，包含 Pipeline 的定义及其运行所需的扩展声明。
_Avoid_: Plugin, Skill, Workflow Repository

**Pipeline**:
由一组有依赖关系的 Node 构成的可复用工作流定义。
_Avoid_: Workflow, Job

**Pipeline Run**:
一次 Pipeline 的具体执行，拥有独立的状态、历史和产出。
_Avoid_: Pipeline Instance, Execution

**Run Snapshot**:
Pipeline Run 启动时冻结的 Package 版本、Pipeline 定义和有效配置，是该次运行可解释与可恢复的依据。
_Avoid_: Current Configuration, Latest Version

**Node**:
Pipeline 中具有明确输入、输出和执行策略的步骤。
_Avoid_: Stage, Step, Task

**Node Run**:
某个 Node 在一次 Pipeline Run 中的逻辑执行，聚合该节点历次 Attempt 的状态和产出。
_Avoid_: Node, Session

**Attempt**:
Node Run 的一次不可覆盖的执行尝试；重试或经反馈边重新执行都会产生新的 Attempt。
_Avoid_: Retry, Run

**Agent Session**:
由具体 Agent Runtime 管理的逻辑会话，是 Attempt 可选的执行通道，而不是 Pipeline 的事实来源。
_Avoid_: Node, Process

**Activity**:
Attempt 执行期间动态出现的计划项、子任务或工具活动，用于展示进度但不参与 Pipeline 调度。
_Avoid_: Node, Step

**Artifact**:
Attempt 产生、可供后续 Node 或人读取的有类型、可追溯结果。
_Avoid_: Output, Attachment, Context

**Artifact Revision**:
Artifact 的一个不可覆盖版本；后续修订通过谱系关系取代旧版本，而不改变历史。
_Avoid_: Edited Artifact, Latest File

**Delivery Slot**:
Pipeline Package 声明的重要交付位置，用于在一组通用 Artifact 中组织并突出业务期望的结果。
_Avoid_: Folder, Output Path

**Workspace Reference**:
由用户明确从某次 Pipeline Run 提升、可供 Workspace 中未来运行复用的 Artifact Revision。
_Avoid_: Memory, Automatic Context

**Handoff**:
Attempt 为后续工作明确发布的交接信息，指向本次执行值得继承的 Artifact、结论与未决事项。
_Avoid_: Transcript, Summary

**Run Brief**:
由已接受的 Handoff 组成的 Pipeline Run 公共上下文，供人查看并供后续 Node 按需读取。
_Avoid_: Global Session, Full Context

**Runtime Adapter**:
将某一种 Agent Runtime 的会话、事件、权限和恢复能力接入 Host App 的边界。
_Avoid_: Agent, Pipeline Plugin

**Executor**:
执行 Node 所引用能力的实现；首期执行语义仅分为 Agent、Action 与 Gate，进程和外部连接都是 Action，子 Pipeline 通过 Graph 组合。
_Avoid_: Node Type, Runtime Adapter

**Capability**:
Node 对执行行为或外部资源提出的抽象需求，由 Workspace 在运行前解析为可用实现。
_Avoid_: Integration Name, Plugin

**Resource Lease**:
Attempt 运行期间对工作目录、端口、环境或部署目标等共享资源的限时占用声明。
_Avoid_: Lock, Permission

**Approval Gate**:
阻止后续 Node 调度、等待明确人工决定的流程控制点。
_Avoid_: Confirmation Dialog, Pause

**Pipeline Authoring Skill**:
Host App 自带的引导式能力，根据用户的实际流程生成、校验并安装符合公开协议的 Pipeline Package。
_Avoid_: Create Pipeline Wizard, Private Generator

**Graph Projection**:
Pipeline 定义、运行轨迹和 Agent 活动的交互式可视化，只负责解释与导航，不是用户编排流程的画布。
_Avoid_: Graph Editor, Low-code Canvas

**Node Focus**:
从 Graph Projection 进入的节点深层观察状态，用于浏览 Node Run、Attempt、Agent Session、Activity、工具活动、变更和 Artifact，同时保留返回原 Graph 位置的连续性。
_Avoid_: Node Page, Detail Modal

**Attention Item**:
需要人提供输入、作出审批或处理异常的可操作事项，可从所有 Pipeline Run 聚合查看并追溯到来源。
_Avoid_: Notification, Alert

**Review Surface**:
围绕一个 Attention Item 汇集问题、上游证据、Artifact、变更、风险与可执行决定的工作界面。
_Avoid_: Modal, Confirmation Dialog

**Change Proposal**:
模型根据用户意图生成的 Pipeline Package 结构化修改，在用户接受前保持为可审查的候选变更。
_Avoid_: Form Edit, Direct Mutation

**Theme Pack**:
在不改变信息结构和状态含义的前提下，为 Host App 提供一组可替换的视觉与动效表达规则。
_Avoid_: Custom CSS, Pipeline Renderer
