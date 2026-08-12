use baron_core::operations::{load_runbook, relevant_to_task, render_bounded_context};

#[test]
fn runbook_is_loaded_only_for_runtime_tasks_and_stays_bounded() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("docs/baron/operations/RUNBOOK.md");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "# Runbook\n\n## Scope\nlocal app\n\n## Start command\ncargo run\n\n## Readiness\nhealth endpoint observed\n\n## Real interface\nHTTP API\n\n## Runtime evidence\nlogs and response\n\n## Owned cleanup\ncurrent process only\n").unwrap();
    assert!(relevant_to_task("run the application end-to-end"));
    assert!(!relevant_to_task("fix a typo in the README"));
    let runbook = load_runbook(temp.path()).unwrap().unwrap();
    assert!(render_bounded_context(&runbook).contains("Application Runbook"));
}
