use std::path::PathBuf;

use agent_pipeline_core::LoadedPackage;

fn main() {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: validate-package <package-directory>");
    let package = LoadedPackage::load(path).expect("package validation failed");
    println!(
        "{} {}: {} pipeline(s) valid",
        package.manifest.metadata.name,
        package.manifest.metadata.version,
        package.pipelines.len()
    );
}
