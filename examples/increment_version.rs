//! Example: Auto-increment patch version
//!
//! Run with: cargo run --example increment_version

use anyhow::Result;
use update_version::parsers::{WalkOptions, toml_parser::TomlParser, Parser};

fn main() -> Result<()> {
    let options = WalkOptions::default();

    // Get current version
    let current = TomlParser::get_current_version("./", &options)?;
    println!("Current version: {}", current);

    // Increment patch version (1.2.3 -> 1.2.4)
    let updated_files = TomlParser::increment_version("./", &options)?;

    // Get new version
    let new_version = TomlParser::get_current_version("./", &options)?;
    println!("New version: {}", new_version);

    println!("\nUpdated {} file(s):", updated_files.len());
    for file in &updated_files {
        println!("  - {}", file.display());
    }

    Ok(())
}
