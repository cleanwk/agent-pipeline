use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::{
        fs::PermissionsExt,
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    process::{Child, Command as ProcessCommand, Stdio},
    sync::mpsc,
    thread,
    time::Duration,
};

use agent_pipeline_core::{Command, Engine, LoadedPackage, RunProjection};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PiCapabilitySnapshot {
    pub session_id: Option<String>,
    pub session_file: Option<String>,
    pub model_id: Option<String>,
    pub auto_compaction_enabled: bool,
}

pub struct PiRpcAdapter;

impl PiRpcAdapter {
    pub fn probe(binary: &Path) -> Result<PiCapabilitySnapshot> {
        let mut child = ProcessCommand::new(binary)
            .args(["--mode", "rpc", "--no-session", "--no-extensions"])
            .env("PI_TELEMETRY", "0")
            .env("PI_SKIP_VERSION_CHECK", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| RunnerError::Rejected("Pi RPC stdin unavailable".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| RunnerError::Rejected("Pi RPC stdout unavailable".into()))?;
        stdin.write_all(b"{\"id\":\"doctor\",\"type\":\"get_state\"}\n")?;
        stdin.flush()?;

        let (sender, receiver) = mpsc::sync_channel(1);
        thread::spawn(move || {
            for line in BufReader::new(stdout)
                .lines()
                .map_while(std::result::Result::ok)
            {
                let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
                    continue;
                };
                if value.get("type").and_then(serde_json::Value::as_str) == Some("response")
                    && value.get("id").and_then(serde_json::Value::as_str) == Some("doctor")
                {
                    let _ = sender.send(value);
                    break;
                }
            }
        });
        let response = receiver
            .recv_timeout(Duration::from_secs(20))
            .map_err(|_| {
                stop_child(&mut child);
                RunnerError::Rejected("Pi RPC capability handshake timed out".into())
            })?;
        stop_child(&mut child);
        if response.get("success").and_then(serde_json::Value::as_bool) != Some(true) {
            return Err(RunnerError::Rejected("Pi RPC rejected get_state".into()));
        }
        let data = response.get("data").cloned().unwrap_or_default();
        Ok(PiCapabilitySnapshot {
            session_id: data
                .get("sessionId")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            session_file: data
                .get("sessionFile")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            model_id: data
                .get("model")
                .and_then(|model| model.get("id"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            auto_compaction_enabled: data
                .get("autoCompactionEnabled")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        })
    }
}

fn stop_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("runner I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("runner protocol error: {0}")]
    Protocol(#[from] serde_json::Error),
    #[error("runner rejected request: {0}")]
    Rejected(String),
    #[error("runner did not become ready")]
    StartupTimeout,
    #[error("engine error: {0}")]
    Engine(#[from] agent_pipeline_core::EngineError),
}

pub type Result<T> = std::result::Result<T, RunnerError>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    protocol_version: u32,
    operation: Operation,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "payload")]
enum Operation {
    Health,
    Snapshot,
    Dispatch(Command),
    InstallPackage { source_path: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPackage {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub pipeline_count: usize,
    pub install_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    protocol_version: u32,
    ok: bool,
    run: Option<RunProjection>,
    package: Option<InstalledPackage>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RunnerClient {
    socket_path: PathBuf,
}

impl RunnerClient {
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
        }
    }

    pub fn health(&self) -> Result<()> {
        self.request(Operation::Health).map(|_| ())
    }

    pub fn snapshot(&self) -> Result<RunProjection> {
        self.request(Operation::Snapshot)?
            .run
            .ok_or_else(|| RunnerError::Rejected("snapshot response omitted run".into()))
    }

    pub fn dispatch(&self, command: Command) -> Result<RunProjection> {
        self.request(Operation::Dispatch(command))?
            .run
            .ok_or_else(|| RunnerError::Rejected("dispatch response omitted run".into()))
    }

    pub fn install_package(&self, source_path: impl Into<PathBuf>) -> Result<InstalledPackage> {
        self.request(Operation::InstallPackage {
            source_path: source_path.into(),
        })?
        .package
        .ok_or_else(|| RunnerError::Rejected("install response omitted package".into()))
    }

    fn request(&self, operation: Operation) -> Result<Response> {
        let mut stream = UnixStream::connect(&self.socket_path)?;
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;
        let request = Request {
            protocol_version: PROTOCOL_VERSION,
            operation,
        };
        serde_json::to_writer(&mut stream, &request)?;
        stream.write_all(b"\n")?;
        stream.flush()?;

        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line)?;
        let response: Response = serde_json::from_str(&line)?;
        if response.protocol_version != PROTOCOL_VERSION {
            return Err(RunnerError::Rejected(format!(
                "unsupported runner protocol {}",
                response.protocol_version
            )));
        }
        if !response.ok {
            return Err(RunnerError::Rejected(
                response
                    .error
                    .unwrap_or_else(|| "unknown runner error".into()),
            ));
        }
        Ok(response)
    }
}

pub fn ensure_runner(
    executable: &Path,
    socket_path: &Path,
    database_path: &Path,
) -> Result<RunnerClient> {
    let client = RunnerClient::new(socket_path);
    if client.health().is_ok() {
        return Ok(client);
    }

    if socket_path.exists() {
        fs::remove_file(socket_path)?;
    }
    ProcessCommand::new(executable)
        .arg("--runner")
        .arg("--socket")
        .arg(socket_path)
        .arg("--database")
        .arg(database_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    for _ in 0..60 {
        if client.health().is_ok() {
            return Ok(client);
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(RunnerError::StartupTimeout)
}

pub fn serve(socket_path: &Path, database_path: &Path) -> Result<()> {
    if let Some(parent) = socket_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if socket_path.exists() {
        fs::remove_file(socket_path)?;
    }
    let listener = UnixListener::bind(socket_path)?;
    fs::set_permissions(socket_path, fs::Permissions::from_mode(0o600))?;
    let state = RunnerState {
        engine: Engine::open(database_path)?,
        package_root: database_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("packages"),
    };
    for connection in listener.incoming() {
        match connection {
            Ok(stream) => handle_connection(stream, &state)?,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

struct RunnerState {
    engine: Engine,
    package_root: PathBuf,
}

fn handle_connection(mut stream: UnixStream, state: &RunnerState) -> Result<()> {
    let mut line = String::new();
    BufReader::new(stream.try_clone()?).read_line(&mut line)?;
    let response = match serde_json::from_str::<Request>(&line) {
        Ok(request) if request.protocol_version == PROTOCOL_VERSION => match request.operation {
            Operation::Health => success(None),
            Operation::Snapshot => match state.engine.snapshot() {
                Ok(run) => success(Some(run)),
                Err(error) => failure(error.to_string()),
            },
            Operation::Dispatch(command) => match state.engine.dispatch(command) {
                Ok(run) => success(Some(run)),
                Err(error) => failure(error.to_string()),
            },
            Operation::InstallPackage { source_path } => {
                match install_package(&source_path, &state.package_root) {
                    Ok(package) => package_success(package),
                    Err(error) => failure(error.to_string()),
                }
            }
        },
        Ok(request) => failure(format!(
            "unsupported runner protocol {}",
            request.protocol_version
        )),
        Err(error) => failure(format!("invalid request: {error}")),
    };
    serde_json::to_writer(&mut stream, &response)?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    Ok(())
}

fn success(run: Option<RunProjection>) -> Response {
    Response {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        run,
        package: None,
        error: None,
    }
}

fn package_success(package: InstalledPackage) -> Response {
    Response {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        run: None,
        package: Some(package),
        error: None,
    }
}

fn failure(error: String) -> Response {
    Response {
        protocol_version: PROTOCOL_VERSION,
        ok: false,
        run: None,
        package: None,
        error: Some(error),
    }
}

pub fn install_package(source_path: &Path, package_root: &Path) -> Result<InstalledPackage> {
    let package = LoadedPackage::load(source_path)
        .map_err(|error| RunnerError::Rejected(error.to_string()))?;
    let metadata = &package.manifest.metadata;
    if !safe_segment(&metadata.name) || !safe_segment(&metadata.version) {
        return Err(RunnerError::Rejected(
            "package name and version must be safe path segments".into(),
        ));
    }
    let destination = package_root.join(&metadata.name).join(&metadata.version);
    if !destination.exists() {
        fs::create_dir_all(
            destination
                .parent()
                .expect("version directory has a parent"),
        )?;
        let staging = package_root.join(format!(
            ".installing-{}-{}-{}",
            metadata.name,
            metadata.version,
            std::process::id()
        ));
        if staging.exists() {
            fs::remove_dir_all(&staging)?;
        }
        copy_package_tree(&package.root, &staging)?;
        fs::rename(&staging, &destination)?;
    }
    Ok(InstalledPackage {
        name: metadata.name.clone(),
        display_name: metadata
            .display_name
            .clone()
            .unwrap_or_else(|| metadata.name.clone()),
        version: metadata.version.clone(),
        pipeline_count: package.pipelines.len(),
        install_path: destination,
    })
}

fn safe_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        && value != "."
        && value != ".."
}

fn copy_package_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if file_type.is_symlink() {
            return Err(RunnerError::Rejected(format!(
                "package symlinks are not allowed: {}",
                entry.path().display()
            )));
        }
        if file_type.is_dir() {
            copy_package_tree(&entry.path(), &target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_round_trips_snapshot_and_command_over_local_socket() {
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("runner.sock");
        let database = directory.path().join("pipeline.db");
        let listener = UnixListener::bind(&socket).unwrap();
        let server_database = database.clone();
        let package_root = directory.path().join("installed");
        let server = thread::spawn(move || {
            let state = RunnerState {
                engine: Engine::open(server_database).unwrap(),
                package_root,
            };
            for _ in 0..2 {
                handle_connection(listener.accept().unwrap().0, &state).unwrap();
            }
        });

        let client = RunnerClient::new(&socket);
        let initial = client.snapshot().unwrap();
        let selected = client
            .dispatch(Command::SelectNode {
                node_id: "spec".into(),
            })
            .unwrap();
        assert_eq!(selected.selected_node_id, "spec");
        assert_eq!(selected.event_count, initial.event_count + 1);
        server.join().unwrap();
    }
}
