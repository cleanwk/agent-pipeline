<script setup lang="ts">
import { computed } from "vue";
import { ArrowRight, Bot, Braces, Check, FileCode2, GitBranch, KeyRound, PackageCheck, PlugZap, ShieldCheck, Sparkles } from "@lucide/vue";
import type { PipelineDefinition, PipelineNode } from "../types";

const props = defineProps<{ definition: PipelineDefinition; nodes: PipelineNode[]; selectedNodeId: string }>();
const emit = defineEmits<{ select: [nodeId: string]; edit: [nodeId: string] }>();
const selected = computed(() => props.definition.nodes.find((node) => node.nodeId === props.selectedNodeId) ?? props.definition.nodes[0]!);
const runtime = computed(() => props.nodes.find((node) => node.id === selected.value.nodeId)?.runtime ?? "Resolved at run start");
</script>

<template>
  <main class="definition-view">
    <header class="definition-header">
      <div>
        <span class="definition-symbol"><FileCode2 :size="22" /></span>
        <div><h2>Pipeline Definition</h2><p>查看这个 Run 冻结的 Package、Graph、Node 契约与能力绑定；修改交给模型生成新的版本。</p></div>
      </div>
      <dl>
        <div><dt>Package</dt><dd>{{ definition.packageName }}@{{ definition.version }}</dd></div>
        <div><dt>Protocol</dt><dd>{{ definition.protocol }}</dd></div>
        <div><dt>Digest</dt><dd>{{ definition.digest }}</dd></div>
        <div><dt>Source</dt><dd>{{ definition.source }}</dd></div>
      </dl>
    </header>

    <div class="definition-map" aria-label="Frozen Pipeline definition index and edge ledger">
      <div class="definition-node-strip">
        <template v-for="node in nodes" :key="node.id">
        <button :class="[{ active: selected.nodeId === node.id }, `definition-node-${node.kind}`]" @click="emit('select', node.id)">
          <span>{{ String(node.index).padStart(2, "0") }}</span><strong>{{ node.title }}</strong><small>{{ node.kind }}</small>
        </button>
        </template>
      </div>
      <div class="definition-edge-ledger" aria-label="Declared graph edges">
        <strong><GitBranch :size="14" />Declared edges</strong>
        <span v-for="edge in definition.edges" :key="`${edge.from}-${edge.to}-${edge.when}`" :class="{ loop: edge.loop }">
          {{ edge.from }} → {{ edge.to }}
          <em v-if="edge.when">when {{ edge.when }}</em>
          <em v-if="edge.loop">loop ≤ {{ edge.loop.maxIterations }} · exhausted: {{ edge.loop.onExhausted }}</em>
        </span>
      </div>
    </div>

    <div class="definition-layout">
      <aside class="definition-index">
        <header><strong>Nodes</strong><span>{{ definition.nodes.length }}</span></header>
        <button v-for="node in nodes" :key="node.id" :class="{ active: selected.nodeId === node.id }" @click="emit('select', node.id)">
          <span>{{ String(node.index).padStart(2, "0") }}</span>
          <span><strong>{{ node.title }}</strong><small>{{ node.kind }} · {{ node.runtime }}</small></span>
          <Check v-if="node.status === 'completed'" :size="14" />
        </button>
        <footer><PackageCheck :size="15" />Installed {{ definition.installedAt }}</footer>
      </aside>

      <section class="definition-detail">
        <header>
          <div><span>{{ selected.nodeId }} · {{ nodes.find(n => n.id === selected.nodeId)?.kind }}</span><h3>{{ nodes.find(n => n.id === selected.nodeId)?.title }}</h3><p>{{ selected.summary }}</p></div>
          <button class="primary-action" @click="emit('edit', selected.nodeId)"><Sparkles :size="15" />让模型修改此 Node</button>
        </header>

        <section class="definition-section">
          <h4>Execution contract</h4>
          <dl class="definition-facts">
            <div><dt>Runtime</dt><dd>{{ runtime }}</dd></div>
            <div><dt>{{ selected.promptRef ? 'Prompt' : 'Action' }}</dt><dd>{{ selected.promptRef || selected.action }}</dd></div>
            <div><dt>Context</dt><dd>{{ selected.context }}</dd></div>
            <div><dt>Sandbox</dt><dd>{{ selected.sandbox }}</dd></div>
            <div><dt>Policy</dt><dd>{{ selected.policy }}</dd></div>
          </dl>
        </section>

        <section class="definition-section contract-grid">
          <div><h4>Reads</h4><ul><li v-for="input in selected.inputs" :key="input"><ArrowRight :size="13" />{{ input }}</li></ul></div>
          <div><h4>Publishes</h4><ul><li v-for="output in selected.outputs" :key="output"><PackageCheck :size="13" />{{ output }}</li></ul></div>
        </section>

        <section class="definition-section source-block">
          <h4>Package source</h4>
          <p><FileCode2 :size="14" />{{ definition.entrypoint }} → {{ selected.promptRef || selected.action }}</p>
          <p><ShieldCheck :size="14" />Definition is frozen for this Run. Editing creates a new Package version.</p>
        </section>
      </section>

      <aside class="capability-inspector">
        <section>
          <h4><Bot :size="15" />Skills <span>{{ selected.skillIds.length }}</span></h4>
          <article v-for="skill in selected.skillIds" :key="skill"><Braces :size="15" /><div><strong>{{ skill }}</strong><span>Loaded for this Node only</span></div></article>
          <p v-if="selected.skillIds.length === 0" class="definition-empty">No Skill binding</p>
        </section>
        <section>
          <h4><PlugZap :size="15" />MCP integrations <span>{{ selected.mcpServers.length }}</span></h4>
          <article v-for="mcp in selected.mcpServers" :key="mcp.name" class="mcp-binding">
            <PlugZap :size="15" /><div><strong>{{ mcp.name }}</strong><span>{{ mcp.transport }} · {{ mcp.permission }} permission</span><code>{{ mcp.tools.join(" · ") }}</code></div>
          </article>
          <p v-if="selected.mcpServers.length === 0" class="definition-empty">No MCP exposed to this Node</p>
        </section>
        <section>
          <h4><KeyRound :size="15" />Capabilities <span>{{ selected.capabilities.length }}</span></h4>
          <div class="capability-list"><span v-for="capability in selected.capabilities" :key="capability">{{ capability }}</span></div>
        </section>
      </aside>
    </div>
  </main>
</template>
