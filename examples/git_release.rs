//! Example: Create a release with git commit and tag
//!
//! Run with: cargo run --example git_release

use anyhow::Result;
use semver::Version;
use update_version::{
    arguments::GitMode,
    git::GitTracker,
    parsers::{WalkOptions, toml_parser::TomlParser, Parser},
};

fn main() -> Result<()> {
    let new_version = Version::parse("1.0.0")?;

    // Update version in Cargo.toml
    println!("Updating version to {}...", new_version);
    let modified_files = TomlParser::update_version("./", &new_version, &WalkOptions::default())?;

    // Open the git repository
    let git = GitTracker::open("./", false)?;

    // Execute git operations: commit, tag, and push
    // Change to GitMode::Commit or GitMode::CommitTag if you don't want to push
    git.execute_git_mode(GitMode::CommitPushTag, &new_version.to_string(), &modified_files)?;

    println!("Release v{} created and pushed!", new_version);

    Ok(())
}
