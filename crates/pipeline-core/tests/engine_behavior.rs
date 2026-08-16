use agent_pipeline_core::{Command, Engine, NodeStatus};

#[test]
fn review_feedback_appends_a_new_implement_attempt_and_event() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("pipeline.db")).expect("engine");

    let before = engine.snapshot().expect("snapshot");
    let before_events = before.event_count;

    let after = engine
        .dispatch(Command::RequestChanges {
            node_id: "review".into(),
            reason: "异步退款必须校验幂等键".into(),
        })
        .expect("request changes");

    let implement = after.node("implement").expect("implement node");
    assert_eq!(implement.attempt, 2);
    assert_eq!(implement.status, NodeStatus::Running);
    assert_eq!(after.event_count, before_events + 1);
    assert_eq!(
        after.artifacts.last().unwrap().title,
        "Review feedback · Attempt 1"
    );
}

#[test]
fn state_survives_reopening_the_engine() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("pipeline.db");
    let engine = Engine::open(&path).expect("engine");
    engine
        .dispatch(Command::SelectNode {
            node_id: "spec".into(),
        })
        .expect("select node");
    drop(engine);

    let reopened = Engine::open(&path).expect("reopen");
    assert_eq!(reopened.snapshot().unwrap().selected_node_id, "spec");
}

#[test]
fn approved_feedback_cycle_reaches_smoke_test_and_publishes_release_artifacts() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("pipeline.db")).expect("engine");
    engine
        .dispatch(Command::RequestChanges {
            node_id: "review".into(),
            reason: "补齐幂等校验".into(),
        })
        .unwrap();
    engine.dispatch(Command::Advance).unwrap();
    engine
        .dispatch(Command::Approve {
            node_id: "review".into(),
        })
        .unwrap();
    engine.dispatch(Command::Advance).unwrap();
    let completed = engine.dispatch(Command::Advance).unwrap();

    assert_eq!(completed.status, "completed");
    assert!(
        completed
            .nodes
            .iter()
            .all(|node| node.status == NodeStatus::Completed)
    );
    assert!(
        completed
            .artifacts
            .iter()
            .any(|artifact| artifact.id == "deployment-receipt")
    );
    assert!(
        completed
            .artifacts
            .iter()
            .any(|artifact| artifact.id == "smoke-report")
    );
}
