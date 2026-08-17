<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ArrowUpRight, Braces, FileText, GitCommitHorizontal, Layers3, PlugZap, ScrollText, TerminalSquare, X } from "@lucide/vue";
import type { Artifact, NodeDefinition, PipelineNode } from "../types";
import StatusMark from "./StatusMark.vue";

const props = withDefaults(defineProps<{ node: PipelineNode; definition: NodeDefinition; artifacts: Artifact[]; busy: boolean; focused?: boolean }>(), { focused: false });
const emit = defineEmits<{ requestChanges: [reason: string]; approve: []; advance: []; focus: []; closeFocus: []; showDefinition: [] }>();
const activeTab = ref<"overview" | "activity" | "artifacts" | "logs">("overview");
const reason = ref("异步退款必须校验幂等键，并补充重复请求回放测试");
const nodeArtifacts = computed(() => props.artifacts.filter((artifact) => artifact.producerNodeId === props.node.id));
watch(() => props.node.id, () => { activeTab.value = "overview"; });
</script>

<template>
  <aside class="inspector">
    <header class="inspector-heading">
      <div><span>节点检查器</span><strong>{{ node.title }}</strong><em>Attempt {{ node.attempt || 1 }}</em></div>
      <button class="icon-button" :aria-label="focused ? '退出 Node Focus' : '进入 Node Focus'" @click="focused ? emit('closeFocus') : emit('focus')"><X v-if="focused" :size="17" /><ArrowUpRight v-else :size="17" /></button>
    </header>
    <nav class="inspector-tabs" role="tablist" aria-label="Node Inspector sections">
      <button v-for="tab in (['overview','activity','artifacts','logs'] as const)" :key="tab" role="tab" :aria-selected="activeTab === tab" :class="{ active: activeTab === tab }" @click="activeTab = tab">
        {{ ({ overview: '概览', activity: 'Activity', artifacts: '产物', logs: 'Logs' } as const)[tab] }}
      </button>
    </nav>

    <div class="inspector-scroll">
      <template v-if="activeTab === 'overview'">
        <section class="inspection-block status-block">
          <h3>状态</h3>
          <div class="status-line"><StatusMark :status="node.status" /><strong>{{ node.status === 'attention' ? '需要确认' : node.status === 'running' ? '正在运行' : node.status === 'completed' ? '已完成' : '等待中' }}</strong><span>{{ node.duration || '--' }}</span></div>
        </section>
        <section class="inspection-block">
          <h3>执行环境</h3>
          <dl><dt>Runtime</dt><dd>{{ node.runtime }}</dd><dt>类型</dt><dd>{{ node.kind }}</dd><dt>Attempt</dt><dd>{{ node.attempt || '尚未创建' }}</dd></dl>
        </section>
        <section class="inspection-block capability-summary">
          <h3>Node 能力</h3>
          <div><span><Braces :size="14" />{{ definition.skillIds.length }} Skills</span><span><PlugZap :size="14" />{{ definition.mcpServers.length }} MCP</span></div>
          <p>{{ definition.capabilities.slice(0, 3).join(' · ') }}<template v-if="definition.capabilities.length > 3"> · +{{ definition.capabilities.length - 3 }}</template></p>
          <button class="secondary-action wide" @click="emit('showDefinition')"><Layers3 :size="15" />查看完整定义与权限</button>
        </section>
        <section v-if="node.id === 'review'" class="inspection-block diff-block">
          <h3>Diff 预览</h3>
          <p>refund_service.rs → refund_service.rs</p>
          <pre><span class="minus">- process_refund(request)</span>
<span class="plus">+ ensure_idempotency(request.key)</span>
<span class="plus">+ replay_completed_refund(request)</span>
  process_refund(request)</pre>
        </section>
        <section v-if="node.status === 'attention' && node.id === 'review'" class="inspection-block decision-block">
          <h3>Review 决定</h3>
          <label for="review-reason">打回原因</label>
          <textarea id="review-reason" v-model="reason" rows="3" />
          <div class="decision-actions">
            <button class="secondary-action" :disabled="busy" @click="emit('requestChanges', reason)">请求修改</button>
            <button class="primary-action" :disabled="busy" @click="emit('approve')">批准并继续</button>
          </div>
        </section>
        <section v-if="node.status === 'running'" class="inspection-block live-block">
          <h3>正在处理</h3>
          <div class="live-pulse"><span></span>{{ node.activities[node.activities.length - 1]?.detail || 'Agent 正在工作' }}</div>
          <button class="secondary-action wide" :disabled="busy" @click="emit('advance')">模拟完成当前工作</button>
        </section>
      </template>

      <section v-else-if="activeTab === 'activity'" class="activity-timeline">
        <div v-for="activity in node.activities" :key="activity.id" class="timeline-row">
          <StatusMark :status="activity.status" />
          <div><header><strong>{{ activity.title }}</strong><time>{{ activity.time }}</time></header><span>{{ activity.detail }}</span></div>
        </div>
        <div v-if="node.activities.length === 0" class="inspector-empty"><Braces :size="22" />节点开始后，模型发布的旁路 Activity 会出现在这里。</div>
      </section>

      <section v-else-if="activeTab === 'artifacts'" class="artifact-list">
        <article v-for="artifact in nodeArtifacts" :key="artifact.id" class="artifact-row">
          <FileText :size="18" /><div><strong>{{ artifact.title }}</strong><span>{{ artifact.mediaType }} · rev {{ artifact.revision }} · {{ artifact.size }}</span><p>{{ artifact.summary }}</p></div>
        </article>
        <div v-if="nodeArtifacts.length === 0" class="inspector-empty"><FileText :size="22" />这个 Node 还没有发布正式 Artifact。</div>
      </section>

      <section v-else class="log-view">
        <div class="log-toolbar"><TerminalSquare :size="15" /> Raw session escape hatch</div>
        <pre><span>11:08:14</span> agent.message  开始检查退款状态机
<span>11:09:02</span> tool.call      <GitCommitHorizontal :size="12" /> git diff --stat
<span>11:10:46</span> tool.result    7 files changed
<span>11:12:03</span> artifact       <ScrollText :size="12" /> Code Review rev 1
<span>11:12:04</span> gate.waiting   human decision required</pre>
      </section>
    </div>
  </aside>
</template>
