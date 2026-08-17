<script setup lang="ts">
import { computed, ref } from "vue";
import { ListFilter, Radio } from "@lucide/vue";
import type { AttentionItem } from "../types";

const props = defineProps<{ items: AttentionItem[]; selectedNodeId: string }>();
defineEmits<{ select: [nodeId: string] }>();
const criticalOnly = ref(false);
const visibleItems = computed(() => criticalOnly.value ? props.items.filter((item) => item.severity === "critical") : props.items);
</script>

<template>
  <aside class="attention-rail">
    <header class="rail-heading">
      <h2>注意事项</h2>
      <button class="icon-button" aria-label="只看需要处理的事项" :aria-pressed="criticalOnly" :title="criticalOnly ? '显示全部' : '只看需要处理'" @click="criticalOnly = !criticalOnly"><ListFilter :size="17" /></button>
    </header>
    <div class="attention-list">
      <button
        v-for="item in visibleItems"
        :key="item.id"
        class="attention-row"
        :class="[{ selected: item.nodeId === selectedNodeId }, `severity-${item.severity}`]"
        @click="$emit('select', item.nodeId)"
      >
        <span class="attention-dot"><Radio :size="14" /></span>
        <span class="attention-copy">
          <strong>{{ item.title }}</strong>
          <span>{{ item.detail }}</span>
        </span>
        <time>{{ item.time }}</time>
      </button>
      <div v-if="visibleItems.length === 0" class="attention-empty">
        <span class="quiet-check">✓</span>
        <strong>当前无需介入</strong>
        <span>Pipeline 正按策略继续执行</span>
      </div>
    </div>
    <footer class="rail-footer"><ListFilter :size="15" />{{ criticalOnly ? `需处理 ${visibleItems.length} / ${items.length}` : `${items.length} 项注意事项` }}</footer>
  </aside>
</template>
