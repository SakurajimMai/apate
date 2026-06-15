use std::fs;

#[test]
fn release_workflow_builds_on_main_push_and_tags() {
    let workflow = fs::read_to_string("../../.github/workflows/release.yml").unwrap();

    assert!(workflow.contains("branches: [main]"));
    assert!(workflow.contains("tags: [\"v*\"]"));
    assert!(workflow.contains("if: startsWith(github.ref, 'refs/tags/')"));
}
