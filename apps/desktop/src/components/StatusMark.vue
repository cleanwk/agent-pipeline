<script setup lang="ts">
import { AlertCircle, Check, Circle, LoaderCircle, XCircle } from "@lucide/vue";
import type { NodeStatus } from "../types";

defineProps<{ status: NodeStatus; size?: number }>();
</script>

<template>
  <span class="status-mark" :class="`is-${status}`" :aria-label="status">
    <Check v-if="status === 'completed'" :size="size || 13" :stroke-width="2.8" />
    <AlertCircle v-else-if="status === 'attention'" :size="size || 15" :stroke-width="2.4" />
    <LoaderCircle v-else-if="status === 'running'" class="spin" :size="size || 15" :stroke-width="2.4" />
    <XCircle v-else-if="status === 'failed'" :size="size || 15" :stroke-width="2.4" />
    <Circle v-else :size="size || 15" :stroke-width="1.8" />
  </span>
</template>
