export type NodeStatus = "completed" | "running" | "attention" | "waiting" | "failed";

export interface Activity {
  id: string;
  title: string;
  detail: string;
  status: NodeStatus;
  time: string;
}

export interface PipelineNode {
  id: string;
  index: number;
  title: string;
  kind: "agent" | "action" | "gate";
  status: NodeStatus;
  attempt: number;
  startedAt?: string;
  finishedAt?: string;
  duration?: string;
  runtime?: string;
  activities: Activity[];
  artifactIds: string[];
}

export interface Artifact {
  id: string;
  title: string;
  mediaType: string;
  revision: number;
  producerNodeId: string;
  producerAttempt: number;
  createdAt: string;
  size: string;
  summary: string;
}

export interface AttentionItem {
  id: string;
  nodeId: string;
  severity: "critical" | "info";
  title: string;
  detail: string;
  time: string;
}

export interface RunProjection {
  id: string;
  title: string;
  status: "running" | "attention" | "completed";
  startedAt: string;
  elapsed: string;
  workspace: string;
  branch: string;
  nodes: PipelineNode[];
  artifacts: Artifact[];
  attention: AttentionItem[];
  brief: string;
  selectedNodeId?: string;
  eventCount: number;
}

export interface AgentProbe {
  id: "pi" | "codex" | "claude" | "opencode";
  name: string;
  path?: string;
  version?: string;
  state: "ready" | "missing" | "degraded";
  transport: string;
  capability?: string;
}

export interface InstalledPackage {
  name: string;
  displayName: string;
  version: string;
  pipelineCount: number;
  installPath: string;
}
