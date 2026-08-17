use agent_pipeline_core::{EdgeDefinition, LoopPolicy};

#[test]
fn bounded_loop_uses_the_public_camel_case_wire_contract() {
    let edge = EdgeDefinition {
        from: "review".into(),
        to: "implement".into(),
        when: Some("changes_requested".into()),
        loop_policy: Some(LoopPolicy {
            max_iterations: 3,
            on_exhausted: "attention".into(),
        }),
        handoff: vec!["review-report".into()],
    };

    let json = serde_json::to_value(edge).unwrap();
    assert_eq!(json["loop"]["maxIterations"], 3);
    assert_eq!(json["loop"]["onExhausted"], "attention");
    assert!(json["loop"].get("max_iterations").is_none());
}
