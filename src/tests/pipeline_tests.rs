use super::helpers::*;
use super::*;
use crate::semantics::resolve::{CapabilityScope, PathKind, SymbolCategory};

#[test]
fn pipeline_module_roundtrip_covers_all_stages() {
    let src = r#"
module pipeline.demo

type TaskResult = Done(Int) | Failed(String)

fn orchestrate(job_id: Int, io: IO, net: Net) -> TaskResult !{io, net} {
  let pending = spawn network::fetch(job_id, net)
  using io::open("log.txt", io) {
    let status = await pending
    if status == 0 {
      TaskResult::Done(status)
    } else {
      TaskResult::Failed("error")
    }
  }
}

fn classify(result: TaskResult) -> Int {
  match result {
    TaskResult::Done(value) => value,
    TaskResult::Failed(_) => 0,
  }
}
"#;

    let module = parse(src);
    assert_eq!(
        module.name,
        vec!["pipeline".to_string(), "demo".to_string()]
    );
    assert_eq!(
        module.items.len(),
        3,
        "expected type alias and two functions"
    );

    let pretty = pretty::module_to_string(&module);
    assert!(
        pretty.contains("fn orchestrate(job_id: Int, io: IO, net: Net) -> TaskResult !{io, net}")
    );
    assert!(pretty.contains("fn classify(result: TaskResult) -> Int"));

    let lowered = lower::lower_module(&module);
    let dump = lower::hir_to_string(&lowered);
    assert!(dump.contains("spawn(network::fetch(job_id, net))"));
    assert!(dump.contains("await(pending)"));
    assert!(dump.contains(
        "if((status == 0), { TaskResult::Done(status); }, { TaskResult::Failed(\"error\"); })"
    ));

    let resolved = resolve::resolve_module(&module);
    let variants = resolved
        .adts
        .get("TaskResult")
        .expect("expected TaskResult variants");
    assert!(variants.contains(&"Done".to_string()));
    assert!(variants.contains(&"Failed".to_string()));

    let done_path = resolved
        .resolved_paths
        .iter()
        .find(|path| path.segments == vec!["TaskResult".to_string(), "Done".to_string()])
        .expect("expected resolved path for TaskResult::Done");
    assert!(matches!(done_path.kind, PathKind::Variant));
    let done_symbol = done_path.resolved.as_ref().expect("variant symbol");
    match &done_symbol.category {
        SymbolCategory::Variant { parent } => assert_eq!(parent, "TaskResult"),
        other => panic!("expected variant symbol, got {other:?}"),
    }

    for cap_name in ["io", "net"] {
        let cap = resolved
            .capabilities
            .iter()
            .find(|cap| cap.name == cap_name)
            .unwrap_or_else(|| panic!("missing capability {cap_name}"));
        match &cap.scope {
            CapabilityScope::Function {
                function,
                module_path,
            } => {
                assert_eq!(function, "orchestrate");
                assert_eq!(
                    module_path,
                    &vec!["pipeline".to_string(), "demo".to_string()]
                );
            }
            other => panic!("expected function capability scope, got {other:?}"),
        }
    }

    assert!(check::check_exhaustiveness(&module).is_empty());
}

#[test]
fn pipeline_reports_missing_variant_in_branchy_match() {
    let src = r#"
module pipeline.demo

type Response = Success | Failure | Retry

fn handle(resp: Response) -> Int {
  match resp {
    Success => 1,
    Retry => {
      let again = 0
      again
    },
  }
}
"#;

    let module = parse(src);
    let diags = check::check_exhaustiveness(&module);
    assert_eq!(diags.len(), 1, "expected a single diagnostic: {diags:?}");
    assert!(diags[0].message.contains("missing variants Failure"));
}
