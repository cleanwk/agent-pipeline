<!--
THESIS: Agent 长任务是一张可持续移交、可下钻且历史不可擦除的系统图；拒绝普通 Dashboard 与拖拽流程画布。
OWN-WORLD: 暖骨色制图面、石墨分区、蓝图蓝关系线、朱红 Attention；细线、方角控制和半透明描图层。
STORY: 用户先看需要介入之处，再沿 Graph 定位 Node，并在 Inspector 中审查 Activity、Artifact 与恢复动作。
FIRST VIEWPORT: 左侧固定 Attention，中间占主导的七阶段 Graph，右侧固定 Inspector；Review Gate 是唯一朱红焦点。
FORM: 系统制图台，批准构图 A，seed 0098c679。FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, and DESIGN.md
-->
<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { Archive, Boxes, ChevronDown, CircleHelp, FileCode2, FileStack, GitBranch, Moon, MoreHorizontal, PanelTop, Plus, Radar, RotateCcw, Settings, ShieldCheck, Sun, WandSparkles } from "@lucide/vue";
import { bootstrap, dispatch, inspectInstalledPackage, installPackage, startWindowDrag, toggleWindowMaximize } from "./api";
import { demoAgents, demoRun } from "./demo";
import { demoDefinition } from "./definition";
import type { AgentProbe, PipelineDefinition, RunProjection } from "./types";
import AttentionRail from "./components/AttentionRail.vue";
import NodeInspector from "./components/NodeInspector.vue";
import OnboardingWizard from "./components/OnboardingWizard.vue";
import PipelineDefinitionView from "./components/PipelineDefinitionView.vue";
import RunGraph from "./components/RunGraph.vue";

type ThemeName = "system" | "draft" | "night" | "warm";
type MainView = "run" | "definition" | "artifacts" | "author";

const loading = ref(true);
const bootError = ref("");
const busy = ref(false);
const onboarding = ref(localStorage.getItem("agent-pipeline:onboarded") !== "true");
const onboardingStep = ref(1);
const run = ref<RunProjection>(structuredClone(demoRun));
const definition = ref<PipelineDefinition>(structuredClone(demoDefinition));
const agents = ref<AgentProbe[]>(structuredClone(demoAgents));
const native = ref(false);
const selectedNodeId = ref("review");
const theme = ref<ThemeName>((localStorage.getItem("agent-pipeline:theme") as ThemeName) || "system");
const themeOpen = ref(false);
const mainView = ref<MainView>("run");
const nodeFocused = ref(false);
const authorPrompt = ref("为发布流程创建一个 Pipeline：先澄清需求，再生成 Spec、实现、Review；Review 有问题时回到实现，最多 3 轮。最后部署到测试环境并执行 Smoke Test。");
const authorGenerated = ref(false);
const authorPackagePath = ref("~/github/agent-pipeline-example");
const packageInstallState = ref<"idle" | "installing" | "installed" | "error">("idle");
const packageInstallMessage = ref("");
const helpOpen = ref(false);
const moreOpen = ref(false);
const permissionsOpen = ref(false);
const toast = ref("");
const activeDeliverySlot = ref("全部产物");

const selectedNode = computed(() => run.value.nodes.find((node) => node.id === selectedNodeId.value) ?? run.value.nodes[0]!);
const selectedDefinition = computed(() => definition.value.nodes.find((node) => node.nodeId === selectedNodeId.value) ?? definition.value.nodes[0]!);
const completedCount = computed(() => run.value.nodes.filter((node) => node.status === "completed").length);
const themeLabel = computed(() => ({ system: "System", draft: "Draft Light", night: "Night Ops", warm: "Warm Paper" })[theme.value]);

onMounted(loadApp);

async function loadApp() {
  loading.value = true;
  bootError.value = "";
  try {
    const data = await bootstrap();
    run.value = data.run;
    agents.value = data.agents;
    native.value = data.native;
    selectedNodeId.value = data.run.selectedNodeId || "review";
    definition.value = data.definition;
  } catch (error) {
    bootError.value = error instanceof Error ? error.message : String(error);
  } finally {
    loading.value = false;
  }
}

function completeOnboarding() {
  localStorage.setItem("agent-pipeline:onboarded", "true");
  onboarding.value = false;
}

function openDoctor() {
  helpOpen.value = false;
  moreOpen.value = false;
  onboardingStep.value = 3;
  onboarding.value = true;
}

function showDefinition(nodeId = selectedNodeId.value) {
  selectedNodeId.value = nodeId;
  mainView.value = "definition";
}

function editDefinition(nodeId: string) {
  const node = definition.value.nodes.find((candidate) => candidate.nodeId === nodeId);
  authorPrompt.value = `修改 ${definition.value.displayName} 的 ${nodeId} Node：${node?.summary ?? ""}\n保留现有输入输出契约，说明需要调整的 Skill、MCP、Capability 与 Policy。`;
  authorGenerated.value = false;
  mainView.value = "author";
}

function notify(message: string) {
  toast.value = message;
  window.setTimeout(() => { if (toast.value === message) toast.value = ""; }, 2600);
}

function exportSnapshot() {
  const snapshot = { exportedAt: new Date().toISOString(), run: run.value, definition: definition.value };
  const href = URL.createObjectURL(new Blob([JSON.stringify(snapshot, null, 2)], { type: "application/json" }));
  const anchor = document.createElement("a");
  anchor.href = href;
  anchor.download = `${run.value.id}-snapshot.json`;
  anchor.click();
  URL.revokeObjectURL(href);
  notify("Run snapshot 已导出到下载目录");
}

function chooseTheme(next: ThemeName) {
  theme.value = next;
  localStorage.setItem("agent-pipeline:theme", next);
  themeOpen.value = false;
}

async function selectNode(nodeId: string) {
  selectedNodeId.value = nodeId;
  run.value = await dispatch({ selectNode: { nodeId } });
}

async function runCommand(command: Parameters<typeof dispatch>[0]) {
  busy.value = true;
  try {
    run.value = await dispatch(command);
    selectedNodeId.value = run.value.selectedNodeId || selectedNodeId.value;
  } catch (error) {
    notify(`Runner command failed: ${error instanceof Error ? error.message : String(error)}`);
  } finally {
    busy.value = false;
  }
}

async function resetDemo() {
  await runCommand({ resetDemo: {} });
  selectedNodeId.value = "review";
  mainView.value = "run";
}

async function installProposal() {
  packageInstallState.value = "installing";
  packageInstallMessage.value = "正在验证 Graph、文件引用和有界循环…";
  try {
    const installed = await installPackage(authorPackagePath.value);
    definition.value = await inspectInstalledPackage(installed.name, installed.version);
    packageInstallState.value = "installed";
    packageInstallMessage.value = `${installed.displayName} ${installed.version} 已安装 · ${installed.pipelineCount} Pipeline`;
  } catch (error) {
    packageInstallState.value = "error";
    packageInstallMessage.value = error instanceof Error ? error.message : String(error);
  }
}
</script>

<template>
  <div v-if="loading" class="boot-screen"><Radar class="spin" :size="28" /><span>正在连接 Local Runner</span></div>
  <div v-else-if="bootError" class="boot-screen boot-error"><Radar :size="28" /><strong>Local Runner 暂时不可用</strong><span>{{ bootError }}</span><button class="primary-action" @click="loadApp">重新连接</button></div>
  <OnboardingWizard v-else-if="onboarding" :key="onboardingStep" :agents="agents" :native="native" :initial-step="onboardingStep" @complete="completeOnboarding" />
  <div v-else class="app-shell" :data-theme="theme">
    <header class="app-titlebar" data-tauri-drag-region @mousedown.left="startWindowDrag" @dblclick="toggleWindowMaximize">
      <div class="drag-spacer"></div><strong>Agent Pipeline</strong>
      <div class="titlebar-actions" @mousedown.stop @dblclick.stop><span :class="['runner-indicator', { native }]">{{ native ? 'Local Runner' : 'Preview mode' }}</span><button class="icon-button" aria-label="重新运行 Doctor" title="重新运行 Doctor" @click="openDoctor"><Settings :size="16" /></button><div class="popover-control"><button class="icon-button" aria-label="帮助" :aria-expanded="helpOpen" @click="helpOpen = !helpOpen; moreOpen = false"><CircleHelp :size="16" /></button><section v-if="helpOpen" class="utility-popover help-popover"><strong>Agent Pipeline 快速帮助</strong><p>先处理 Attention，再从 Graph 下钻 Node。Definition 解释每个阶段获得的 Skill、MCP 与权限。</p><dl><dt>Graph</dt><dd>运行状态与子任务</dd><dt>Definition</dt><dd>冻结的插件协议与能力</dd><dt>Node Focus</dt><dd>完整会话、产物与日志</dd></dl><span>全部数据留在本机；Agent 是否联网由各自 Client 决定。</span></section></div></div>
    </header>
    <nav class="command-bar">
      <div class="run-title"><button class="brand-button" aria-label="Mission Control" @click="mainView = 'run'"><Radar :size="18" /></button><div><h1>{{ run.title }} <span :class="`run-state state-${run.status}`">{{ run.status === 'attention' ? '等待确认' : run.status === 'running' ? '运行中' : '已完成' }}</span></h1><p>Run ID: {{ run.id }}　开始时间: {{ run.startedAt }}　运行时长: {{ run.elapsed }}</p></div></div>
      <div class="command-actions">
        <button :class="{ active: mainView === 'run' }" @click="mainView = 'run'"><GitBranch :size="15" />Graph</button>
        <button :class="{ active: mainView === 'definition' }" @click="mainView = 'definition'"><FileCode2 :size="15" />Definition</button>
        <button :class="{ active: mainView === 'artifacts' }" @click="mainView = 'artifacts'"><FileStack :size="15" />Deliverables <span>{{ run.artifacts.length }}</span></button>
        <button :class="{ active: mainView === 'author' }" @click="mainView = 'author'"><WandSparkles :size="15" />Create Preview</button>
        <div class="theme-control"><button aria-haspopup="menu" :aria-expanded="themeOpen" @click="themeOpen = !themeOpen"><Sun v-if="theme === 'draft' || theme === 'warm'" :size="15" /><Moon v-else-if="theme === 'night'" :size="15" /><PanelTop v-else :size="15" />{{ themeLabel }}<ChevronDown :size="13" /></button><div v-if="themeOpen" class="theme-menu" role="menu"><button v-for="choice in (['system','draft','night','warm'] as ThemeName[])" :key="choice" role="menuitemradio" :aria-checked="theme === choice" @click="chooseTheme(choice)"><span :class="`theme-swatch swatch-${choice}`"></span>{{ ({ system: 'System · 跟随 macOS', draft: 'Draft Light', night: 'Night Ops', warm: 'Warm Paper' } as const)[choice] }}</button></div></div>
        <div class="popover-control"><button class="icon-button" aria-label="更多" :aria-expanded="moreOpen" @click="moreOpen = !moreOpen; helpOpen = false"><MoreHorizontal :size="18" /></button><section v-if="moreOpen" class="utility-popover more-popover"><button @click="showDefinition(); moreOpen = false"><FileCode2 :size="15" />查看 Pipeline 定义</button><button @click="openDoctor"><Settings :size="15" />Environment Doctor</button><button @click="resetDemo(); moreOpen = false"><RotateCcw :size="15" />重置演示 Run</button></section></div>
      </div>
    </nav>

    <main v-if="mainView === 'run'" class="run-workspace" :class="{ 'node-focus': nodeFocused }">
      <AttentionRail :items="run.attention" :selected-node-id="selectedNodeId" @select="selectNode" />
      <section class="graph-column">
        <div class="run-context"><span><strong>{{ completedCount }}/{{ run.nodes.length }}</strong> 大节点完成</span><p>{{ run.brief }}</p><button @click="resetDemo"><RotateCcw :size="14" />重置示例</button></div>
        <RunGraph :nodes="run.nodes" :edges="definition.edges" :selected-node-id="selectedNodeId" @select="selectNode" />
        <footer class="run-statusbar"><span><Boxes :size="14" />{{ run.workspace }}</span><span><GitBranch :size="14" />{{ run.branch }}</span><span>{{ run.eventCount }} events · SQLite durable</span></footer>
      </section>
      <NodeInspector :node="selectedNode" :definition="selectedDefinition" :artifacts="run.artifacts" :busy="busy" :focused="nodeFocused" @focus="nodeFocused = true" @close-focus="nodeFocused = false" @show-definition="showDefinition(selectedNode.id)" @request-changes="reason => runCommand({ requestChanges: { nodeId: 'review', reason } })" @approve="runCommand({ approve: { nodeId: 'review' } })" @advance="runCommand({ advance: {} })" />
    </main>

    <PipelineDefinitionView v-else-if="mainView === 'definition'" :definition="definition" :nodes="run.nodes" :selected-node-id="selectedNodeId" @select="selectNode" @edit="editDefinition" />

    <main v-else-if="mainView === 'artifacts'" class="deliverables-view">
      <header><div><h2>Run Deliverables</h2><p>正式发布的 Artifact revision。日志、临时文件和完整 transcript 不会混入交付物。</p></div><button class="secondary-action" @click="exportSnapshot"><Archive :size="15" />导出 Run snapshot</button></header>
      <div class="delivery-layout"><aside><h3>Delivery Slots</h3><button v-for="group in ['全部产物','需求与决策','技术方案','实现与验证','部署与冒烟']" :key="group" :class="{ active: activeDeliverySlot === group }" @click="activeDeliverySlot = group">{{ group }}<span>{{ group === '全部产物' ? run.artifacts.length : group === '需求与决策' ? 2 : group === '技术方案' ? 1 : group === '实现与验证' ? 3 : 0 }}</span></button></aside><section class="deliverable-table"><article v-for="artifact in run.artifacts" v-show="activeDeliverySlot === '全部产物' || (activeDeliverySlot === '需求与决策' && ['grill','ticket'].includes(artifact.producerNodeId)) || (activeDeliverySlot === '技术方案' && artifact.producerNodeId === 'spec') || (activeDeliverySlot === '实现与验证' && ['implement','review'].includes(artifact.producerNodeId)) || (activeDeliverySlot === '部署与冒烟' && ['deploy','smoke'].includes(artifact.producerNodeId))" :key="artifact.id"><span class="file-mark"><FileStack :size="19" /></span><div><h3>{{ artifact.title }} <em>rev {{ artifact.revision }}</em></h3><p>{{ artifact.summary }}</p><span>{{ artifact.mediaType }} · {{ artifact.size }} · {{ artifact.createdAt }}</span></div><div class="artifact-origin"><span>{{ run.nodes.find(n => n.id === artifact.producerNodeId)?.title }}</span><strong>Attempt {{ artifact.producerAttempt }}</strong></div></article><div v-if="activeDeliverySlot === '部署与冒烟'" class="delivery-empty">此阶段尚未发布 Artifact；完成 Deploy 与 Smoke Test 后会自动归档。</div></section></div>
    </main>

    <main v-else class="author-view">
      <section class="author-intent"><span class="author-symbol"><WandSparkles :size="26" /></span><h2>用自然语言设计 Pipeline</h2><p><strong>MVP 交互预览：</strong>这里演示未来由本地模型定义 Graph、节点契约、Skill、MCP 与 Policy 的审查体验；当前版本不会调用模型或写入 Package。</p><label><span>描述团队实际怎么工作</span><textarea v-model="authorPrompt" rows="7" /></label><button class="primary-action" @click="authorGenerated = true"><WandSparkles :size="16" />预览 Package Proposal</button></section>
      <section class="proposal-preview" :class="{ empty: !authorGenerated }">
        <template v-if="authorGenerated">
          <header><div><strong>release-pipeline</strong><span>{{ packageInstallState === 'installed' ? 'Existing source installed' : 'Design prototype · not generated' }}</span></div><button class="secondary-action" :aria-expanded="permissionsOpen" @click="permissionsOpen = !permissionsOpen"><ShieldCheck :size="15" />审查 6 项权限</button></header>
          <div v-if="permissionsOpen" class="permission-review"><strong>新增权限</strong><span>filesystem.read · filesystem.write(worktree)</span><span>git.diff · git.commit</span><span>mcp.requirements.read · mcp.ocm.deploy</span><p>安装前按 Node 隔离授权；密钥值不会写入 Package。</p></div>
          <div class="proposal-graph"><span>Grill</span><i>→</i><span>Spec</span><i>→</i><span>Implement</span><i>↔</i><span class="attention-outline">Review</span><i>→</i><span>Deploy</span><i>→</i><span>Smoke</span></div>
          <div class="proposal-files">
            <h3>将创建</h3><p>pipeline.yaml · prompts/*.md · schemas/*.json · tests/scenarios.yaml · agent-pipeline.lock</p>
            <h3>示例预检查（非实际执行）</h3><p class="valid">✓ Graph schema　✓ Review loop max_iterations: 3　✓ Output contracts　✓ Capability resolution</p>
            <label class="package-source"><span>Package source</span><input v-model="authorPackagePath" /></label>
            <p v-if="packageInstallMessage" :class="['install-result', packageInstallState]">{{ packageInstallMessage }}</p>
          </div>
          <footer><button class="secondary-action" @click="notify(`Existing source: ${authorPackagePath}`)">显示现有 Source 路径</button><button class="primary-action" :disabled="packageInstallState === 'installing'" @click="installProposal">{{ packageInstallState === 'installing' ? 'Validating…' : packageInstallState === 'installed' ? 'Installed' : 'Validate & Install Existing Source' }}</button></footer>
        </template>
        <template v-else><Plus :size="34" /><strong>Design prototype 会在这里出现</strong><span>当前用于验证审查体验；本地模型生成 Package 的执行链路尚未接入。</span></template>
      </section>
    </main>
    <div class="app-toast" role="status" aria-live="polite" :class="{ visible: toast }">{{ toast }}</div>
  </div>
</template>
