use agent_pipeline_core::PipelinePackage;

#[test]
fn bounded_feedback_loop_is_valid() {
    let package = PipelinePackage::parse(
        r#"
protocol: agent-pipeline.dev/v1alpha1
kind: PipelinePackage
metadata:
  name: seven-stage-product-delivery
  version: 0.1.0
pipelines:
  - id: product-delivery
    entry: grill
    nodes:
      - { id: grill, type: agent }
      - { id: implement, type: agent }
      - { id: review, type: gate }
    edges:
      - { from: grill, to: implement }
      - { from: implement, to: review }
      - from: review
        to: implement
        when: changes_requested
        loop: { max_iterations: 3, on_exhausted: attention }
"#,
    )
    .expect("valid package");

    assert_eq!(package.metadata.name, "seven-stage-product-delivery");
    assert!(package.validate().is_ok());
}

#[test]
fn unbounded_feedback_loop_is_rejected() {
    let package = PipelinePackage::parse(
        r#"
protocol: agent-pipeline.dev/v1alpha1
kind: PipelinePackage
metadata: { name: unsafe-loop, version: 0.1.0 }
pipelines:
  - id: unsafe
    entry: implement
    nodes:
      - { id: implement, type: agent }
      - { id: review, type: gate }
    edges:
      - { from: implement, to: review }
      - { from: review, to: implement, when: changes_requested }
"#,
    )
    .expect("parse package");

    assert!(
        package
            .validate()
            .unwrap_err()
            .to_string()
            .contains("bounded loop")
    );
}
