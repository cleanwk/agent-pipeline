<script setup lang="ts">
import { computed } from "vue";
import { GitBranch, History } from "@lucide/vue";
import type { PipelineDefinition, PipelineNode } from "../types";
import StatusMark from "./StatusMark.vue";

const props = defineProps<{ nodes: PipelineNode[]; edges: PipelineDefinition["edges"]; selectedNodeId: string }>();
defineEmits<{ select: [nodeId: string] }>();

const positions = computed(() => {
  const map: Record<string, { x: number; y: number }> = {};
  props.nodes.forEach((node, index) => {
    const column = index % 4;
    const row = Math.floor(index / 4);
    map[node.id] = { x: 30 + column * 170, y: 70 + row * 190 };
  });
  return map;
});
const position = (id: string) => positions.value[id] ?? { x: 30, y: 78 };
const selected = computed(() => props.nodes.find((node) => node.id === props.selectedNodeId) ?? props.nodes[0]);
const rowCount = computed(() => Math.max(1, Math.ceil(props.nodes.length / 4)));
const activityTop = computed(() => 70 + rowCount.value * 190);
const historyTop = computed(() => activityTop.value + 142);
const stageHeight = computed(() => historyTop.value + 126);
const edgePath = (fromId: string, toId: string) => {
  const from = position(fromId); const to = position(toId);
  const startX = from.x + 140; const startY = from.y + 58; const endX = to.x; const endY = to.y + 58;
  if (from.y === to.y && to.x < from.x) return `M${from.x + 70} ${from.y} V${from.y - 28} H${to.x + 70} V${to.y}`;
  if (from.y === to.y) return `M${startX} ${startY} H${endX}`;
  return `M${startX} ${startY} H680 Q696 ${startY} 696 ${startY + 16} V${endY - 16} Q696 ${endY} 680 ${endY} H${endX}`;
};
</script>

<template>
  <section class="graph-scroll" aria-label="Pipeline Graph">
    <div class="graph-stage" :style="{ height: `${stageHeight}px`, '--activity-top': `${activityTop}px`, '--history-top': `${historyTop}px` }">
      <svg class="graph-wires" :viewBox="`0 0 720 ${stageHeight}`" :style="{ height: `${stageHeight}px` }" aria-hidden="true">
        <defs>
          <marker id="arrow-blue" markerWidth="7" markerHeight="7" refX="6" refY="3.5" orient="auto"><path d="M0 0L7 3.5L0 7Z" fill="var(--wire)" /></marker>
          <marker id="arrow-red" markerWidth="7" markerHeight="7" refX="6" refY="3.5" orient="auto"><path d="M0 0L7 3.5L0 7Z" fill="var(--attention)" /></marker>
        </defs>
        <path v-for="edge in edges" :key="`edge-${edge.from}-${edge.to}-${edge.when || 'always'}`" :class="['wire', { 'attention-wire': edge.when, 'feedback-wire': edge.loop }]" :d="edgePath(edge.from, edge.to)" />
      </svg>

      <button
        v-for="node in nodes"
        :key="node.id"
        class="graph-node"
        :class="[`status-${node.status}`, { selected: node.id === selectedNodeId }]"
        :style="{ left: `${position(node.id).x}px`, top: `${position(node.id).y}px` }"
        @click="$emit('select', node.id)"
      >
        <span class="node-topline"><span>{{ String(node.index).padStart(2, "0") }}</span><StatusMark :status="node.status" /></span>
        <strong>{{ node.title }}</strong>
        <span class="node-meta"><span>{{ node.status === "completed" ? `完成${node.attempt > 1 ? ` · Attempt ${node.attempt}` : ""}` : node.status === "attention" ? "需要确认" : node.status === "running" ? `运行中 · Attempt ${node.attempt}` : "等待中" }}</span><time>{{ node.finishedAt || node.startedAt || "--:--" }}</time></span>
      </button>

      <section class="graph-activity-lane" aria-label="Selected node subtasks">
        <header><span>NODE ACTIVITY</span><strong>{{ selected?.title }}</strong><small>{{ selected?.activities.length || 0 }} subtasks</small></header>
        <div v-if="selected?.activities.length" class="activity-lane-items"><article v-for="activity in selected.activities" :key="activity.id"><StatusMark :status="activity.status" :size="11" /><span><b>{{ activity.title }}</b><small>{{ activity.detail }}</small></span><time>{{ activity.time }}</time></article></div>
        <p v-else>节点运行后，模型通过旁路 Schema 发布的子任务会投影到这里。</p>
      </section>
      <div class="history-label"><History :size="14" /> 历史 Attempt</div>
      <div v-for="(node, index) in nodes.slice(3, 7)" :key="`history-${node.id}`" class="history-node" :style="{ left: `${32 + index * 160}px` }">
        <span>{{ String(node.index).padStart(2, "0") }}</span><strong>{{ node.title }}</strong>
      </div>
      <div class="graph-legend"><GitBranch :size="14" /> Graph 冻结 · Activity 实时投影</div>
    </div>
  </section>
</template>
