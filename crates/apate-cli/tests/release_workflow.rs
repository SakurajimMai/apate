use std::fs;

#[test]
fn release_workflow_builds_on_main_push_and_tags() {
    let workflow = fs::read_to_string("../../.github/workflows/release.yml").unwrap();

    assert!(workflow.contains("branches: [main]"));
    assert!(workflow.contains("tags: [\"v*\"]"));
    assert!(workflow.contains("name: Publish tagged release"));
    assert!(workflow.contains("if: startsWith(github.ref, 'refs/tags/')"));
    assert!(workflow.contains("name: Publish latest prerelease"));
    assert!(workflow.contains("if: github.ref == 'refs/heads/main'"));
    assert!(workflow.contains("tag_name: latest"));
    assert!(workflow.contains("prerelease: true"));
    assert!(workflow.contains("overwrite_files: true"));
    assert!(workflow.contains("tag_name: ${{ github.ref_name }}"));
}
