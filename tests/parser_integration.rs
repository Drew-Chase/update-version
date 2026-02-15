//! Integration tests for version parsers

use semver::Version;
use std::fs;
use tempfile::TempDir;
use update_version::parsers::{
    WalkOptions, package_json_parser::PackageJsonParser, tauri_config_parser::TauriConfigParser,
    toml_parser::TomlParser, Parser,
};

// ============================================================================
// TOML Parser Integration Tests
// ============================================================================

#[test]
fn test_toml_update_version() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        r#"[package]
name = "test-crate"
version = "1.0.0"
edition = "2021"
"#,
    )
    .unwrap();

    let new_version = Version::parse("2.0.0").unwrap();
    let updated =
        TomlParser::update_version(temp_dir.path(), &new_version, &WalkOptions::default())
            .unwrap();

    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0], cargo_toml);

    let content = fs::read_to_string(&cargo_toml).unwrap();
    assert!(content.contains(r#"version = "2.0.0""#));
}

#[test]
fn test_toml_increment_version() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        r#"[package]
name = "test-crate"
version = "1.2.3"
"#,
    )
    .unwrap();

    TomlParser::increment_version(temp_dir.path(), &WalkOptions::default()).unwrap();

    let content = fs::read_to_string(&cargo_toml).unwrap();
    assert!(content.contains(r#"version = "1.2.4""#));
}

#[test]
fn test_toml_get_current_version() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        r#"[package]
name = "test-crate"
version = "3.2.1"
"#,
    )
    .unwrap();

    let version =
        TomlParser::get_current_version(temp_dir.path(), &WalkOptions::default()).unwrap();
    assert_eq!(version, Version::parse("3.2.1").unwrap());
}

#[test]
fn test_toml_multiple_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create nested Cargo.toml files
    let root_toml = temp_dir.path().join("Cargo.toml");
    let sub_dir = temp_dir.path().join("crates").join("sub-crate");
    fs::create_dir_all(&sub_dir).unwrap();
    let sub_toml = sub_dir.join("Cargo.toml");

    fs::write(
        &root_toml,
        r#"[package]
name = "root"
version = "1.0.0"
"#,
    )
    .unwrap();

    fs::write(
        &sub_toml,
        r#"[package]
name = "sub-crate"
version = "1.0.0"
"#,
    )
    .unwrap();

    let new_version = Version::parse("2.0.0").unwrap();
    let updated =
        TomlParser::update_version(temp_dir.path(), &new_version, &WalkOptions::default())
            .unwrap();

    assert_eq!(updated.len(), 2);

    let root_content = fs::read_to_string(&root_toml).unwrap();
    let sub_content = fs::read_to_string(&sub_toml).unwrap();

    assert!(root_content.contains(r#"version = "2.0.0""#));
    assert!(sub_content.contains(r#"version = "2.0.0""#));
}

// ============================================================================
// Package.json Parser Integration Tests
// ============================================================================

#[test]
fn test_package_json_update_version() {
    let temp_dir = TempDir::new().unwrap();
    let package_json = temp_dir.path().join("package.json");

    fs::write(
        &package_json,
        r#"{
  "name": "test-package",
  "version": "1.0.0",
  "description": "A test"
}"#,
    )
    .unwrap();

    let new_version = Version::parse("2.0.0").unwrap();
    let updated =
        PackageJsonParser::update_version(temp_dir.path(), &new_version, &WalkOptions::default())
            .unwrap();

    assert_eq!(updated.len(), 1);

    let content = fs::read_to_string(&package_json).unwrap();
    assert!(content.contains(r#""version": "2.0.0""#));
}

#[test]
fn test_package_json_increment_version() {
    let temp_dir = TempDir::new().unwrap();
    let package_json = temp_dir.path().join("package.json");

    fs::write(
        &package_json,
        r#"{
  "name": "test",
  "version": "0.1.0"
}"#,
    )
    .unwrap();

    PackageJsonParser::increment_version(temp_dir.path(), &WalkOptions::default()).unwrap();

    let content = fs::read_to_string(&package_json).unwrap();
    assert!(content.contains(r#""version": "0.1.1""#));
}

#[test]
fn test_package_json_get_current_version() {
    let temp_dir = TempDir::new().unwrap();
    let package_json = temp_dir.path().join("package.json");

    fs::write(
        &package_json,
        r#"{
  "name": "test",
  "version": "5.4.3"
}"#,
    )
    .unwrap();

    let version =
        PackageJsonParser::get_current_version(temp_dir.path(), &WalkOptions::default()).unwrap();
    assert_eq!(version, Version::parse("5.4.3").unwrap());
}

// ============================================================================
// Tauri Config Parser Integration Tests
// ============================================================================

#[test]
fn test_tauri_config_update_version() {
    let temp_dir = TempDir::new().unwrap();
    let tauri_conf = temp_dir.path().join("tauri.conf.json");

    fs::write(
        &tauri_conf,
        r#"{
  "productName": "My App",
  "version": "1.0.0",
  "identifier": "com.example.app"
}"#,
    )
    .unwrap();

    let new_version = Version::parse("2.0.0").unwrap();
    let updated =
        TauriConfigParser::update_version(temp_dir.path(), &new_version, &WalkOptions::default())
            .unwrap();

    assert_eq!(updated.len(), 1);

    let content = fs::read_to_string(&tauri_conf).unwrap();
    assert!(content.contains(r#""version": "2.0.0""#));
}

#[test]
fn test_tauri_config_strips_prerelease() {
    let temp_dir = TempDir::new().unwrap();
    let tauri_conf = temp_dir.path().join("tauri.conf.json");

    fs::write(
        &tauri_conf,
        r#"{
  "version": "1.0.0"
}"#,
    )
    .unwrap();

    // Tauri doesn't support prerelease versions, so they should be stripped
    let new_version = Version::parse("2.0.0-beta.1").unwrap();
    TauriConfigParser::update_version(temp_dir.path(), &new_version, &WalkOptions::default())
        .unwrap();

    let content = fs::read_to_string(&tauri_conf).unwrap();
    assert!(content.contains(r#""version": "2.0.0""#));
    assert!(!content.contains("beta"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_no_matching_files_returns_error() {
    let temp_dir = TempDir::new().unwrap();

    // Empty directory - no Cargo.toml
    let result = TomlParser::get_current_version(temp_dir.path(), &WalkOptions::default());
    assert!(result.is_err());
}

#[test]
fn test_no_version_in_file_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    // Cargo.toml without version field
    fs::write(
        &cargo_toml,
        r#"[package]
name = "test-crate"
edition = "2021"
"#,
    )
    .unwrap();

    let result = TomlParser::get_current_version(temp_dir.path(), &WalkOptions::default());
    assert!(result.is_err());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_preserves_file_structure() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    let original = r#"[package]
name = "test-crate"
version = "1.0.0"
edition = "2021"
description = "A test crate"

[dependencies]
serde = "1.0"
"#;

    fs::write(&cargo_toml, original).unwrap();

    let new_version = Version::parse("2.0.0").unwrap();
    TomlParser::update_version(temp_dir.path(), &new_version, &WalkOptions::default()).unwrap();

    let content = fs::read_to_string(&cargo_toml).unwrap();

    // Check that other content is preserved
    assert!(content.contains(r#"name = "test-crate""#));
    assert!(content.contains(r#"edition = "2021""#));
    assert!(content.contains(r#"description = "A test crate""#));
    assert!(content.contains(r#"serde = "1.0""#));
}

#[test]
fn test_version_with_prerelease() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        r#"[package]
name = "test"
version = "1.0.0-alpha.1"
"#,
    )
    .unwrap();

    let version =
        TomlParser::get_current_version(temp_dir.path(), &WalkOptions::default()).unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
    assert!(!version.pre.is_empty());
}

// ============================================================================
// Regression: No Double Quotes
// ============================================================================

#[test]
fn test_toml_no_double_quote_after_version() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        r#"[package]
name = "test"
version = "1.0.0"
edition = "2024"
"#,
    )
    .unwrap();

    let new_version = Version::parse("2.0.0").unwrap();
    TomlParser::update_version(temp_dir.path(), &new_version, &WalkOptions::default()).unwrap();

    let content = fs::read_to_string(&cargo_toml).unwrap();
    assert!(
        !content.contains(r#"version = "2.0.0""#)
            || !content.contains(r#"version = "2.0.0"""#),
        "Double quote detected in output: {}",
        content
    );
    // Ensure the closing quote is followed by a newline, not another quote
    assert!(
        content.contains("version = \"2.0.0\"\n") || content.contains("version = \"2.0.0\"\r\n"),
        "Version line not properly terminated: {}",
        content
    );
}

#[test]
fn test_package_json_no_double_quote_after_version() {
    let temp_dir = TempDir::new().unwrap();
    let package_json = temp_dir.path().join("package.json");

    fs::write(
        &package_json,
        r#"{
  "name": "test",
  "version": "1.0.0"
}"#,
    )
    .unwrap();

    let new_version = Version::parse("2.0.0").unwrap();
    PackageJsonParser::update_version(temp_dir.path(), &new_version, &WalkOptions::default())
        .unwrap();

    let content = fs::read_to_string(&package_json).unwrap();
    assert!(
        !content.contains(r#""version": "2.0.0"""#),
        "Double quote detected in output: {}",
        content
    );
}

// ============================================================================
// Ignore File Tests
// ============================================================================

#[test]
fn test_uvignore_excludes_directory() {
    let temp_dir = TempDir::new().unwrap();

    let root_toml = temp_dir.path().join("Cargo.toml");
    fs::write(
        &root_toml,
        r#"[package]
name = "root"
version = "1.0.0"
"#,
    )
    .unwrap();

    // Create .uvignore that ignores "vendor/"
    fs::write(temp_dir.path().join(".uvignore"), "vendor/\n").unwrap();

    // Create vendor/Cargo.toml which should be ignored
    let vendor_dir = temp_dir.path().join("vendor");
    fs::create_dir_all(&vendor_dir).unwrap();
    fs::write(
        vendor_dir.join("Cargo.toml"),
        r#"[package]
name = "vendored"
version = "1.0.0"
"#,
    )
    .unwrap();

    let new_version = Version::parse("2.0.0").unwrap();
    let updated =
        TomlParser::update_version(temp_dir.path(), &new_version, &WalkOptions::default())
            .unwrap();

    // Only root should be updated, vendor should be ignored
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0], root_toml);

    // Verify vendor file was NOT modified
    let vendor_content = fs::read_to_string(vendor_dir.join("Cargo.toml")).unwrap();
    assert!(vendor_content.contains(r#"version = "1.0.0""#));
}

#[test]
fn test_gitignore_excludes_directory() {
    let temp_dir = TempDir::new().unwrap();

    let root_toml = temp_dir.path().join("Cargo.toml");
    fs::write(
        &root_toml,
        r#"[package]
name = "root"
version = "1.0.0"
"#,
    )
    .unwrap();

    // Initialize a git repo (required for gitignore processing)
    fs::create_dir(temp_dir.path().join(".git")).unwrap();

    // Create .gitignore that ignores "target/"
    fs::write(temp_dir.path().join(".gitignore"), "target/\n").unwrap();

    // Create target/Cargo.toml which should be ignored
    let target_dir = temp_dir.path().join("target");
    fs::create_dir_all(&target_dir).unwrap();
    fs::write(
        target_dir.join("Cargo.toml"),
        r#"[package]
name = "built"
version = "1.0.0"
"#,
    )
    .unwrap();

    let new_version = Version::parse("2.0.0").unwrap();
    let updated =
        TomlParser::update_version(temp_dir.path(), &new_version, &WalkOptions::default())
            .unwrap();

    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0], root_toml);
}

#[test]
fn test_no_ignore_walks_everything() {
    let temp_dir = TempDir::new().unwrap();

    let root_toml = temp_dir.path().join("Cargo.toml");
    fs::write(
        &root_toml,
        r#"[package]
name = "root"
version = "1.0.0"
"#,
    )
    .unwrap();

    // Create .uvignore that ignores "sub/"
    fs::write(temp_dir.path().join(".uvignore"), "sub/\n").unwrap();

    let sub_dir = temp_dir.path().join("sub");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(
        sub_dir.join("Cargo.toml"),
        r#"[package]
name = "sub"
version = "1.0.0"
"#,
    )
    .unwrap();

    let options = WalkOptions { no_ignore: true };
    let new_version = Version::parse("2.0.0").unwrap();
    let updated = TomlParser::update_version(temp_dir.path(), &new_version, &options).unwrap();

    // Both files found since ignores are disabled
    assert_eq!(updated.len(), 2);
}

// ============================================================================
// Prerelease Increment Tests
// ============================================================================

#[test]
fn test_increment_prerelease_with_numeric_suffix() {
    // alpha.0 -> alpha.1
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        r#"[package]
name = "test"
version = "0.0.1-alpha.0"
"#,
    )
    .unwrap();

    TomlParser::increment_version(temp_dir.path(), &WalkOptions::default()).unwrap();

    let content = fs::read_to_string(&cargo_toml).unwrap();
    assert!(
        content.contains(r#"version = "0.0.1-alpha.1""#),
        "Expected 0.0.1-alpha.1, got: {}",
        content
    );
}

#[test]
fn test_increment_prerelease_without_numeric_suffix() {
    // alpha -> bump patch, keep label
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        r#"[package]
name = "test"
version = "0.0.1-alpha"
"#,
    )
    .unwrap();

    TomlParser::increment_version(temp_dir.path(), &WalkOptions::default()).unwrap();

    let content = fs::read_to_string(&cargo_toml).unwrap();
    assert!(
        content.contains(r#"version = "0.0.2-alpha""#),
        "Expected 0.0.2-alpha, got: {}",
        content
    );
}

#[test]
fn test_increment_prerelease_beta_numeric() {
    // beta.5 -> beta.6
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        r#"[package]
name = "test"
version = "1.0.0-beta.5"
"#,
    )
    .unwrap();

    TomlParser::increment_version(temp_dir.path(), &WalkOptions::default()).unwrap();

    let content = fs::read_to_string(&cargo_toml).unwrap();
    assert!(
        content.contains(r#"version = "1.0.0-beta.6""#),
        "Expected 1.0.0-beta.6, got: {}",
        content
    );
}
