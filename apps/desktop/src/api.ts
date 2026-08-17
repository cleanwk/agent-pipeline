import { demoAgents, demoRun } from "./demo";
import { applyDemoCommand } from "./projection";
import type { PipelineCommand } from "./projection";
import type { AgentProbe, InstalledPackage, McpBinding, PipelineDefinition, RunProjection } from "./types";

export type { PipelineCommand } from "./projection";

const copy = <T>(value: T): T => structuredClone(value);
let browserRun = copy(demoRun);

async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

function isTauriRuntime(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

export async function bootstrap(): Promise<{ run: RunProjection; agents: AgentProbe[]; native: boolean; definition: PipelineDefinition }> {
  try {
    const data = await tauriInvoke<{ run: RunProjection; agents: AgentProbe[]; native: boolean; definition: unknown }>("bootstrap");
    return { ...data, definition: mapPackageInspection(data.definition as any) };
  } catch (error) {
    if (isTauriRuntime()) throw error;
    return { run: copy(browserRun), agents: copy(demoAgents), native: false, definition: copy((await import("./definition")).demoDefinition) };
  }
}

export async function dispatch(command: PipelineCommand): Promise<RunProjection> {
  try {
    const nativeCommand = "selectNode" in command
      ? { selectNode: { node_id: command.selectNode.nodeId } }
      : "requestChanges" in command
        ? { requestChanges: { node_id: command.requestChanges.nodeId, reason: command.requestChanges.reason } }
        : "approve" in command
          ? { approve: { node_id: command.approve.nodeId } }
          : "advance" in command
            ? "advance"
            : "resetDemo";
    return await tauriInvoke("dispatch", { command: nativeCommand });
  } catch (error) {
    if (isTauriRuntime()) throw error;
    browserRun = applyDemoCommand(browserRun, command);
    return copy(browserRun);
  }
}

export async function installPackage(sourcePath: string): Promise<InstalledPackage> {
  if (!isTauriRuntime()) throw new Error("Package installation is available in the installed macOS app only.");
  return tauriInvoke("install_package", { sourcePath });
}

export async function inspectPackage(sourcePath: string): Promise<PipelineDefinition> {
  if (!isTauriRuntime()) return copy((await import("./definition")).demoDefinition);
  const inspected = await tauriInvoke<any>("inspect_package", { sourcePath });
  return mapPackageInspection(inspected);
}

export async function inspectInstalledPackage(name: string, version: string): Promise<PipelineDefinition> {
  if (!isTauriRuntime()) return copy((await import("./definition")).demoDefinition);
  const inspected = await tauriInvoke<any>("inspect_installed_package", { name, version });
  return mapPackageInspection(inspected);
}

export function mapPackageInspection(inspected: any): PipelineDefinition {
  const pipeline = inspected.pipelines[0];
  if (!pipeline) throw new Error("Installed package contains no Pipeline definition");
  const nodes = pipeline.nodes.map((node: any) => {
    const incoming = pipeline.edges.filter((edge: any) => edge.to === node.id);
    const policy = node.approval
      ? `Approval: ${node.approval}`
      : node.decisionSchema?.outcomes
        ? `Decision: ${node.decisionSchema.outcomes.join(" | ")}`
        : "Package default policy";
    return {
      nodeId: node.id,
      summary: node.title ? `${node.title} stage declared by the installed Package.` : `${node.id} stage`,
      promptRef: node.prompt ?? undefined,
      action: node.action ?? undefined,
      skillIds: node.skills ?? [],
      mcpServers: (node.mcp ?? []) as McpBinding[],
      capabilities: node.capabilities ?? [],
      inputs: incoming.flatMap((edge: any) => edge.handoff?.length ? edge.handoff : [`${edge.from} outputs`]),
      outputs: Object.keys(node.outputs ?? {}),
      context: JSON.stringify(pipeline.context ?? {}),
      policy,
      sandbox: JSON.stringify(node.sandbox ?? {})
    };
  });
  return {
    protocol: inspected.manifest.protocol,
    packageName: inspected.manifest.metadata.name,
    displayName: inspected.manifest.metadata.displayName ?? inspected.manifest.metadata.name,
    version: inspected.manifest.metadata.version,
    source: inspected.root,
    digest: inspected.digest,
    entrypoint: inspected.manifest.pipelines[0]?.path ?? pipeline.id,
    installedAt: "in the immutable local package store",
    contextPolicy: JSON.stringify(pipeline.context ?? {}),
    nodes,
    edges: pipeline.edges.map((edge: any) => ({
      from: edge.from, to: edge.to, when: edge.when ?? undefined,
      loop: edge.loop ? { maxIterations: edge.loop.maxIterations, onExhausted: edge.loop.onExhausted } : undefined
    }))
  };
}

export async function startWindowDrag(): Promise<void> {
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().startDragging();
  } catch { /* Browser preview has no native window. */ }
}

export async function toggleWindowMaximize(): Promise<void> {
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().toggleMaximize();
  } catch { /* Browser preview has no native window. */ }
}
