//! Example: Update version across all supported file types
//!
//! Run with: cargo run --example update_all_types

use anyhow::Result;
use semver::Version;
use update_version::parsers::{
    WalkOptions, package_json_parser::PackageJsonParser, tauri_config_parser::TauriConfigParser,
    toml_parser::TomlParser, Parser,
};

fn main() -> Result<()> {
    let new_version = Version::parse("3.0.0")?;
    let project_path = "./";
    let options = WalkOptions::default();

    println!("Updating all files to version {}...\n", new_version);

    // Update Cargo.toml files
    match TomlParser::update_version(project_path, &new_version, &options) {
        Ok(files) => {
            println!("TOML files updated:");
            for file in files {
                println!("  - {}", file.display());
            }
        }
        Err(e) => println!("No TOML files found: {}", e),
    }

    // Update package.json files
    match PackageJsonParser::update_version(project_path, &new_version, &options) {
        Ok(files) => {
            println!("\npackage.json files updated:");
            for file in files {
                println!("  - {}", file.display());
            }
        }
        Err(e) => println!("\nNo package.json files found: {}", e),
    }

    // Update tauri.conf.json files
    match TauriConfigParser::update_version(project_path, &new_version, &options) {
        Ok(files) => {
            println!("\ntauri.conf.json files updated:");
            for file in files {
                println!("  - {}", file.display());
            }
        }
        Err(e) => println!("\nNo tauri.conf.json files found: {}", e),
    }

    println!("\nDone!");
    Ok(())
}
