use anyhow::Result;
use clap::Parser;
use log::LevelFilter;
use semver::Version;
use std::path::Path;
use update_version::{
    arguments::{Arguments, GitMode, SupportedTypes},
    git::GitTracker,
    parsers::{
        Parser as UpdateVersionParser, package_json_parser::PackageJsonParser,
        tauri_config_parser::TauriConfigParser, toml_parser::TomlParser,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();
    pretty_env_logger::env_logger::builder()
        .filter_level(if args.verbose { LevelFilter::Debug } else { LevelFilter::Info })
        .format_timestamp(None)
        .init();

    let version = args.new_version.map(|v| Version::parse(&v)).transpose()?;
    let path: &Path = args.path.as_ref();

    // Get or determine the version to use
    let final_version = match &version {
        Some(v) => v.clone(),
        None => {
            // Get current version from first available parser to determine what we'll increment to
            get_next_version(path, &args.supported_types)?
        }
    };

    match args.supported_types {
        SupportedTypes::All => {
            apply_version::<TomlParser>(path, version.as_ref())?;
            apply_version::<PackageJsonParser>(path, version.as_ref())?;
            apply_version::<TauriConfigParser>(path, version.as_ref())?;
        }
        SupportedTypes::TOML => {
            apply_version::<TomlParser>(path, version.as_ref())?
        }
        SupportedTypes::PackageJSON => {
            apply_version::<PackageJsonParser>(path, version.as_ref())?
        }
        SupportedTypes::TauriConfig => {
            apply_version::<TauriConfigParser>(path, version.as_ref())?
        }
    }

    // Handle git operations if mode is not None
    if args.git_mode != GitMode::None {
        let git = GitTracker::open(&args.path)?;
        git.execute_git_mode(args.git_mode, &final_version.to_string())?;
    }

    Ok(())
}

fn apply_version<P: UpdateVersionParser>(path: &Path, version: Option<&Version>) -> Result<()> {
    match version {
        Some(v) => {
            P::update_version(path, v)?;
        }
        None => {
            P::increment_version(path)?;
        }
    }
    Ok(())
}

/// Gets the next version by reading current version and incrementing patch
fn get_next_version(path: &Path, supported_types: &SupportedTypes) -> Result<Version> {
    // Try to get current version from available parsers
    let current = match supported_types {
        SupportedTypes::All | SupportedTypes::TOML => {
            TomlParser::get_current_version(path)
                .or_else(|_| PackageJsonParser::get_current_version(path))
                .or_else(|_| TauriConfigParser::get_current_version(path))
        }
        SupportedTypes::PackageJSON => PackageJsonParser::get_current_version(path),
        SupportedTypes::TauriConfig => TauriConfigParser::get_current_version(path),
    }?;

    // Increment patch version
    let mut next = current;
    next.patch += 1;
    Ok(next)
}
