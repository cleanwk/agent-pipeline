use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("storage error: {0}")]
    Storage(#[from] rusqlite::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("unknown node: {0}")]
    UnknownNode(String),
    #[error("invalid command: {0}")]
    InvalidCommand(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("package syntax error: {0}")]
    Syntax(#[from] serde_yaml::Error),
    #[error("package validation error: {0}")]
    Validation(String),
    #[error("package I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageManifest {
    pub protocol: String,
    pub kind: String,
    pub metadata: PackageManifestMetadata,
    pub pipelines: Vec<PipelineReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageManifestMetadata {
    pub name: String,
    pub display_name: Option<String>,
    pub version: String,
    pub description: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineReference {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LoadedPackage {
    pub root: PathBuf,
    pub manifest: PackageManifest,
    pub pipelines: Vec<PipelineDefinition>,
}

#[derive(Debug, Deserialize)]
struct PipelineDocument {
    protocol: String,
    kind: String,
    metadata: PipelineDocumentMetadata,
    entry: Option<String>,
    nodes: Vec<NodeDefinition>,
    edges: Vec<EdgeDefinition>,
    #[serde(default)]
    inputs: serde_yaml::Value,
    #[serde(default)]
    context: serde_yaml::Value,
    #[serde(default)]
    policies: serde_yaml::Value,
    #[serde(default, rename = "deliverySlots")]
    delivery_slots: Vec<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
struct PipelineDocumentMetadata {
    id: String,
}

impl LoadedPackage {
    pub fn load(root: impl AsRef<Path>) -> std::result::Result<Self, PackageError> {
        let root = root.as_ref().canonicalize()?;
        let manifest_path = root.join("agent-pipeline.package.yaml");
        let source = fs::read_to_string(&manifest_path)?;
        let manifest: PackageManifest = serde_yaml::from_str(&source)?;
        if manifest.protocol != "agent-pipeline.dev/v1alpha1" || manifest.kind != "PipelinePackage"
        {
            return Err(PackageError::Validation(
                "unsupported package protocol or kind".into(),
            ));
        }
        if manifest.metadata.name.is_empty() || manifest.metadata.version.is_empty() {
            return Err(PackageError::Validation(
                "package name and version are required".into(),
            ));
        }
        let mut pipelines = Vec::with_capacity(manifest.pipelines.len());
        for reference in &manifest.pipelines {
            let pipeline_path = confined_path(&root, &reference.path)?;
            let pipeline_source = fs::read_to_string(&pipeline_path)?;
            let document: PipelineDocument = serde_yaml::from_str(&pipeline_source)?;
            if document.protocol != "agent-pipeline.dev/v1alpha1" || document.kind != "Pipeline" {
                return Err(PackageError::Validation(format!(
                    "{} is not a supported Pipeline document",
                    reference.path.display()
                )));
            }
            let value: serde_yaml::Value = serde_yaml::from_str(&pipeline_source)?;
            validate_file_references(&value, pipeline_path.parent().unwrap_or(&root), &root)?;
            let entry = document
                .entry
                .or_else(|| document.nodes.first().map(|node| node.id.clone()))
                .ok_or_else(|| {
                    PackageError::Validation(format!(
                        "pipeline {} has no nodes",
                        document.metadata.id
                    ))
                })?;
            let pipeline = PipelineDefinition {
                id: document.metadata.id,
                entry,
                nodes: document.nodes,
                edges: document.edges,
                inputs: document.inputs,
                context: document.context,
                policies: document.policies,
                delivery_slots: document.delivery_slots,
            };
            validate_pipeline(&pipeline)?;
            pipelines.push(pipeline);
        }
        Ok(Self {
            root,
            manifest,
            pipelines,
        })
    }
}

fn confined_path(root: &Path, relative: &Path) -> std::result::Result<PathBuf, PackageError> {
    if relative.is_absolute() {
        return Err(PackageError::Validation(format!(
            "absolute package path is forbidden: {}",
            relative.display()
        )));
    }
    let path = root.join(relative).canonicalize()?;
    if !path.starts_with(root) {
        return Err(PackageError::Validation(format!(
            "package path escapes root: {}",
            relative.display()
        )));
    }
    Ok(path)
}

fn validate_file_references(
    value: &serde_yaml::Value,
    base: &Path,
    root: &Path,
) -> std::result::Result<(), PackageError> {
    match value {
        serde_yaml::Value::Mapping(mapping) => {
            for (key, child) in mapping {
                if matches!(key.as_str(), Some("prompt" | "schema"))
                    && let Some(relative) = child.as_str()
                {
                    let resolved = base.join(relative).canonicalize()?;
                    if !resolved.starts_with(root) || !resolved.is_file() {
                        return Err(PackageError::Validation(format!(
                            "referenced file is outside the package or missing: {relative}"
                        )));
                    }
                }
                validate_file_references(child, base, root)?;
            }
        }
        serde_yaml::Value::Sequence(sequence) => {
            for child in sequence {
                validate_file_references(child, base, root)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelinePackage {
    pub protocol: String,
    pub kind: String,
    pub metadata: PackageMetadata,
    pub pipelines: Vec<PipelineDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDefinition {
    pub id: String,
    pub entry: String,
    pub nodes: Vec<NodeDefinition>,
    pub edges: Vec<EdgeDefinition>,
    #[serde(default)]
    pub inputs: serde_yaml::Value,
    #[serde(default)]
    pub context: serde_yaml::Value,
    #[serde(default)]
    pub policies: serde_yaml::Value,
    #[serde(default, rename = "deliverySlots")]
    pub delivery_slots: Vec<serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub prompt: Option<PathBuf>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub mcp: Vec<serde_yaml::Value>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub inputs: serde_yaml::Value,
    #[serde(default)]
    pub outputs: serde_yaml::Value,
    #[serde(default)]
    pub sandbox: serde_yaml::Value,
    #[serde(default)]
    pub approval: Option<String>,
    #[serde(default, rename = "decisionSchema")]
    pub decision_schema: serde_yaml::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDefinition {
    pub from: String,
    pub to: String,
    pub when: Option<String>,
    #[serde(rename = "loop")]
    pub loop_policy: Option<LoopPolicy>,
    #[serde(default)]
    pub handoff: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoopPolicy {
    #[serde(alias = "max_iterations")]
    pub max_iterations: u32,
    #[serde(alias = "on_exhausted")]
    pub on_exhausted: String,
}

impl PipelinePackage {
    pub fn parse(source: &str) -> std::result::Result<Self, PackageError> {
        Ok(serde_yaml::from_str(source)?)
    }

    pub fn validate(&self) -> std::result::Result<(), PackageError> {
        if self.protocol != "agent-pipeline.dev/v1alpha1" || self.kind != "PipelinePackage" {
            return Err(PackageError::Validation(
                "unsupported protocol or kind".into(),
            ));
        }
        for pipeline in &self.pipelines {
            validate_pipeline(pipeline)?;
        }
        Ok(())
    }
}

fn validate_pipeline(pipeline: &PipelineDefinition) -> std::result::Result<(), PackageError> {
    let nodes: HashSet<&str> = pipeline.nodes.iter().map(|node| node.id.as_str()).collect();
    if nodes.len() != pipeline.nodes.len() {
        return Err(PackageError::Validation(format!(
            "pipeline {} contains duplicate node ids",
            pipeline.id
        )));
    }
    if !nodes.contains(pipeline.entry.as_str()) {
        return Err(PackageError::Validation(format!(
            "pipeline {} entry does not exist",
            pipeline.id
        )));
    }
    if let Some(node) = pipeline
        .nodes
        .iter()
        .find(|node| !matches!(node.node_type.as_str(), "agent" | "action" | "gate"))
    {
        return Err(PackageError::Validation(format!(
            "node {} has unsupported type {}",
            node.id, node.node_type
        )));
    }
    for edge in &pipeline.edges {
        if !nodes.contains(edge.from.as_str()) || !nodes.contains(edge.to.as_str()) {
            return Err(PackageError::Validation(format!(
                "edge {} -> {} references an unknown node",
                edge.from, edge.to
            )));
        }
        if let Some(policy) = &edge.loop_policy
            && (policy.max_iterations == 0
                || !matches!(policy.on_exhausted.as_str(), "attention" | "fail" | "skip"))
        {
            return Err(PackageError::Validation(format!(
                "edge {} -> {} has an invalid loop policy",
                edge.from, edge.to
            )));
        }
    }
    validate_bounded_cycles(pipeline)
}

fn validate_bounded_cycles(pipeline: &PipelineDefinition) -> std::result::Result<(), PackageError> {
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &pipeline.edges {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }
    let cycle_edges: Vec<&EdgeDefinition> = pipeline
        .edges
        .iter()
        .filter(|edge| {
            reachable(
                edge.to.as_str(),
                edge.from.as_str(),
                &adjacency,
                &mut HashSet::new(),
            )
        })
        .collect();
    if !cycle_edges.is_empty() && !cycle_edges.iter().any(|edge| edge.loop_policy.is_some()) {
        let edge = cycle_edges[0];
        return Err(PackageError::Validation(format!(
            "cycle containing {} -> {} requires a bounded loop policy",
            edge.from, edge.to
        )));
    }
    Ok(())
}

fn reachable<'a>(
    current: &'a str,
    target: &str,
    adjacency: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
) -> bool {
    if current == target {
        return true;
    }
    if !visited.insert(current) {
        return false;
    }
    adjacency.get(current).is_some_and(|next| {
        next.iter()
            .any(|node| reachable(node, target, adjacency, visited))
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Command {
    SelectNode { node_id: String },
    RequestChanges { node_id: String, reason: String },
    Approve { node_id: String },
    Advance,
    ResetDemo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Completed,
    Running,
    Attention,
    Waiting,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub status: NodeStatus,
    pub time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineNode {
    pub id: String,
    pub index: u8,
    pub title: String,
    pub kind: String,
    pub status: NodeStatus,
    pub attempt: u32,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub duration: Option<String>,
    pub runtime: String,
    pub activities: Vec<Activity>,
    pub artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub id: String,
    pub title: String,
    pub media_type: String,
    pub revision: u32,
    pub producer_node_id: String,
    pub producer_attempt: u32,
    pub created_at: String,
    pub size: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttentionItem {
    pub id: String,
    pub node_id: String,
    pub severity: String,
    pub title: String,
    pub detail: String,
    pub time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunProjection {
    pub id: String,
    pub title: String,
    pub status: String,
    pub started_at: String,
    pub elapsed: String,
    pub workspace: String,
    pub branch: String,
    pub nodes: Vec<PipelineNode>,
    pub artifacts: Vec<Artifact>,
    pub attention: Vec<AttentionItem>,
    pub brief: String,
    pub selected_node_id: String,
    #[serde(default)]
    pub event_count: u64,
    #[serde(default = "default_definition_package")]
    pub definition_package: String,
    #[serde(default = "default_definition_version")]
    pub definition_version: String,
    #[serde(default = "default_definition_digest")]
    pub definition_digest: String,
}

fn default_definition_package() -> String {
    "seven-stage-product-delivery".into()
}

fn default_definition_version() -> String {
    "0.2.0".into()
}

fn default_definition_digest() -> String {
    "sha256:c54ba184fbdd7530db90e79a0ee9cb7f8c72c6d94612c448e0f0206419e18708".into()
}

impl RunProjection {
    pub fn node(&self, id: &str) -> Option<&PipelineNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    fn node_mut(&mut self, id: &str) -> Result<&mut PipelineNode> {
        self.nodes
            .iter_mut()
            .find(|node| node.id == id)
            .ok_or_else(|| EngineError::UnknownNode(id.to_owned()))
    }
}

pub struct Engine {
    connection: Connection,
}

impl Engine {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let connection = Connection::open(path)?;
        connection.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS current_run (
               singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
               projection_json TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS run_events (
               sequence INTEGER PRIMARY KEY AUTOINCREMENT,
               event_type TEXT NOT NULL,
               payload_json TEXT NOT NULL,
               created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
             );",
        )?;
        let existing: Option<i64> = connection
            .query_row(
                "SELECT singleton FROM current_run WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        if existing.is_none() {
            let json = serde_json::to_string(&demo_projection())?;
            connection.execute(
                "INSERT INTO current_run(singleton, projection_json) VALUES(1, ?1)",
                [json],
            )?;
            connection.execute(
                "INSERT INTO run_events(event_type, payload_json) VALUES('run.seeded', '{}')",
                [],
            )?;
        }
        Ok(Self { connection })
    }

    pub fn snapshot(&self) -> Result<RunProjection> {
        let json: String = self.connection.query_row(
            "SELECT projection_json FROM current_run WHERE singleton = 1",
            [],
            |row| row.get(0),
        )?;
        let mut projection: RunProjection = serde_json::from_str(&json)?;
        let event_count: i64 =
            self.connection
                .query_row("SELECT COUNT(*) FROM run_events", [], |row| row.get(0))?;
        projection.event_count = event_count as u64;
        Ok(projection)
    }

    pub fn dispatch(&self, command: Command) -> Result<RunProjection> {
        let mut projection = self.snapshot()?;
        let (event_type, payload) = match command {
            Command::SelectNode { node_id } => {
                if projection.node(&node_id).is_none() {
                    return Err(EngineError::UnknownNode(node_id));
                }
                projection.selected_node_id = node_id.clone();
                ("ui.node_selected", serde_json::json!({ "nodeId": node_id }))
            }
            Command::RequestChanges { node_id, reason } => {
                if node_id != "review" {
                    return Err(EngineError::InvalidCommand(
                        "only Review can request changes in the example pipeline".into(),
                    ));
                }
                {
                    let review = projection.node_mut("review")?;
                    review.status = NodeStatus::Completed;
                    review.finished_at = Some("11:14".into());
                    review.activities.push(Activity {
                        id: "r3".into(),
                        title: "已请求修改".into(),
                        detail: reason.clone(),
                        status: NodeStatus::Completed,
                        time: "11:14".into(),
                    });
                }
                {
                    let implement = projection.node_mut("implement")?;
                    implement.attempt += 1;
                    implement.status = NodeStatus::Running;
                    implement.started_at = Some("11:14".into());
                    implement.finished_at = None;
                    implement.duration = Some("1m".into());
                    implement.activities = vec![Activity {
                        id: "i2-1".into(),
                        title: "读取 Review feedback".into(),
                        detail: "正在定位异步退款幂等校验路径".into(),
                        status: NodeStatus::Running,
                        time: "11:15".into(),
                    }];
                }
                projection.status = "running".into();
                projection.selected_node_id = "implement".into();
                projection.attention = vec![AttentionItem {
                    id: "attn-implement-2".into(),
                    node_id: "implement".into(),
                    severity: "info".into(),
                    title: "Implement · Attempt 2 正在运行".into(),
                    detail: "已把 Review feedback 作为显式 Handoff 注入".into(),
                    time: "11:15".into(),
                }];
                projection.artifacts.push(Artifact {
                    id: "review-feedback-1".into(),
                    title: "Review feedback · Attempt 1".into(),
                    media_type: "Markdown".into(),
                    revision: 1,
                    producer_node_id: "review".into(),
                    producer_attempt: 1,
                    created_at: "11:14".into(),
                    size: "1.3 KB".into(),
                    summary: reason.clone(),
                });
                projection.brief = format!(
                    "Review 已打回 Attempt 1：{reason}。Implement Attempt 2 正在执行，并已读取冻结的 Spec、Patch、测试报告与 Review feedback。"
                );
                (
                    "review.changes_requested",
                    serde_json::json!({ "nodeId": node_id, "reason": reason }),
                )
            }
            Command::Approve { node_id } => {
                if node_id != "review" {
                    return Err(EngineError::InvalidCommand(
                        "only Review is approvable in the example pipeline".into(),
                    ));
                }
                let review = projection.node_mut("review")?;
                review.status = NodeStatus::Completed;
                review.finished_at = Some("11:14".into());
                let deploy = projection.node_mut("deploy")?;
                deploy.status = NodeStatus::Running;
                deploy.attempt = 1;
                deploy.started_at = Some("11:14".into());
                deploy.activities = vec![Activity {
                    id: "d1".into(),
                    title: "准备部署清单".into(),
                    detail: "解析 OCM 环境能力并检查权限".into(),
                    status: NodeStatus::Running,
                    time: "11:15".into(),
                }];
                projection.status = "running".into();
                projection.selected_node_id = "deploy".into();
                projection.attention.clear();
                ("review.approved", serde_json::json!({ "nodeId": node_id }))
            }
            Command::Advance => {
                advance_demo(&mut projection)?;
                ("run.advanced", serde_json::json!({}))
            }
            Command::ResetDemo => {
                projection = demo_projection();
                ("run.reset", serde_json::json!({}))
            }
        };
        self.persist(&projection, event_type, &payload)?;
        self.snapshot()
    }

    fn persist(
        &self,
        projection: &RunProjection,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Result<()> {
        let projection_json = serde_json::to_string(projection)?;
        let payload_json = serde_json::to_string(payload)?;
        self.connection.execute_batch("BEGIN IMMEDIATE")?;
        let result = (|| -> Result<()> {
            self.connection.execute(
                "UPDATE current_run SET projection_json = ?1 WHERE singleton = 1",
                [projection_json],
            )?;
            self.connection.execute(
                "INSERT INTO run_events(event_type, payload_json) VALUES(?1, ?2)",
                params![event_type, payload_json],
            )?;
            Ok(())
        })();
        match result {
            Ok(()) => self.connection.execute_batch("COMMIT")?,
            Err(error) => {
                let _ = self.connection.execute_batch("ROLLBACK");
                return Err(error);
            }
        }
        Ok(())
    }
}

fn advance_demo(projection: &mut RunProjection) -> Result<()> {
    if projection
        .node("implement")
        .is_some_and(|node| node.status == NodeStatus::Running)
    {
        let implement = projection.node_mut("implement")?;
        implement.status = NodeStatus::Completed;
        implement.finished_at = Some("11:34".into());
        implement.duration = Some("20m".into());
        implement.activities.push(Activity {
            id: "i2-2".into(),
            title: "补齐幂等校验".into(),
            detail: "新增唯一约束与重复请求回放测试".into(),
            status: NodeStatus::Completed,
            time: "11:34".into(),
        });
        let review = projection.node_mut("review")?;
        review.status = NodeStatus::Attention;
        review.attempt = 2;
        review.started_at = Some("11:34".into());
        review.finished_at = None;
        review.activities = vec![Activity {
            id: "r2-1".into(),
            title: "复核 Attempt 2".into(),
            detail: "变更满足 feedback，等待最终批准".into(),
            status: NodeStatus::Attention,
            time: "11:39".into(),
        }];
        projection.selected_node_id = "review".into();
        projection.status = "attention".into();
        projection.attention = vec![AttentionItem {
            id: "attn-review-2".into(),
            node_id: "review".into(),
            severity: "critical".into(),
            title: "Review Attempt 2 需要确认".into(),
            detail: "幂等修复已完成，请复核变更".into(),
            time: "11:39".into(),
        }];
    } else if projection
        .node("deploy")
        .is_some_and(|node| node.status == NodeStatus::Running)
    {
        let deploy = projection.node_mut("deploy")?;
        deploy.status = NodeStatus::Completed;
        deploy.finished_at = Some("11:48".into());
        deploy.duration = Some("9m".into());
        deploy.artifact_ids = vec!["deployment-receipt".into()];
        deploy.activities.push(Activity {
            id: "d2".into(),
            title: "部署到测试环境".into(),
            detail: "OCM ref: env/refund-test-42".into(),
            status: NodeStatus::Completed,
            time: "11:48".into(),
        });
        projection.artifacts.push(Artifact {
            id: "deployment-receipt".into(),
            title: "Deployment Receipt".into(),
            media_type: "Environment reference".into(),
            revision: 1,
            producer_node_id: "deploy".into(),
            producer_attempt: 1,
            created_at: "11:48".into(),
            size: "Local ref".into(),
            summary: "测试环境 refund-test-42 部署成功。".into(),
        });
        let smoke = projection.node_mut("smoke")?;
        smoke.status = NodeStatus::Running;
        smoke.attempt = 1;
        smoke.started_at = Some("11:48".into());
        smoke.activities = vec![Activity {
            id: "sm1".into(),
            title: "执行核心退款路径".into(),
            detail: "创建退款、重复请求回放与状态查询".into(),
            status: NodeStatus::Running,
            time: "11:49".into(),
        }];
        projection.selected_node_id = "smoke".into();
        projection.brief = "Review Attempt 2 已批准，测试环境部署完成。Smoke Test 正依据 Deployment Receipt 验证退款主路径与幂等回放。".into();
    } else if projection
        .node("smoke")
        .is_some_and(|node| node.status == NodeStatus::Running)
    {
        let smoke = projection.node_mut("smoke")?;
        smoke.status = NodeStatus::Completed;
        smoke.finished_at = Some("11:56".into());
        smoke.duration = Some("8m".into());
        smoke.artifact_ids = vec!["smoke-report".into()];
        smoke.activities.push(Activity {
            id: "sm2".into(),
            title: "发布 Smoke Test 报告".into(),
            detail: "6 passed · 0 failed".into(),
            status: NodeStatus::Completed,
            time: "11:56".into(),
        });
        projection.artifacts.push(Artifact {
            id: "smoke-report".into(),
            title: "Smoke Test Report".into(),
            media_type: "JSON".into(),
            revision: 1,
            producer_node_id: "smoke".into(),
            producer_attempt: 1,
            created_at: "11:56".into(),
            size: "3.6 KB".into(),
            summary: "退款主路径、重复请求回放和状态查询共 6 项全部通过。".into(),
        });
        projection.status = "completed".into();
        projection.attention.clear();
        projection.selected_node_id = "smoke".into();
        projection.brief = "七阶段 Pipeline 已完成。Review feedback 在 Attempt 2 关闭，部署与 Smoke Test 均已发布可追溯 Artifact。".into();
    }
    Ok(())
}

fn activity(id: &str, title: &str, detail: &str, status: NodeStatus, time: &str) -> Activity {
    Activity {
        id: id.into(),
        title: title.into(),
        detail: detail.into(),
        status,
        time: time.into(),
    }
}

fn node(
    id: &str,
    index: u8,
    title: &str,
    kind: &str,
    status: NodeStatus,
    attempt: u32,
    runtime: &str,
) -> PipelineNode {
    PipelineNode {
        id: id.into(),
        index,
        title: title.into(),
        kind: kind.into(),
        status,
        attempt,
        started_at: None,
        finished_at: None,
        duration: None,
        runtime: runtime.into(),
        activities: vec![],
        artifact_ids: vec![],
    }
}

fn demo_projection() -> RunProjection {
    let mut grill = node("grill", 1, "Grill", "agent", NodeStatus::Completed, 1, "Pi");
    grill.finished_at = Some("10:15".into());
    grill.duration = Some("8m".into());
    grill.artifact_ids = vec!["grill-record".into()];
    grill.activities = vec![
        activity(
            "g1",
            "梳理目标与边界",
            "确认退款入口、权限和异常路径",
            NodeStatus::Completed,
            "10:11",
        ),
        activity(
            "g2",
            "关闭关键问题",
            "12 个问题均已回答",
            NodeStatus::Completed,
            "10:15",
        ),
    ];
    let mut ticket = node(
        "ticket",
        2,
        "Ticket",
        "action",
        NodeStatus::Completed,
        1,
        "Local action",
    );
    ticket.finished_at = Some("10:17".into());
    ticket.duration = Some("2m".into());
    ticket.artifact_ids = vec!["ticket".into()];
    let mut spec = node("spec", 3, "Spec", "agent", NodeStatus::Completed, 1, "Pi");
    spec.finished_at = Some("10:28".into());
    spec.duration = Some("11m".into());
    spec.artifact_ids = vec!["spec".into()];
    spec.activities = vec![
        activity(
            "s1",
            "建立技术约束",
            "幂等键、状态机与补偿策略",
            NodeStatus::Completed,
            "10:21",
        ),
        activity(
            "s2",
            "发布技术方案",
            "Spec revision 2",
            NodeStatus::Completed,
            "10:28",
        ),
    ];
    let mut implement = node(
        "implement",
        4,
        "Implement",
        "agent",
        NodeStatus::Completed,
        1,
        "Codex",
    );
    implement.finished_at = Some("11:02".into());
    implement.duration = Some("34m".into());
    implement.artifact_ids = vec!["patch".into(), "test-report".into()];
    implement.activities = vec![
        activity(
            "i1",
            "设计实现方案",
            "映射 Spec 到模块改动",
            NodeStatus::Completed,
            "10:31",
        ),
        activity(
            "i2",
            "编码实现",
            "7 files changed · +284 −19",
            NodeStatus::Completed,
            "10:52",
        ),
        activity(
            "i3",
            "单元测试",
            "42 passed · 0 failed",
            NodeStatus::Completed,
            "11:02",
        ),
    ];
    let mut review = node(
        "review",
        5,
        "Review",
        "gate",
        NodeStatus::Attention,
        1,
        "Codex",
    );
    review.started_at = Some("11:02".into());
    review.duration = Some("10m".into());
    review.artifact_ids = vec!["review".into()];
    review.activities = vec![
        activity(
            "r1",
            "分析代码变更",
            "检查退款状态机与数据一致性",
            NodeStatus::Completed,
            "11:08",
        ),
        activity(
            "r2",
            "等待人工确认",
            "异步退款缺少强制幂等校验",
            NodeStatus::Attention,
            "11:12",
        ),
    ];
    let deploy = node(
        "deploy",
        6,
        "Deploy",
        "action",
        NodeStatus::Waiting,
        0,
        "OCM adapter",
    );
    let smoke = node(
        "smoke",
        7,
        "Smoke Test",
        "agent",
        NodeStatus::Waiting,
        0,
        "Pi",
    );
    let artifacts = vec![
        Artifact {
            id: "grill-record".into(),
            title: "Grill 问答记录".into(),
            media_type: "Markdown".into(),
            revision: 1,
            producer_node_id: "grill".into(),
            producer_attempt: 1,
            created_at: "10:15".into(),
            size: "8.2 KB".into(),
            summary: "12 个关键问题及已确认答案。".into(),
        },
        Artifact {
            id: "ticket".into(),
            title: "PAY-2841".into(),
            media_type: "Ticket reference".into(),
            revision: 1,
            producer_node_id: "ticket".into(),
            producer_attempt: 1,
            created_at: "10:17".into(),
            size: "Local ref".into(),
            summary: "退款能力开发与上线追踪。".into(),
        },
        Artifact {
            id: "spec".into(),
            title: "退款技术方案".into(),
            media_type: "Markdown".into(),
            revision: 2,
            producer_node_id: "spec".into(),
            producer_attempt: 1,
            created_at: "10:28".into(),
            size: "24.6 KB".into(),
            summary: "退款状态机、幂等、补偿与可观测性设计。".into(),
        },
        Artifact {
            id: "patch".into(),
            title: "实现 Patch".into(),
            media_type: "Git diff".into(),
            revision: 1,
            producer_node_id: "implement".into(),
            producer_attempt: 1,
            created_at: "10:52".into(),
            size: "18.3 KB".into(),
            summary: "7 files changed · +284 −19。".into(),
        },
        Artifact {
            id: "test-report".into(),
            title: "单元测试报告".into(),
            media_type: "JSON".into(),
            revision: 1,
            producer_node_id: "implement".into(),
            producer_attempt: 1,
            created_at: "11:02".into(),
            size: "4.8 KB".into(),
            summary: "42 passed · 0 failed。".into(),
        },
        Artifact {
            id: "review".into(),
            title: "Code Review".into(),
            media_type: "Markdown".into(),
            revision: 1,
            producer_node_id: "review".into(),
            producer_attempt: 1,
            created_at: "11:12".into(),
            size: "3.1 KB".into(),
            summary: "发现异步退款幂等校验缺失，等待确认。".into(),
        },
    ];
    RunProjection {
        id: "run_20260816_0017".into(), title: "支付退款能力上线".into(), status: "attention".into(),
        started_at: "2026-08-16 10:14:32".into(), elapsed: "2h 34m 18s".into(), workspace: "payments-platform".into(), branch: "agent/refund-capability".into(),
        nodes: vec![grill, ticket, spec, implement, review, deploy, smoke], artifacts,
        attention: vec![
            AttentionItem { id: "attn-review".into(), node_id: "review".into(), severity: "critical".into(), title: "Review 需要确认".into(), detail: "请检查 Review 的输出并决定是否打回".into(), time: "10:16".into() },
            AttentionItem { id: "attn-deploy".into(), node_id: "deploy".into(), severity: "info".into(), title: "部署尚未执行".into(), detail: "Deploy 等待 Review gate".into(), time: "10:14".into() },
        ],
        brief: "退款能力已完成需求澄清、Ticket 与技术方案。Implement Attempt 1 已发布实现与测试结果；Review 正等待确认异步退款幂等边界。".into(),
        selected_node_id: "review".into(), event_count: 0,
        definition_package: default_definition_package(),
        definition_version: default_definition_version(),
        definition_digest: default_definition_digest(),
    }
}
