use std::path::PathBuf;

use agent_pipeline_runner::PiRpcAdapter;

fn main() {
    let binary = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/opt/homebrew/bin/pi"));
    let snapshot = PiRpcAdapter::probe(&binary).expect("Pi RPC probe failed");
    println!("{snapshot:?}");
}
