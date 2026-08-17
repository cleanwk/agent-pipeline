use std::{path::PathBuf, process::Command as ProcessCommand};

use agent_pipeline_core::{Command, RunProjection};
use agent_pipeline_runner::{
    InstalledPackage, PackageInspection, PiRpcAdapter, RunnerClient, ensure_runner,
};
use serde::Serialize;
use tauri::{Manager, State};

struct AppState {
    runner: RunnerClient,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentProbe {
    id: &'static str,
    name: &'static str,
    path: Option<String>,
    version: Option<String>,
    state: &'static str,
    transport: &'static str,
    capability: Option<String>,
}

#[derive(Serialize)]
struct Bootstrap {
    run: RunProjection,
    definition: PackageInspection,
    agents: Vec<AgentProbe>,
    native: bool,
}

#[tauri::command]
fn bootstrap(state: State<'_, AppState>) -> std::result::Result<Bootstrap, String> {
    let run = state.runner.snapshot().map_err(|error| error.to_string())?;
    let definition = state
        .runner
        .inspect_installed_package(&run.definition_package, &run.definition_version)
        .map_err(|error| error.to_string())?;
    if definition.digest != run.definition_digest {
        return Err(format!(
            "Pipeline definition integrity check failed: Run expects {}, installed Package is {}",
            run.definition_digest, definition.digest
        ));
    }
    Ok(Bootstrap {
        run,
        definition,
        agents: probe_agents(),
        native: true,
    })
}

#[tauri::command]
fn dispatch(
    command: Command,
    state: State<'_, AppState>,
) -> std::result::Result<RunProjection, String> {
    state
        .runner
        .dispatch(command)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn install_package(
    source_path: String,
    state: State<'_, AppState>,
) -> std::result::Result<InstalledPackage, String> {
    let source_path = expand_home(source_path)?;
    state
        .runner
        .install_package(source_path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn inspect_package(
    source_path: String,
    state: State<'_, AppState>,
) -> std::result::Result<PackageInspection, String> {
    let source_path = expand_home(source_path)?;
    state
        .runner
        .inspect_package(source_path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn inspect_installed_package(
    name: String,
    version: String,
    state: State<'_, AppState>,
) -> std::result::Result<PackageInspection, String> {
    state
        .runner
        .inspect_installed_package(name, version)
        .map_err(|error| error.to_string())
}

fn expand_home(source_path: String) -> std::result::Result<PathBuf, String> {
    if let Some(relative) = source_path.strip_prefix("~/") {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| "HOME is unavailable".to_string())
            .map(|home| home.join(relative))
    } else {
        Ok(PathBuf::from(source_path))
    }
}

fn probe_agents() -> Vec<AgentProbe> {
    [
        (
            "pi",
            "Pi",
            "RPC",
            vec!["/opt/homebrew/bin/pi", "/usr/local/bin/pi"],
        ),
        (
            "codex",
            "Codex",
            "ACP / App Server",
            vec!["/opt/homebrew/bin/codex", "/usr/local/bin/codex"],
        ),
        ("claude", "Claude Code", "ACP", vec![".local/bin/claude"]),
        ("opencode", "OpenCode", "ACP", vec![".bun/bin/opencode"]),
    ]
    .into_iter()
    .map(|(id, name, transport, candidates)| {
        let path = find_binary(&candidates);
        let version = path.as_ref().and_then(read_version);
        let (state, capability) = if id == "pi" {
            match path.as_ref().map(|binary| PiRpcAdapter::probe(binary)) {
                Some(Ok(snapshot)) => (
                    "ready",
                    Some(format!(
                        "RPC verified · context owned by Pi · auto compaction {}",
                        if snapshot.auto_compaction_enabled {
                            "on"
                        } else {
                            "off"
                        }
                    )),
                ),
                Some(Err(error)) => ("degraded", Some(error.to_string())),
                None => ("missing", None),
            }
        } else if path.is_some() {
            ("ready", None)
        } else {
            ("missing", None)
        };
        AgentProbe {
            id,
            name,
            path: path.as_ref().map(|p| p.display().to_string()),
            version,
            state,
            transport,
            capability,
        }
    })
    .collect()
}

fn find_binary(candidates: &[&str]) -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    candidates.iter().find_map(|candidate| {
        let path = if candidate.starts_with('.') {
            home.as_ref()?.join(candidate)
        } else {
            PathBuf::from(candidate)
        };
        path.is_file().then_some(path)
    })
}

fn read_version(path: &PathBuf) -> Option<String> {
    let mut command = ProcessCommand::new(path);
    command.arg("--version");
    if path.ends_with("pi") {
        command
            .env("PI_TELEMETRY", "0")
            .env("PI_SKIP_VERSION_CHECK", "1");
    }
    if path.ends_with("opencode") {
        command.env("OPENCODE_AUTO_SHARE", "false");
    }
    let output = command.output().ok()?;
    let text = if output.stdout.is_empty() {
        output.stderr
    } else {
        output.stdout
    };
    let version = String::from_utf8_lossy(&text).trim().to_owned();
    (!version.is_empty()).then_some(version)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let executable = std::env::current_exe()?;
            let runner = ensure_runner(
                &executable,
                &app_dir.join("runner-v1.sock"),
                &app_dir.join("pipeline.db"),
            )?;
            let bundled_package = app
                .path()
                .resource_dir()?
                .join("resources/seven-stage-product-delivery");
            runner.install_package(&bundled_package)?;
            app.manage(AppState { runner });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            dispatch,
            install_package,
            inspect_package,
            inspect_installed_package
        ])
        .run(tauri::generate_context!())
        .expect("error while running Agent Pipeline");
}
