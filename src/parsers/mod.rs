use anyhow::Result;
use log::{debug, info};
use semver::Version;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod package_json_parser;
pub mod tauri_config_parser;
pub mod toml_parser;

#[derive(Debug, Error)]
enum ParsingError {
    #[error("No versions found in directory: {0}")]
    NoVersionFoundError(String),
}

pub trait Parser {
    fn update_version(path: impl AsRef<Path>, version: &Version) -> Result<Vec<PathBuf>> {
        info!("Updating version to {}", version);
        let files = Self::get_matching_files(path)?;
        let version_regex = Self::version_match_regex()?;
        for file in &files {
            debug!("Checking file: '{}'", file.display());
            let contents = std::fs::read_to_string(&file)?;
            let new_contents = version_regex
                .replace(contents.as_str(), Self::version_line_format(version)?)
                .to_string();
            std::fs::write(&file, new_contents)?;
        }
        Ok(files)
    }
    fn increment_version(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        let path = path.as_ref();
        let current_version = Self::get_current_version(&path)?;
        let mut new_version = current_version.clone();
        new_version.patch += 1;
        debug!(
            "Incrementing version from {} -> {}",
            current_version, new_version
        );
        Self::update_version(path, &new_version)
    }
    fn get_current_version(path: impl AsRef<Path>) -> Result<Version> {
        let path = path.as_ref();
        let files = Self::get_matching_files(path)?;
        let version_regex = Self::version_match_regex()?;

        for file in files {
            let contents = std::fs::read_to_string(file)?;
            if let Some(captures) = version_regex.captures(contents.as_str()) {
                if let Some(version) = captures.get(1) {
                    let version = version.as_str();
                    debug!("Found current version: {}", version);
                    return Ok(Version::parse(version)?);
                }
            }
        }

        Err(ParsingError::NoVersionFoundError(path.to_string_lossy().to_string()).into())
    }

    fn get_matching_files(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        debug!("Checking matching files");
        let mut files: Vec<PathBuf> = vec![];
        let path = path.as_ref();
        let walkdir_iter = walkdir::WalkDir::new(path);
        let filename_regex = Self::filename_match_regex()?;

        for item in walkdir_iter {
            let item = item?;
            let path = item.path();
            if filename_regex.is_match(path.to_string_lossy().as_ref()) {
                files.push(path.to_path_buf());
            }
        }

        debug!("Found files: {:?}", files);
        Ok(files)
    }

    fn version_match_regex() -> Result<regex::Regex>;
    fn filename_match_regex() -> Result<regex::Regex>;
    fn version_line_format(version: &Version) -> Result<String>;
}
