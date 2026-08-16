<script setup lang="ts">
import { ListFilter, Radio } from "@lucide/vue";
import type { AttentionItem } from "../types";

defineProps<{ items: AttentionItem[]; selectedNodeId: string }>();
defineEmits<{ select: [nodeId: string] }>();
</script>

<template>
  <aside class="attention-rail">
    <header class="rail-heading">
      <h2>注意事项</h2>
      <button class="icon-button" aria-label="筛选注意事项"><ListFilter :size="17" /></button>
    </header>
    <div class="attention-list">
      <button
        v-for="item in items"
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
      <div v-if="items.length === 0" class="attention-empty">
        <span class="quiet-check">✓</span>
        <strong>当前无需介入</strong>
        <span>Pipeline 正按策略继续执行</span>
      </div>
    </div>
    <footer class="rail-footer"><ListFilter :size="15" /> {{ items.length }} 项注意事项</footer>
  </aside>
</template>
