// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const MANIFEST_MARKER: &str = "<!-- validator-quick-start:manifest -->";
const SOURCE_MARKER: &str = "<!-- validator-quick-start:source -->";
const INSTALLATION: &str = "[dependencies]\nqubit-validator = \"0.1\"";

fn extract_block(readme: &str, marker: &str, name: &str, language: &str) -> Result<String, String> {
    let mut markers = readme.match_indices(marker);
    let marker_at = markers.next();
    let count = usize::from(marker_at.is_some()) + markers.count();
    if count != 1 {
        return Err(format!("expected exactly one {name} marker, found {count}"));
    }

    let after_marker = &readme[marker_at.expect("count checked").0 + marker.len()..];
    let mut lines = after_marker.lines();
    let fence = lines
        .by_ref()
        .find(|line| line.trim_start().starts_with("```"))
        .ok_or_else(|| format!("{name} block has no fence"))?;
    if fence.trim() != format!("```{language}") {
        return Err(format!("{name} block must use a {language} fence"));
    }

    let mut body = Vec::new();
    for line in lines {
        if line.trim() == "```" {
            return Ok(body.join("\n"));
        }
        body.push(line);
    }
    Err(format!("{name} block has no closing fence"))
}

fn extract_quick_start(readme: &str) -> Result<(String, String), String> {
    let manifest = extract_block(readme, MANIFEST_MARKER, "manifest", "toml")?;
    let source = extract_block(readme, SOURCE_MARKER, "source", "rust")?;
    if manifest.trim() != INSTALLATION {
        return Err("installation block must contain only qubit-validator = \"0.1\"".into());
    }
    Ok((manifest, source))
}

struct TemporaryApp {
    path: PathBuf,
}

impl TemporaryApp {
    fn new(document: &str) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must follow Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rs-validator-readme-{document}-{}-{timestamp}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("create unique temporary Cargo application");
        Self { path }
    }
}

impl Drop for TemporaryApp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).expect("remove temporary Cargo application");
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    use super::INSTALLATION;
    use super::MANIFEST_MARKER;
    use super::SOURCE_MARKER;
    use super::TemporaryApp;
    use super::extract_quick_start;

    const VALID_README: &str = r#"
<!-- validator-quick-start:manifest -->
```toml
[dependencies]
qubit-validator = "0.1"
```

<!-- validator-quick-start:source -->
```rust
fn main() {}
```
"#;

    #[test]
    fn test_readme_quick_start_programs_run() {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut failures = Vec::new();

        for document in ["README.md", "README.zh_CN.md"] {
            let readme = fs::read_to_string(project_root.join(document))
                .unwrap_or_else(|error| panic!("{document}: could not read README: {error}"));
            let (_, source) = extract_quick_start(&readme).unwrap_or_else(|error| panic!("{document}: {error}"));
            let app = TemporaryApp::new(document);
            fs::create_dir(app.path.join("src")).expect("create temporary src directory");
            fs::write(app.path.join("src/main.rs"), source).expect("write README program");
            let manifest = format!(
                "[package]\nname = \"readme-example\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nqubit-validator = {{ path = {:?} }}\n",
                project_root.to_string_lossy()
            );
            fs::write(app.path.join("Cargo.toml"), manifest).expect("write temporary Cargo manifest");

            let output = Command::new("cargo")
                .args(["run", "--offline", "--quiet"])
                .current_dir(&app.path)
                .env("CARGO_TARGET_DIR", app.path.join("target"))
                .output()
                .expect("run temporary Cargo application");
            if !output.status.success() {
                failures.push(format!(
                    "{document}: cargo run --offline --quiet exited with {}\nstderr:\n{}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        assert!(failures.is_empty(), "{}", failures.join("\n\n"));
    }

    #[test]
    fn test_extract_quick_start_accepts_valid_blocks() {
        let (manifest, source) = extract_quick_start(VALID_README).expect("valid blocks");
        assert_eq!(manifest, INSTALLATION);
        assert_eq!(source, "fn main() {}");
    }

    #[test]
    fn test_extract_quick_start_rejects_missing_marker() {
        let readme = VALID_README.replace(MANIFEST_MARKER, "");
        assert_eq!(
            extract_quick_start(&readme).unwrap_err(),
            "expected exactly one manifest marker, found 0"
        );
    }

    #[test]
    fn test_extract_quick_start_rejects_missing_source_marker() {
        let readme = VALID_README.replace(SOURCE_MARKER, "");
        assert_eq!(
            extract_quick_start(&readme).unwrap_err(),
            "expected exactly one source marker, found 0"
        );
    }

    #[test]
    fn test_extract_quick_start_rejects_duplicate_marker() {
        let readme = format!("{VALID_README}\n{SOURCE_MARKER}");
        assert_eq!(
            extract_quick_start(&readme).unwrap_err(),
            "expected exactly one source marker, found 2"
        );
    }

    #[test]
    fn test_extract_quick_start_rejects_duplicate_manifest_marker() {
        let readme = format!("{VALID_README}\n{MANIFEST_MARKER}");
        assert_eq!(
            extract_quick_start(&readme).unwrap_err(),
            "expected exactly one manifest marker, found 2"
        );
    }

    #[test]
    fn test_extract_quick_start_rejects_wrong_fence_language() {
        let readme = VALID_README.replace("```toml", "```rust");
        assert_eq!(
            extract_quick_start(&readme).unwrap_err(),
            "manifest block must use a toml fence"
        );
    }

    #[test]
    fn test_extract_quick_start_rejects_wrong_source_fence_language() {
        let readme = VALID_README.replace("```rust", "```toml");
        assert_eq!(
            extract_quick_start(&readme).unwrap_err(),
            "source block must use a rust fence"
        );
    }

    #[test]
    fn test_extract_quick_start_rejects_wrong_installation() {
        let readme = VALID_README.replace("qubit-validator = \"0.1\"", "qubit-validator = \"0.2\"");
        assert_eq!(
            extract_quick_start(&readme).unwrap_err(),
            "installation block must contain only qubit-validator = \"0.1\""
        );
    }

    #[test]
    fn test_extract_quick_start_rejects_extra_dependency() {
        let readme = VALID_README.replace("qubit-validator = \"0.1\"", "qubit-validator = \"0.1\"\nother = \"1\"");
        assert_eq!(
            extract_quick_start(&readme).unwrap_err(),
            "installation block must contain only qubit-validator = \"0.1\""
        );
    }
}
