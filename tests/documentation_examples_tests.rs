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
const GUIDE_MARKERS: [(&str, &str); 4] = [
    ("<!-- validator-guide:rules -->", "rules"),
    ("<!-- validator-guide:startup -->", "startup"),
    ("<!-- validator-guide:service -->", "service"),
    ("<!-- validator-guide:service-continuation -->", "service-continuation"),
];
const ADVANCED_GUIDE_MARKERS: [(&str, &str); 2] = [
    ("<!-- validator-guide:confirmation -->", "confirmation"),
    ("<!-- validator-guide:max-length -->", "max-length"),
];
const DOMAIN_RULE_GUIDE_MARKER: &str = "<!-- validator-guide:domain-rule -->";

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

fn extract_guide_block(guide: &str, marker: &str, name: &str) -> Result<String, String> {
    let mut markers = guide.match_indices(marker);
    let marker_at = markers.next();
    let count = usize::from(marker_at.is_some()) + markers.count();
    if count != 1 {
        return Err(format!("expected exactly one {name} marker, found {count}"));
    }

    let after_marker = &guide[marker_at.expect("count checked").0 + marker.len()..];
    let mut lines = after_marker.lines();
    let fence = lines
        .by_ref()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| format!("{name} block has no fence"))?;
    if fence.trim() != "```rust" {
        return Err(format!("{name} block must use a rust fence"));
    }

    let mut body = Vec::new();
    for line in lines {
        if line.trim() == "```" {
            return Ok(body.join("\n"));
        }
        if line.trim_start().starts_with("```")
            || GUIDE_MARKERS
                .iter()
                .any(|(other_marker, _)| line.contains(other_marker))
        {
            break;
        }
        body.push(line);
    }
    Err(format!("{name} block has no closing fence"))
}

fn extract_guide_basic_path(guide: &str) -> Result<[String; 4], String> {
    let blocks = GUIDE_MARKERS
        .map(|(marker, name)| extract_guide_block(guide, marker, name))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    blocks.try_into().map_err(|_| "expected four guide blocks".to_owned())
}

fn extract_guide_advanced_modules(guide: &str, document: &str) -> Result<[String; 2], String> {
    ADVANCED_GUIDE_MARKERS
        .map(|(marker, name)| extract_advanced_guide_block(guide, document, marker, name))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| format!("{document}: expected two advanced guide modules"))
}

fn extract_advanced_guide_block(guide: &str, document: &str, marker: &str, name: &str) -> Result<String, String> {
    let context = format!("{document}: {name} ({marker})");
    let mut markers = guide.match_indices(marker);
    let marker_at = markers.next();
    let count = usize::from(marker_at.is_some()) + markers.count();
    if count != 1 {
        return Err(format!("{context}: expected exactly one marker, found {count}"));
    }

    let marker_end = marker_at.expect("count checked").0 + marker.len();
    let after_marker = guide[marker_end..]
        .strip_prefix("\r\n")
        .or_else(|| guide[marker_end..].strip_prefix('\n'))
        .unwrap_or(&guide[marker_end..]);
    let mut lines = after_marker.lines();
    let fence = lines
        .next()
        .ok_or_else(|| format!("{context}: marker must be followed immediately by a rust fence"))?;
    if fence.trim() != "```rust" {
        return Err(format!(
            "{context}: marker must be followed immediately by a rust fence"
        ));
    }

    let mut body = Vec::new();
    for line in lines {
        if line.trim() == "```" {
            return Ok(body.join("\n"));
        }
        if line.trim_start().starts_with("```") {
            return Err(format!("{context}: rust fence must close with ```"));
        }
        body.push(line);
    }
    Err(format!("{context}: rust fence has no closing fence"))
}

#[must_use = "the temporary application directory is removed when its guard is dropped"]
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
    use std::path::Path;
    use std::path::PathBuf;
    use std::process::Command;

    use super::ADVANCED_GUIDE_MARKERS;
    use super::DOMAIN_RULE_GUIDE_MARKER;
    use super::GUIDE_MARKERS;
    use super::INSTALLATION;
    use super::MANIFEST_MARKER;
    use super::SOURCE_MARKER;
    use super::TemporaryApp;
    use super::extract_advanced_guide_block;
    use super::extract_guide_advanced_modules;
    use super::extract_guide_basic_path;
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

    const VALID_GUIDE: &str = r#"
<!-- validator-guide:rules -->
```rust
pub struct Rule;
```
<!-- validator-guide:startup -->
```rust
pub fn startup() {}
```
<!-- validator-guide:service -->
```rust
pub fn service() {}
```
<!-- validator-guide:service-continuation -->
```rust
pub fn continuation() {}
```
"#;

    const VALID_ADVANCED_GUIDE: &str = r#"
<!-- validator-guide:confirmation -->
```rust
pub fn confirmation() {}
```
<!-- validator-guide:max-length -->
```rust
pub fn max_length() {}
```
"#;

    const VALID_DOMAIN_RULE_GUIDE: &str = r#"<!-- validator-guide:domain-rule -->
```rust
fn main() {}
```"#;

    #[test]
    fn test_extract_guide_advanced_modules_rejects_missing_marker() {
        for (marker, _) in ADVANCED_GUIDE_MARKERS {
            let guide = VALID_ADVANCED_GUIDE.replace(marker, "");
            let error = extract_guide_advanced_modules(&guide, "doc/user_guide.md").unwrap_err();
            assert!(error.contains("doc/user_guide.md"), "{error}");
            assert!(error.contains(marker), "{error}");
            assert!(error.contains("found 0"), "{error}");
        }
    }

    #[test]
    fn test_extract_guide_advanced_modules_rejects_duplicate_marker() {
        for (marker, _) in ADVANCED_GUIDE_MARKERS {
            let guide = format!("{VALID_ADVANCED_GUIDE}\n{marker}");
            let error = extract_guide_advanced_modules(&guide, "doc/user_guide.md").unwrap_err();
            assert!(error.contains("doc/user_guide.md"), "{error}");
            assert!(error.contains(marker), "{error}");
            assert!(error.contains("found 2"), "{error}");
        }
    }

    #[test]
    fn test_extract_guide_advanced_modules_rejects_wrong_fence() {
        for (marker, _) in ADVANCED_GUIDE_MARKERS {
            let guide = VALID_ADVANCED_GUIDE.replace(&format!("{marker}\n```rust"), &format!("{marker}\n```toml"));
            let error = extract_guide_advanced_modules(&guide, "doc/user_guide.md").unwrap_err();
            assert!(error.contains("doc/user_guide.md"), "{error}");
            assert!(error.contains(marker), "{error}");
            assert!(error.contains("rust fence"), "{error}");
        }
    }

    #[test]
    fn test_extract_guide_advanced_modules_accepts_crlf() {
        let guide = VALID_ADVANCED_GUIDE.replace('\n', "\r\n");
        let [confirmation, max_length] =
            extract_guide_advanced_modules(&guide, "doc/user_guide.md").expect("extract CRLF guide");
        assert_eq!(confirmation, "pub fn confirmation() {}");
        assert_eq!(max_length, "pub fn max_length() {}");
    }

    #[test]
    fn test_extract_domain_rule_guide_rejects_missing_marker() {
        let guide = VALID_DOMAIN_RULE_GUIDE.replace(DOMAIN_RULE_GUIDE_MARKER, "");
        let error = extract_advanced_guide_block(&guide, "doc/user_guide.md", DOMAIN_RULE_GUIDE_MARKER, "domain-rule")
            .unwrap_err();
        assert!(error.contains(DOMAIN_RULE_GUIDE_MARKER), "{error}");
        assert!(error.contains("found 0"), "{error}");
    }

    #[test]
    fn test_extract_domain_rule_guide_rejects_duplicate_marker() {
        let guide = format!("{VALID_DOMAIN_RULE_GUIDE}\n{DOMAIN_RULE_GUIDE_MARKER}");
        let error = extract_advanced_guide_block(&guide, "doc/user_guide.md", DOMAIN_RULE_GUIDE_MARKER, "domain-rule")
            .unwrap_err();
        assert!(error.contains(DOMAIN_RULE_GUIDE_MARKER), "{error}");
        assert!(error.contains("found 2"), "{error}");
    }

    #[test]
    fn test_extract_domain_rule_guide_rejects_wrong_fence() {
        let guide = VALID_DOMAIN_RULE_GUIDE.replace(
            &format!("{DOMAIN_RULE_GUIDE_MARKER}\n```rust"),
            &format!("{DOMAIN_RULE_GUIDE_MARKER}\n```toml"),
        );
        let error = extract_advanced_guide_block(&guide, "doc/user_guide.md", DOMAIN_RULE_GUIDE_MARKER, "domain-rule")
            .unwrap_err();
        assert!(error.contains(DOMAIN_RULE_GUIDE_MARKER), "{error}");
        assert!(error.contains("rust fence"), "{error}");
    }

    #[test]
    fn test_extract_domain_rule_guide_rejects_unclosed_fence() {
        let guide = VALID_DOMAIN_RULE_GUIDE.replace("fn main() {}\n```", "fn main() {}");
        let error = extract_advanced_guide_block(&guide, "doc/user_guide.md", DOMAIN_RULE_GUIDE_MARKER, "domain-rule")
            .unwrap_err();
        assert!(error.contains(DOMAIN_RULE_GUIDE_MARKER), "{error}");
        assert!(error.contains("no closing fence"), "{error}");
    }

    fn run_user_guide_basic_path(target_dir: &Path) -> Vec<String> {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut failures = Vec::new();

        for (document, directory) in [
            ("doc/user_guide.md", "guide_en"),
            ("doc/user_guide.zh_CN.md", "guide_zh"),
        ] {
            let guide = fs::read_to_string(project_root.join(document))
                .unwrap_or_else(|error| panic!("{document}: could not read guide: {error}"));
            let [rules, startup, service, continuation] =
                extract_guide_basic_path(&guide).unwrap_or_else(|error| panic!("{document}: {error}"));
            let app = TemporaryApp::new(directory);
            let profile = app.path.join("src/profile");
            fs::create_dir_all(&profile).expect("create temporary profile directory");
            fs::write(profile.join("rules.rs"), rules).expect("write rules module");
            fs::write(profile.join("startup.rs"), startup).expect("write startup module");
            fs::write(
                profile.join("service.rs"),
                format!(
                    "{service}\n{continuation}\n{}",
                    r#"
#[cfg(test)]
mod documented_flow_tests {
    use std::cell::Cell;

    use super::ProfileStore;
    use super::UpdateProfileResult;
    use super::update_profile;

    #[derive(Default)]
    struct CountingStore(Cell<usize>);

    impl ProfileStore for CountingStore {
        fn save_display_name(
            &self,
            _: &str,
            _: &str,
        ) -> Result<(), Box<dyn std::error::Error>> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }
    }

    #[test]
    fn test_accepted_name_is_saved_and_blank_name_is_rejected()
        -> Result<(), Box<dyn std::error::Error>>
    {
        let rule = crate::profile::startup::bind_display_name_rule()?;
        let store = CountingStore::default();
        assert!(matches!(
            update_profile(&store, &rule, "user-1", "Ada")?,
            UpdateProfileResult::Updated
        ));
        assert_eq!(store.0.get(), 1);

        let UpdateProfileResult::Rejected(report) =
            update_profile(&store, &rule, "user-1", "   ")?
        else {
            panic!("blank name must be rejected");
        };
        assert!(!report.is_truncated());
        assert!(!report.is_valid());
        assert_eq!(report.violations().len(), 1);
        assert_eq!(report.violations()[0].code().as_str(), "text.blank");
        assert_eq!(report.violations()[0].path().render(), "display_name");
        assert_eq!(store.0.get(), 1);
        Ok(())
    }
}
"#
                ),
            )
            .expect("write service module");
            fs::write(profile.join("mod.rs"), "mod rules;\nmod startup;\nmod service;\n")
                .expect("write profile module");
            fs::write(app.path.join("src/main.rs"), "mod profile;\nfn main() {}\n")
                .expect("write application entry point");
            let manifest = format!(
                "[package]\nname = \"{directory}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nqubit-validator = {{ path = {:?} }}\n",
                project_root.to_string_lossy()
            );
            fs::write(app.path.join("Cargo.toml"), manifest).expect("write temporary Cargo manifest");

            let output = Command::new("cargo")
                .args(["test", "--offline"])
                .current_dir(&app.path)
                .env("CARGO_TARGET_DIR", target_dir)
                .output()
                .unwrap_or_else(|error| panic!("{document}: failed to start cargo test --offline: {error}"));
            if !output.status.success() {
                failures.push(format!(
                    "{document}: cargo test --offline exited with {}\nstdout:\n{}\nstderr:\n{}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.contains("accepted_name_is_saved_and_blank_name_is_rejected") || !stdout.contains("1 passed")
                {
                    failures.push(format!(
                        "{document}: cargo test --offline did not report the expected test and result\nstdout:\n{stdout}\nstderr:\n{}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
        }

        failures
    }

    #[test]
    fn test_extract_guide_basic_path_rejects_missing_marker() {
        for (marker, name) in GUIDE_MARKERS {
            let guide = VALID_GUIDE.replace(marker, "");
            assert_eq!(
                extract_guide_basic_path(&guide).unwrap_err(),
                format!("expected exactly one {name} marker, found 0")
            );
        }
    }

    #[test]
    fn test_extract_guide_basic_path_rejects_duplicate_marker() {
        for (marker, name) in GUIDE_MARKERS {
            let guide = format!("{VALID_GUIDE}\n{marker}");
            assert_eq!(
                extract_guide_basic_path(&guide).unwrap_err(),
                format!("expected exactly one {name} marker, found 2")
            );
        }
    }

    #[test]
    fn test_extract_guide_basic_path_rejects_wrong_fence_language() {
        for (marker, name) in GUIDE_MARKERS {
            let guide = VALID_GUIDE.replace(&format!("{marker}\n```rust"), &format!("{marker}\n```toml"));
            assert_eq!(
                extract_guide_basic_path(&guide).unwrap_err(),
                format!("{name} block must use a rust fence")
            );
        }
    }

    #[test]
    fn test_extract_guide_basic_path_rejects_unclosed_fence() {
        for (line, name) in [
            ("pub struct Rule;", "rules"),
            ("pub fn startup() {}", "startup"),
            ("pub fn service() {}", "service"),
            ("pub fn continuation() {}", "service-continuation"),
        ] {
            let guide = VALID_GUIDE.replace(&format!("{line}\n```"), line);
            assert_eq!(
                extract_guide_basic_path(&guide).unwrap_err(),
                format!("{name} block has no closing fence")
            );
        }
    }

    fn run_user_guide_advanced_modules(target_dir: &Path) -> Vec<String> {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut failures = Vec::new();

        for (document, directory) in [
            ("doc/user_guide.md", "guide_advanced_en"),
            ("doc/user_guide.zh_CN.md", "guide_advanced_zh"),
        ] {
            let guide = fs::read_to_string(project_root.join(document))
                .unwrap_or_else(|error| panic!("{document}: could not read guide: {error}"));
            let [confirmation, max_length] =
                extract_guide_advanced_modules(&guide, document).unwrap_or_else(|error| panic!("{error}"));
            let app = TemporaryApp::new(directory);
            let profile = app.path.join("src/profile");
            fs::create_dir_all(&profile).expect("create temporary profile directory");
            fs::write(profile.join("confirmation.rs"), confirmation).expect("write confirmation module");
            fs::write(profile.join("max_length.rs"), max_length).expect("write max-length module");
            fs::write(profile.join("mod.rs"), "pub mod confirmation;\npub mod max_length;\n")
                .expect("write profile module");
            fs::write(
                app.path.join("src/main.rs"),
                r#"
mod profile;

#[cfg(test)]
mod documented_advanced_examples_tests {
    use qubit_validator::BoundValidationContext;
    use qubit_validator::ValidationOutcome;
    use qubit_validator::ValidationValue;

    use crate::profile::confirmation::bind_confirmation_rule;
    use crate::profile::confirmation::check_confirmation;
    use crate::profile::max_length::bind_max_length_rule;

    fn assert_invalid_code(outcome: ValidationOutcome, expected: &str) {
        let ValidationOutcome::Invalid(violations) = outcome else {
            panic!("expected Invalid outcome");
        };
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code().as_str(), expected);
    }

    #[test]
    fn test_confirmation_module_accepts_matching_email_and_rejects_mismatch() {
        let rule = bind_confirmation_rule().expect("bind confirmation rule");
        assert!(matches!(
            check_confirmation(&rule, "ada@example.com", "ada@example.com")
                .expect("validate matching email"),
            ValidationOutcome::Valid
        ));
        assert_invalid_code(
            check_confirmation(&rule, "ada@example.com", "grace@example.com")
                .expect("validate mismatching email"),
            "email.confirmation_mismatch",
        );
    }

    #[test]
    fn test_max_length_module_counts_unicode_scalar_values() {
        let rule = bind_max_length_rule(3).expect("bind maximum length rule");
        assert!(matches!(
            rule.validate(ValidationValue::Text("猫é🙂"), &BoundValidationContext::new(&[]))
                .expect("validate three scalar values"),
            ValidationOutcome::Valid
        ));
        assert_invalid_code(
            rule.validate(ValidationValue::Text("猫é🙂a"), &BoundValidationContext::new(&[]))
                .expect("validate four scalar values"),
            "text.too_long",
        );
    }
}

fn main() {}
"#,
            )
            .expect("write application entry point");
            let manifest = format!(
                "[package]\nname = \"{directory}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nqubit-validator = {{ path = {:?} }}\n",
                project_root.to_string_lossy()
            );
            fs::write(app.path.join("Cargo.toml"), manifest).expect("write temporary Cargo manifest");

            let output = Command::new("cargo")
                .args(["test", "--offline"])
                .current_dir(&app.path)
                .env("CARGO_TARGET_DIR", target_dir)
                .output()
                .unwrap_or_else(|error| panic!("{document}: failed to start cargo test --offline: {error}"));
            if !output.status.success() {
                failures.push(format!(
                    "{document}: cargo test --offline exited with {}\nstdout:\n{}\nstderr:\n{}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.contains("2 passed") {
                    failures.push(format!(
                        "{document}: cargo test --offline did not report both advanced tests passing\nstdout:\n{stdout}\nstderr:\n{}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
        }

        failures
    }

    fn run_user_guide_domain_rule_programs(target_dir: &Path) -> Vec<String> {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut failures = Vec::new();

        for (document, directory) in [
            ("doc/user_guide.md", "guide_domain_rule_en"),
            ("doc/user_guide.zh_CN.md", "guide_domain_rule_zh"),
        ] {
            let guide = fs::read_to_string(project_root.join(document))
                .unwrap_or_else(|error| panic!("{document}: could not read guide: {error}"));
            let program = extract_advanced_guide_block(&guide, document, DOMAIN_RULE_GUIDE_MARKER, "domain-rule")
                .unwrap_or_else(|error| panic!("{error}"));
            let app = TemporaryApp::new(directory);
            fs::create_dir(app.path.join("src")).expect("create temporary src directory");
            fs::write(app.path.join("src/main.rs"), program).expect("write domain-rule guide program");
            let manifest = format!(
                "[package]\nname = \"{directory}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nqubit-validator = {{ path = {:?} }}\n",
                project_root.to_string_lossy()
            );
            fs::write(app.path.join("Cargo.toml"), manifest).expect("write temporary Cargo manifest");

            let output = Command::new("cargo")
                .args(["run", "--offline", "--quiet"])
                .current_dir(&app.path)
                .env("CARGO_TARGET_DIR", target_dir)
                .output()
                .unwrap_or_else(|error| panic!("{document}: failed to start cargo run --offline: {error}"));
            if !output.status.success() {
                failures.push(format!(
                    "{document}: cargo run --offline --quiet exited with {}\nstdout:\n{}\nstderr:\n{}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        failures
    }

    fn run_readme_quick_start_programs(target_dir: &Path) -> Vec<String> {
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
                .env("CARGO_TARGET_DIR", target_dir)
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

        failures
    }

    #[test]
    fn test_documentation_programs_run() {
        let cache = TemporaryApp::new("shared-target");
        let target_dir = cache.path.join("target");
        let mut failures = Vec::new();
        failures.extend(run_user_guide_basic_path(&target_dir));
        failures.extend(run_user_guide_advanced_modules(&target_dir));
        failures.extend(run_user_guide_domain_rule_programs(&target_dir));
        failures.extend(run_readme_quick_start_programs(&target_dir));
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
