use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn repository_keeps_only_rust_implementation_records() {
    let root = workspace_root();
    let forbidden_paths = [
        ["apate", ".sln"].concat(),
        ["apate", "/apate.csproj"].concat(),
        ["apate", "/ApateUI.cs"].concat(),
        ["apate", "/AboutUI.cs"].concat(),
        ["apate", "/InfoBox.cs"].concat(),
        ["apate", "/Program.cs"].concat(),
        ["apate", "/Resources/mask.mp4"].concat(),
    ];

    for relative_path in &forbidden_paths {
        assert!(
            !root.join(relative_path).exists(),
            "non-rust implementation file still exists: {relative_path}"
        );
    }

    let searchable_files = collect_searchable_files(&root);
    let forbidden_terms = [
        ".NET",
        "WinForms",
        "C#/WinForms",
        "v1.4.2",
        "apate/Resources",
        "旧版",
        "旧版本",
        "旧格式",
        "legacy",
        "Legacy",
    ];

    for path in searchable_files {
        let content = fs::read_to_string(&path).unwrap();
        for term in forbidden_terms {
            assert!(
                !content.contains(term),
                "{} still contains non-rust record: {term}",
                path.strip_prefix(&root).unwrap().display()
            );
        }
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn collect_searchable_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_searchable_files_inner(root, &mut files);
    files
}

fn collect_searchable_files_inner(path: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if file_name == ".git" || file_name == "target" {
            continue;
        }

        if path.is_dir() {
            collect_searchable_files_inner(&path, files);
            continue;
        }

        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            continue;
        };
        if path.ends_with("rust_only_repo.rs") {
            continue;
        }
        if matches!(
            extension,
            "md" | "rs" | "toml" | "yml" | "yaml" | "gitignore"
        ) {
            files.push(path);
        }
    }
}
