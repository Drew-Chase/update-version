use anyhow::Result;
use log::{debug, info};
use semver::Version;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod package_json_parser;
pub mod tauri_config_parser;
pub mod toml_parser;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParsingError {
    #[error("No versions found in directory: {0}")]
    NoVersionFoundError(String),
}

/// Options controlling how directory walking behaves with respect to ignore files.
#[derive(Debug, Clone, Default)]
pub struct WalkOptions {
    /// When `true`, disables all ignore file processing (.gitignore, .uvignore, etc.).
    /// When `false` (default), ignore files are respected.
    pub no_ignore: bool,
}

/// Increments a semver version, preserving prerelease labels.
///
/// - `1.2.3` → `1.2.4` (no prerelease: bump patch)
/// - `1.0.0-alpha.0` → `1.0.0-alpha.1` (numeric prerelease suffix: bump it)
/// - `1.0.0-alpha` → `1.0.1-alpha` (non-numeric prerelease: bump patch, keep label)
pub fn increment_semver(version: &Version) -> Result<Version> {
    let mut next = version.clone();
    next.build = semver::BuildMetadata::EMPTY;

    if version.pre.is_empty() {
        next.patch += 1;
        return Ok(next);
    }

    // Split prerelease into dot-separated identifiers
    let pre_str = version.pre.as_str();
    let parts: Vec<&str> = pre_str.split('.').collect();

    // Check if the last identifier is numeric
    if let Some(last) = parts.last() {
        if let Ok(n) = last.parse::<u64>() {
            // Increment the numeric suffix: alpha.0 -> alpha.1
            let prefix = &parts[..parts.len() - 1];
            let new_pre = if prefix.is_empty() {
                format!("{}", n + 1)
            } else {
                format!("{}.{}", prefix.join("."), n + 1)
            };
            next.pre = semver::Prerelease::new(&new_pre)?;
            return Ok(next);
        }
    }

    // Non-numeric prerelease (e.g. "alpha"): bump patch, keep label
    next.patch += 1;
    Ok(next)
}

pub trait Parser {
    fn update_version(
        path: impl AsRef<Path>,
        version: &Version,
        options: &WalkOptions,
    ) -> Result<Vec<PathBuf>> {
        info!("Updating version to {}", version);
        let files = Self::get_matching_files(path, options)?;
        let version_regex = Self::version_match_regex()?;
        for file in &files {
            debug!("Checking file: '{}'", file.display());
            let contents = std::fs::read_to_string(file)?;
            let new_contents = version_regex
                .replace(contents.as_str(), Self::version_line_format(version)?)
                .to_string();
            std::fs::write(file, new_contents)?;
        }
        Ok(files)
    }
    fn increment_version(path: impl AsRef<Path>, options: &WalkOptions) -> Result<Vec<PathBuf>> {
        let path = path.as_ref();
        let current_version = Self::get_current_version(path, options)?;
        let new_version = increment_semver(&current_version)?;
        debug!(
            "Incrementing version from {} -> {}",
            current_version, new_version
        );
        Self::update_version(path, &new_version, options)
    }
    fn get_current_version(path: impl AsRef<Path>, options: &WalkOptions) -> Result<Version> {
        let path = path.as_ref();
        let files = Self::get_matching_files(path, options)?;
        let version_regex = Self::version_match_regex()?;

        for file in files {
            let contents = std::fs::read_to_string(file)?;
            if let Some(captures) = version_regex.captures(contents.as_str())
                && let Some(version) = captures.get(2)
            {
                let version = version.as_str();
                debug!("Found current version: {}", version);
                return Ok(Version::parse(version)?);
            }
        }

        Err(ParsingError::NoVersionFoundError(path.to_string_lossy().to_string()).into())
    }

    fn get_matching_files(path: impl AsRef<Path>, options: &WalkOptions) -> Result<Vec<PathBuf>> {
        debug!("Checking matching files");
        let mut files: Vec<PathBuf> = vec![];
        let path = path.as_ref();
        let filename_regex = Self::filename_match_regex()?;

        let mut builder = ignore::WalkBuilder::new(path);

        if options.no_ignore {
            // Disable ignore file processing but keep hidden file filtering
            // so .git/ and other hidden directories are still skipped
            builder.git_ignore(false);
            builder.git_global(false);
            builder.git_exclude(false);
        } else {
            builder.add_custom_ignore_filename(".uvignore");
        }

        for item in builder.build() {
            let item = item?;
            let path = item.path();
            if filename_regex.is_match(path.to_string_lossy().as_ref()) {
                files.push(path.to_path_buf());
            }
        }

        // Sort by path depth (shallowest first) then lexicographically for deterministic ordering
        files.sort_by(|a, b| {
            a.components().count().cmp(&b.components().count())
                .then_with(|| a.cmp(b))
        });

        debug!("Found files: {:?}", files);
        Ok(files)
    }

    fn version_match_regex() -> Result<regex::Regex>;
    fn filename_match_regex() -> Result<regex::Regex>;
    fn version_line_format(version: &Version) -> Result<String>;
}
