<script setup lang="ts">
import { computed, ref } from "vue";
import { ArrowRight, Check, ChevronRight, Cpu, Database, FolderGit2, HardDrive, Radar, ShieldCheck } from "@lucide/vue";
import type { AgentProbe } from "../types";

const props = withDefaults(defineProps<{ agents: AgentProbe[]; native: boolean; initialStep?: number }>(), { initialStep: 1 });
const emit = defineEmits<{ complete: [] }>();
const step = ref(props.initialStep);
const workspace = ref("/Users/kai/github/pipeline");
const readyCount = computed(() => props.agents.filter((agent) => agent.state === "ready").length);
</script>

<template>
  <main class="onboarding-shell">
    <div class="onboarding-titlebar" data-tauri-drag-region>Agent Pipeline</div>
    <aside class="onboarding-index">
      <div class="brand-lockup"><span class="brand-mark"><Radar :size="20" /></span><strong>Agent Pipeline</strong></div>
      <ol>
        <li v-for="item in ['本地边界','机器检查','发现 Agent','Workspace','准备完成']" :key="item" :class="{ active: step === Number(Object.keys(['本地边界','机器检查','发现 Agent','Workspace','准备完成']).find(k => ['本地边界','机器检查','发现 Agent','Workspace','准备完成'][Number(k)] === item)) + 1, done: step > ['本地边界','机器检查','发现 Agent','Workspace','准备完成'].indexOf(item) + 1 }">
          <span>{{ step > ['本地边界','机器检查','发现 Agent','Workspace','准备完成'].indexOf(item) + 1 ? '✓' : ['本地边界','机器检查','发现 Agent','Workspace','准备完成'].indexOf(item) + 1 }}</span>{{ item }}
        </li>
      </ol>
      <div class="local-promise"><ShieldCheck :size="17" /><span><strong>0 Cloud · 0 Telemetry</strong>App 数据只留在这台 Mac</span></div>
    </aside>
    <section class="onboarding-work">
      <div v-if="step === 1" class="onboarding-panel">
        <span class="panel-symbol"><ShieldCheck :size="32" /></span>
        <h1>先把边界说清楚</h1>
        <p>Agent Pipeline 只在本机保存 Workspace、Run、Session 索引、Artifact 与事件。Host App 不包含云端同步、遥测或崩溃上报。</p>
        <div class="boundary-lines">
          <div><Database :size="18" /><span><strong>本地持久化</strong>SQLite 与 Artifact snapshot 位于 App Data</span><Check :size="17" /></div>
          <div><HardDrive :size="18" /><span><strong>用户自己的 Agent</strong>认证仍由 Pi、Codex、Claude Code 与 OpenCode 持有</span><Check :size="17" /></div>
          <div><ShieldCheck :size="18" /><span><strong>网络显式授权</strong>MCP 与业务平台按 Node 能力和 Policy 暴露</span><Check :size="17" /></div>
        </div>
      </div>
      <div v-else-if="step === 2" class="onboarding-panel">
        <span class="panel-symbol"><Cpu :size="32" /></span><h1>这台 Mac 可以运行</h1><p>首发只支持 Apple Silicon 与 macOS 14 及以上。所有探测均为只读，不会修改 Shell 配置。</p>
        <div class="host-report"><div><span>Architecture</span><strong>arm64</strong><Check :size="17" /></div><div><span>Host</span><strong>{{ native ? 'Tauri native' : 'Browser preview' }}</strong><Check :size="17" /></div><div><span>Data directory</span><strong>可写</strong><Check :size="17" /></div></div>
      </div>
      <div v-else-if="step === 3" class="onboarding-panel agent-scan-panel">
        <span class="panel-symbol"><Radar :size="32" /></span><h1>发现 {{ readyCount }} 个 Agent Runtime</h1><p>安装、版本、认证与协议能力分开检查。这里不发送模型请求，也不会产生费用。</p>
        <div class="agent-probes">
          <article v-for="agent in agents" :key="agent.id" class="agent-probe"><span class="agent-monogram">{{ agent.name.slice(0, 2) }}</span><div><strong>{{ agent.name }}</strong><span>{{ agent.path || '未发现 binary' }}</span><small>{{ agent.capability || `${agent.transport} · ${agent.version || 'version unavailable'}` }}</small></div><em :class="agent.state">{{ agent.state === 'ready' ? 'Ready' : agent.state === 'degraded' ? 'Degraded' : 'Missing' }}</em></article>
        </div>
      </div>
      <div v-else-if="step === 4" class="onboarding-panel">
        <span class="panel-symbol"><FolderGit2 :size="32" /></span><h1>选择第一个 Workspace</h1><p>Workspace 是项目、能力目录与默认 Policy 的稳定边界；每次 Run 会冻结它的环境快照。</p>
        <label class="workspace-field"><span>Repository root</span><input v-model="workspace" /><button aria-label="使用当前示例目录" title="MVP 使用已验证的本地示例目录" @click.prevent="workspace = '~/github/agent-pipeline-example'">…</button></label>
        <div class="workspace-summary"><span>默认 Runtime</span><strong>Pi · RPC</strong><span>Execution sandbox</span><strong>Existing repo，可按 Node 切 worktree</strong></div>
      </div>
      <div v-else class="onboarding-panel ready-panel">
        <span class="panel-symbol success"><Check :size="34" /></span><h1>启动台已经准备好</h1><p>接下来选择 Workspace、Agent Plugin 与 Runtime，再用自然语言描述 Mission。Graph 只在启动后显示运行事实。</p>
        <div class="ready-route"><span>Workspace</span><ChevronRight :size="15" /><span>Agent Plugin</span><ChevronRight :size="15" /><span>Runtime</span><ChevronRight :size="15" /><span class="highlight">Mission</span></div>
      </div>
      <footer class="onboarding-actions"><span>{{ step }} / 5</span><button v-if="step > 1" class="text-action" @click="step--">返回</button><button class="primary-action" @click="step < 5 ? step++ : emit('complete')">{{ step < 5 ? '继续' : '返回启动台' }}<ArrowRight :size="16" /></button></footer>
    </section>
  </main>
</template>
