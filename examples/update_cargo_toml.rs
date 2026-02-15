//! Example: Update version in Cargo.toml files
//!
//! Run with: cargo run --example update_cargo_toml

use anyhow::Result;
use semver::Version;
use update_version::parsers::{WalkOptions, toml_parser::TomlParser, Parser};

fn main() -> Result<()> {
    // Set a specific version
    let new_version = Version::parse("2.0.0")?;
    let updated_files = TomlParser::update_version("./", &new_version, &WalkOptions::default())?;

    println!("Updated {} file(s):", updated_files.len());
    for file in &updated_files {
        println!("  - {}", file.display());
    }

    Ok(())
}
