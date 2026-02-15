use anyhow::Result;
use clap::Parser;
use log::LevelFilter;
use semver::Version;
use std::path::Path;
use update_version::{
    arguments::{Arguments, GitMode, SupportedTypes},
    git::GitTracker,
    parsers::{
        Parser as UpdateVersionParser, WalkOptions, package_json_parser::PackageJsonParser,
        tauri_config_parser::TauriConfigParser, toml_parser::TomlParser,
    },
};

fn main() -> Result<()> {
    let args = Arguments::parse();
    pretty_env_logger::env_logger::builder()
        .filter_level(if args.verbose { LevelFilter::Debug } else { LevelFilter::Info })
        .format_timestamp(None)
        .init();

    let walk_options = WalkOptions { no_ignore: args.no_ignore };
    let version = args.new_version.map(|v| Version::parse(&v)).transpose()?;
    let path: &Path = args.path.as_ref();

    // Get or determine the version to use
    let final_version = match &version {
        Some(v) => v.clone(),
        None => {
            // Get current version from first available parser to determine what we'll increment to
            get_next_version(path, &args.supported_types, &walk_options)?
        }
    };

    let mut modified_files = Vec::new();

    match args.supported_types {
        SupportedTypes::All => {
            modified_files.extend(apply_version::<TomlParser>(path, &final_version, &walk_options)?);
            modified_files.extend(apply_version::<PackageJsonParser>(path, &final_version, &walk_options)?);
            modified_files.extend(apply_version::<TauriConfigParser>(path, &final_version, &walk_options)?);
        }
        SupportedTypes::TOML => {
            modified_files.extend(apply_version::<TomlParser>(path, &final_version, &walk_options)?);
        }
        SupportedTypes::PackageJSON => {
            modified_files.extend(apply_version::<PackageJsonParser>(path, &final_version, &walk_options)?);
        }
        SupportedTypes::TauriConfig => {
            modified_files.extend(apply_version::<TauriConfigParser>(path, &final_version, &walk_options)?);
        }
    }

    // Handle git operations if mode is not None
    if args.git_mode != GitMode::None {
        let git = GitTracker::open(&args.path, args.allow_insecure)?;
        git.execute_git_mode(args.git_mode, &final_version.to_string(), &modified_files)?;
    }

    Ok(())
}

fn apply_version<P: UpdateVersionParser>(
    path: &Path,
    version: &Version,
    options: &WalkOptions,
) -> Result<Vec<std::path::PathBuf>> {
    Ok(P::update_version(path, version, options)?)
}

/// Gets the next version by reading current version and incrementing patch
fn get_next_version(
    path: &Path,
    supported_types: &SupportedTypes,
    options: &WalkOptions,
) -> Result<Version> {
    // Try to get current version from available parsers
    let current = match supported_types {
        SupportedTypes::All | SupportedTypes::TOML => {
            TomlParser::get_current_version(path, options)
                .or_else(|_| PackageJsonParser::get_current_version(path, options))
                .or_else(|_| TauriConfigParser::get_current_version(path, options))
        }
        SupportedTypes::PackageJSON => PackageJsonParser::get_current_version(path, options),
        SupportedTypes::TauriConfig => TauriConfigParser::get_current_version(path, options),
    }?;

    let mut next = current;
    if next.pre.is_empty() {
        next.patch += 1;
    } else {
        next.pre = semver::Prerelease::EMPTY;
    }
    next.build = semver::BuildMetadata::EMPTY;
    Ok(next)
}
