use std::fs;

use agent_pipeline_core::LoadedPackage;

#[test]
fn loads_a_directory_package_and_resolves_its_references() {
    let directory = tempfile::tempdir().unwrap();
    fs::create_dir_all(directory.path().join("pipelines")).unwrap();
    fs::create_dir_all(directory.path().join("prompts")).unwrap();
    fs::create_dir_all(directory.path().join("schemas")).unwrap();
    fs::write(directory.path().join("prompts/run.md"), "Run the node.").unwrap();
    fs::write(directory.path().join("schemas/output.json"), "{}").unwrap();
    fs::write(
        directory.path().join("agent-pipeline.package.yaml"),
        r#"
protocol: agent-pipeline.dev/v1alpha1
kind: PipelinePackage
metadata:
  name: example
  displayName: Example
  version: 1.0.0
pipelines:
  - path: pipelines/main.yaml
"#,
    )
    .unwrap();
    fs::write(
        directory.path().join("pipelines/main.yaml"),
        r#"
protocol: agent-pipeline.dev/v1alpha1
kind: Pipeline
metadata: { id: main }
nodes:
  - id: work
    type: agent
    prompt: ../prompts/run.md
    outputs:
      result: { schema: ../schemas/output.json }
edges: []
"#,
    )
    .unwrap();

    let package = LoadedPackage::load(directory.path()).unwrap();
    assert_eq!(package.manifest.metadata.name, "example");
    assert_eq!(package.pipelines[0].entry, "work");
}

#[test]
fn rejects_a_reference_that_escapes_the_package_root() {
    let directory = tempfile::tempdir().unwrap();
    fs::create_dir_all(directory.path().join("pipelines")).unwrap();
    fs::write(
        directory.path().join("agent-pipeline.package.yaml"),
        r#"
protocol: agent-pipeline.dev/v1alpha1
kind: PipelinePackage
metadata: { name: unsafe, version: 1.0.0 }
pipelines: [{ path: pipelines/main.yaml }]
"#,
    )
    .unwrap();
    fs::write(
        directory.path().join("pipelines/main.yaml"),
        r#"
protocol: agent-pipeline.dev/v1alpha1
kind: Pipeline
metadata: { id: main }
nodes: [{ id: work, type: agent, prompt: ../../outside.md }]
edges: []
"#,
    )
    .unwrap();

    assert!(LoadedPackage::load(directory.path()).is_err());
}
