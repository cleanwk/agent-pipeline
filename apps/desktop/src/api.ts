import { demoAgents, demoRun } from "./demo";
import { applyDemoCommand } from "./projection";
import type { PipelineCommand } from "./projection";
import type { AgentProbe, InstalledPackage, RunProjection } from "./types";

export type { PipelineCommand } from "./projection";

const copy = <T>(value: T): T => structuredClone(value);
let browserRun = copy(demoRun);

async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

export async function bootstrap(): Promise<{ run: RunProjection; agents: AgentProbe[]; native: boolean }> {
  try {
    return await tauriInvoke("bootstrap");
  } catch {
    return { run: copy(browserRun), agents: copy(demoAgents), native: false };
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
  } catch {
    browserRun = applyDemoCommand(browserRun, command);
    return copy(browserRun);
  }
}

export async function installPackage(sourcePath: string): Promise<InstalledPackage> {
  return tauriInvoke("install_package", { sourcePath });
}
