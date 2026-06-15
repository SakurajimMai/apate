use std::fs;

#[test]
fn release_workflow_builds_on_main_push_and_tags() {
    let workflow = fs::read_to_string("../../.github/workflows/release.yml").unwrap();

    assert!(workflow.contains("branches: [main]"));
    assert!(workflow.contains("tags: [\"v*\"]"));
    assert!(workflow.contains("target: x86_64-pc-windows-msvc"));
    assert!(workflow.contains("target: x86_64-unknown-linux-gnu"));
    assert!(workflow.contains("Copy-Item \"target\\${{ matrix.target }}\\release\\apate.exe\""));
    assert!(workflow.contains("cp target/${{ matrix.target }}/release/apate dist/apate"));
    assert!(workflow.contains("name: Publish tagged release"));
    assert!(workflow.contains("if: startsWith(github.ref, 'refs/tags/')"));
    assert!(workflow.contains("name: Publish latest prerelease"));
    assert!(workflow.contains("if: github.ref == 'refs/heads/main'"));
    assert!(workflow.contains("tag_name: latest"));
    assert!(workflow.contains("prerelease: true"));
    assert!(workflow.contains("overwrite_files: true"));
    assert!(workflow.contains("tag_name: ${{ github.ref_name }}"));
    assert!(workflow.contains("dist/**/*.zip"));
    assert!(workflow.contains("dist/**/*.tar.gz"));
    assert!(!workflow.contains("Copy-Item \"CHANGELOG.md\""));
    assert!(!workflow.contains("cp CHANGELOG.md dist/CHANGELOG.md"));
}
