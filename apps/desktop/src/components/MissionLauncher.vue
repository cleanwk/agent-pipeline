<script setup lang="ts">
import { computed, ref } from "vue";
import {
  AlertTriangle,
  ArrowRight,
  Bot,
  Braces,
  Check,
  FolderGit2,
  PackageOpen,
  PlugZap,
  Plus,
  Settings,
  ShieldCheck,
} from "@lucide/vue";
import type { AgentProbe, PipelineDefinition } from "../types";

export interface MissionLaunchRequest {
  workspace: string;
  packageName: string;
  packageVersion: string;
  runtimeId: AgentProbe["id"];
  task: string;
}

const props = defineProps<{
  definitions: PipelineDefinition[];
  agents: AgentProbe[];
  native: boolean;
  busy: boolean;
  installState: "idle" | "installing" | "installed" | "error";
  installMessage: string;
}>();

const emit = defineEmits<{
  launch: [request: MissionLaunchRequest];
  install: [sourcePath: string];
  doctor: [];
}>();

const savedWorkspace = localStorage.getItem("agent-pipeline:last-workspace") || "~/github/agent-pipeline-example";
const workspace = ref(savedWorkspace);
const task = ref("检查当前仓库，理解目标与约束，制定计划并完成任务；在关键副作用前请求确认。");
const packageKey = ref(props.definitions[0] ? `${props.definitions[0].packageName}@${props.definitions[0].version}` : "");
const runtimeId = ref<AgentProbe["id"]>(props.agents.find((agent) => agent.state === "ready")?.id || "pi");
const installOpen = ref(false);
const packageSource = ref("~/github/agent-pipeline-example");

const selectedDefinition = computed(() => props.definitions.find(
  (candidate) => `${candidate.packageName}@${candidate.version}` === packageKey.value,
) ?? props.definitions[0]);
const selectedRuntime = computed(() => props.agents.find((agent) => agent.id === runtimeId.value));
const savedWorkspaceName = computed(() => {
  const parts = savedWorkspace.split("/").filter(Boolean);
  return parts[parts.length - 1] || "Workspace";
});
const skillCount = computed(() => new Set(selectedDefinition.value?.nodes.flatMap((node) => node.skillIds) ?? []).size);
const mcpCount = computed(() => new Set(selectedDefinition.value?.nodes.flatMap((node) => node.mcpServers.map((mcp) => mcp.name)) ?? []).size);
const canOpenDemo = computed(() => Boolean(workspace.value.trim() && task.value.trim() && selectedDefinition.value && selectedRuntime.value));

function chooseWorkspace(path: string) {
  workspace.value = path;
}

function launch() {
  const definition = selectedDefinition.value;
  if (!definition || !canOpenDemo.value) return;
  localStorage.setItem("agent-pipeline:last-workspace", workspace.value.trim());
  emit("launch", {
    workspace: workspace.value.trim(),
    packageName: definition.packageName,
    packageVersion: definition.version,
    runtimeId: runtimeId.value,
    task: task.value.trim(),
  });
}
</script>

<template>
  <main class="mission-launcher">
    <header class="launcher-heading">
      <div>
        <h1>从 Workspace 开始</h1>
        <p>选择 Agent 工作目录与能力包，再交给 Runtime。Graph 只在 Mission 启动后显示实际运行投影。</p>
      </div>
      <button class="secondary-action" @click="emit('doctor')"><Settings :size="15" />环境 Doctor</button>
    </header>

    <div class="launcher-body">
      <section class="launcher-form" aria-label="创建 Mission">
        <section class="launcher-section workspace-section">
          <header><FolderGit2 :size="18" /><div><h2>Workspace</h2><p>Agent 的文件、Git 与执行边界</p></div><span class="section-state"><Check :size="13" />本地目录</span></header>
          <label class="launcher-field">
            <span>Repository root</span>
            <input v-model="workspace" autocomplete="off" spellcheck="false" placeholder="/Users/you/github/project" />
          </label>
          <div class="workspace-recents" aria-label="最近 Workspace">
            <button :class="{ active: workspace === savedWorkspace }" @click="chooseWorkspace(savedWorkspace)"><FolderGit2 :size="14" /><span><strong>{{ savedWorkspaceName }}</strong><small>{{ savedWorkspace }}</small></span></button>
            <button :class="{ active: workspace === '~/github/pipeline' }" @click="chooseWorkspace('~/github/pipeline')"><FolderGit2 :size="14" /><span><strong>pipeline</strong><small>~/github/pipeline</small></span></button>
          </div>
        </section>

        <section class="launcher-section plugin-section">
          <header><PackageOpen :size="18" /><div><h2>Agent Plugin</h2><p>Skills、Tools、产物合同与权限声明</p></div><button class="text-action" @click="installOpen = !installOpen"><Plus :size="14" />安装本地 Package</button></header>
          <div v-if="definitions.length" class="plugin-list">
            <button
              v-for="candidate in definitions"
              :key="`${candidate.packageName}@${candidate.version}`"
              :class="{ active: packageKey === `${candidate.packageName}@${candidate.version}` }"
              @click="packageKey = `${candidate.packageName}@${candidate.version}`"
            >
              <span class="plugin-mark"><PackageOpen :size="18" /></span>
              <span class="plugin-copy"><strong>{{ candidate.displayName }}</strong><small>{{ candidate.packageName }} · v{{ candidate.version }}</small></span>
              <span class="plugin-facts"><em><Braces :size="12" />{{ new Set(candidate.nodes.flatMap(node => node.skillIds)).size }} Skills</em><em><PlugZap :size="12" />{{ new Set(candidate.nodes.flatMap(node => node.mcpServers.map(mcp => mcp.name))).size }} MCP</em></span>
              <Check v-if="packageKey === `${candidate.packageName}@${candidate.version}`" class="selected-check" :size="16" />
            </button>
          </div>
          <div v-else class="launcher-empty"><PackageOpen :size="20" /><span><strong>还没有可用的 Plugin</strong>安装一个本地 Package 后才能创建 Mission。</span></div>
          <div v-if="installOpen" class="inline-installer">
            <label><span>Package source</span><input v-model="packageSource" spellcheck="false" /></label>
            <button class="secondary-action" :disabled="installState === 'installing'" @click="emit('install', packageSource)">{{ installState === 'installing' ? '验证中…' : '验证并安装' }}</button>
            <p v-if="installMessage" :class="installState">{{ installMessage }}</p>
          </div>
        </section>

        <section class="launcher-section runtime-section">
          <header><Bot :size="18" /><div><h2>Agent Runtime</h2><p>使用本机已经安装并认证的 Agent</p></div></header>
          <div class="runtime-list">
            <button v-for="agent in agents" :key="agent.id" :disabled="agent.state === 'missing'" :class="{ active: runtimeId === agent.id, degraded: agent.state === 'degraded' }" @click="runtimeId = agent.id">
              <span class="runtime-monogram">{{ agent.name.slice(0, 2) }}</span>
              <span><strong>{{ agent.name }}</strong><small>{{ agent.capability || `${agent.transport} · ${agent.version || 'version unavailable'}` }}</small></span>
              <em :class="agent.state">{{ agent.state === 'ready' ? 'Ready' : agent.state === 'degraded' ? 'Degraded' : 'Missing' }}</em>
            </button>
          </div>
        </section>

        <section class="launcher-section task-section">
          <header><Bot :size="18" /><div><h2>Mission</h2><p>描述目标，不需要预先画流程图</p></div></header>
          <label class="launcher-field"><span>希望 Agent 完成什么？</span><textarea v-model="task" rows="5" /></label>
        </section>
      </section>

      <aside class="launch-review">
        <div class="review-heading"><ShieldCheck :size="21" /><div><strong>启动前检查</strong><span>Mission Snapshot</span></div></div>
        <dl>
          <div><dt>Workspace</dt><dd>{{ workspace || '尚未选择' }}</dd></div>
          <div><dt>Plugin</dt><dd>{{ selectedDefinition?.displayName || '尚未选择' }}<small v-if="selectedDefinition">{{ selectedDefinition.packageName }}@{{ selectedDefinition.version }}</small></dd></div>
          <div><dt>Runtime</dt><dd>{{ selectedRuntime?.name || '尚未选择' }}<small>{{ selectedRuntime?.transport }}</small></dd></div>
          <div><dt>能力面</dt><dd>{{ skillCount }} Skills · {{ mcpCount }} MCP<small>{{ selectedDefinition?.nodes.length || 0 }} 个声明边界</small></dd></div>
        </dl>
        <div class="demo-disclosure"><AlertTriangle :size="17" /><p><strong>当前为 Demo Adapter</strong>这次打开的是明确标记的合成 Run，用于验证 Mission Control；真实 Runtime 尚未执行 Prompt，不会产生模型费用。</p></div>
        <div class="local-boundary"><ShieldCheck :size="15" /><span><strong>Local-first</strong>Workspace、Snapshot 与运行事件保存在本机。</span></div>
        <button class="launch-button" :disabled="!canOpenDemo || busy" @click="launch">
          <span><strong>{{ busy ? '正在准备…' : '打开 Demo Mission' }}</strong><small>冻结选择并进入运行投影</small></span><ArrowRight :size="18" />
        </button>
        <p v-if="!native" class="preview-note">Browser preview 不会访问本机文件或 Agent。</p>
      </aside>
    </div>
  </main>
</template>
