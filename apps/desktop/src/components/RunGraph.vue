<script setup lang="ts">
import { computed } from "vue";
import { GitBranch, History } from "@lucide/vue";
import type { PipelineNode } from "../types";
import StatusMark from "./StatusMark.vue";

const props = defineProps<{ nodes: PipelineNode[]; selectedNodeId: string }>();
defineEmits<{ select: [nodeId: string] }>();

const positions: Record<string, { x: number; y: number }> = {
  grill: { x: 34, y: 82 }, ticket: { x: 242, y: 82 }, spec: { x: 450, y: 82 },
  implement: { x: 24, y: 304 }, review: { x: 196, y: 304 }, deploy: { x: 368, y: 304 }, smoke: { x: 540, y: 304 }
};

const position = (id: string) => positions[id] ?? { x: 0, y: 0 };

const implement = computed(() => props.nodes.find((node) => node.id === "implement"));
</script>

<template>
  <section class="graph-scroll" aria-label="Pipeline Graph">
    <div class="graph-stage">
      <svg class="graph-wires" viewBox="0 0 720 650" aria-hidden="true">
        <defs>
          <marker id="arrow-blue" markerWidth="7" markerHeight="7" refX="6" refY="3.5" orient="auto"><path d="M0 0L7 3.5L0 7Z" fill="var(--wire)" /></marker>
          <marker id="arrow-red" markerWidth="7" markerHeight="7" refX="6" refY="3.5" orient="auto"><path d="M0 0L7 3.5L0 7Z" fill="var(--attention)" /></marker>
        </defs>
        <path class="wire" d="M174 135 H242" />
        <path class="wire" d="M382 135 H450" />
        <path class="wire" d="M590 135 H625 Q646 135 646 158 V232 Q646 250 625 250 H94 Q74 250 74 278 V304" />
        <path class="wire" d="M164 360 H196" />
        <path class="wire attention-wire" d="M336 360 H368" />
        <path class="wire attention-wire" d="M508 360 H540" />
        <path class="wire" d="M164 438 H176 Q190 438 190 420 V394 Q190 377 196 377" />
        <path v-if="implement?.attempt === 2" class="wire feedback-wire" d="M266 304 V270 Q266 250 242 250 H112 Q94 250 94 279 V304" />
        <path class="history-wire" d="M164 566 H196 M336 566 H368 M508 566 H540" />
      </svg>

      <button
        v-for="node in nodes"
        :key="node.id"
        class="graph-node"
        :class="[`status-${node.status}`, { selected: node.id === selectedNodeId, expanded: node.id === 'implement' }]"
        :style="{ left: `${position(node.id).x}px`, top: `${position(node.id).y}px` }"
        @click="$emit('select', node.id)"
      >
        <span class="node-topline"><span>{{ String(node.index).padStart(2, "0") }}</span><StatusMark :status="node.status" /></span>
        <strong>{{ node.title }}</strong>
        <span class="node-meta"><span>{{ node.status === "completed" ? `完成${node.attempt > 1 ? ` · Attempt ${node.attempt}` : ""}` : node.status === "attention" ? "需要确认" : node.status === "running" ? `运行中 · Attempt ${node.attempt}` : "等待中" }}</span><time>{{ node.finishedAt || node.startedAt || "--:--" }}</time></span>
        <span v-if="node.id === 'implement'" class="node-activities">
          <span v-for="activity in node.activities" :key="activity.id" class="activity-mini">
            <StatusMark :status="activity.status" :size="11" />
            <span><b>{{ activity.title }}</b><small>{{ activity.detail }}</small></span>
            <time>{{ activity.time }}</time>
          </span>
        </span>
      </button>

      <div class="history-label"><History :size="14" /> 历史 Attempt</div>
      <div v-for="node in nodes.slice(3)" :key="`history-${node.id}`" class="history-node" :style="{ left: `${position(node.id).x + 18}px` }">
        <span>{{ String(node.index).padStart(2, "0") }}</span><strong>{{ node.title }}</strong>
      </div>
      <div class="graph-legend"><GitBranch :size="14" /> Graph 冻结 · Activity 实时投影</div>
    </div>
  </section>
</template>
