use crate::tooling::{MetricValue, PipelineSnapshot, StageStatus};

#[test]
fn pipeline_snapshot_reports_stage_metrics() {
    let src = r#"
module tooling.capture

fn compute(x: Int, io: IO, net: Net) -> Int !{io, net} {
  let doubled = x + x
  doubled
}
"#;

    let snapshot = PipelineSnapshot::capture(src);
    assert_eq!(
        snapshot.module_path(),
        &["tooling".to_string(), "capture".to_string()]
    );

    let stages = snapshot.stages();
    let names: Vec<&str> = stages.iter().map(|stage| stage.name()).collect();
    assert_eq!(
        names,
        vec!["lexer", "parser", "resolve", "check", "lower", "ir"]
    );

    for stage in stages {
        assert!(matches!(stage.status(), StageStatus::Success));
    }

    let resolve_stage = snapshot
        .stages()
        .iter()
        .find(|stage| stage.name() == "resolve")
        .expect("resolve stage present");
    let capabilities = resolve_stage
        .metrics()
        .iter()
        .find(|metric| metric.key() == "capabilities")
        .expect("capability metric present");
    match capabilities.value() {
        MetricValue::List(values) => {
            assert!(values.contains(&"io".to_string()));
            assert!(values.contains(&"net".to_string()));
        }
        other => panic!("expected list metric, got {other:?}"),
    }

    let check_stage = snapshot
        .stages()
        .iter()
        .find(|stage| stage.name() == "check")
        .expect("check stage present");
    let diagnostics = check_stage
        .metrics()
        .iter()
        .find(|metric| metric.key() == "diagnostics")
        .expect("diagnostics metric");
    match diagnostics.value() {
        MetricValue::Integer(count) => assert_eq!(*count, 0),
        other => panic!("expected integer metric, got {other:?}"),
    }
}

#[test]
fn pipeline_snapshot_handles_parse_errors() {
    let src = "module broken\nfn missing_brace(x: Int) {";
    let snapshot = PipelineSnapshot::capture(src);

    assert!(snapshot.module_path().is_empty());
    let stages = snapshot.stages();
    assert_eq!(
        stages.len(),
        2,
        "only lexer and parser stages should be recorded"
    );
    assert!(matches!(stages[0].status(), StageStatus::Success));
    assert!(matches!(stages[1].status(), StageStatus::Failed { .. }));
}

#[test]
fn pipeline_snapshot_serializes_to_json() {
    let src = r#"
module tooling.json

type Alias = Int

fn identity(x: Int) -> Int {
  x
}
"#;

    let snapshot = PipelineSnapshot::capture(src);
    let json = snapshot.to_json_string();

    assert!(json.contains("\"module_path\""));
    assert!(json.contains("\"stages\""));
    assert!(json.contains("\"lexer\""));
    assert!(json.contains("\"resolve\""));
    assert!(json.contains("\"diagnostics\""));
    assert!(json.contains("\"ok\":true"));
}
