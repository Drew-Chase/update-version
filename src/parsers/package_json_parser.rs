use regex::Regex;
use semver::Version;
use crate::parsers::Parser;

pub struct PackageJsonParser;

impl Parser for PackageJsonParser {
	fn version_match_regex() -> anyhow::Result<Regex> {
    Ok(Regex::new(r#"(?m)^\s*"version"\s*:\s*"([^"]*)""#)?)
}

	fn filename_match_regex() -> anyhow::Result<Regex> {
		Ok(Regex::new(r#"(?i)[/\\]package\.json$"#)?)
	}

	fn version_line_format(version: &Version) -> anyhow::Result<String> {
		Ok(format!(r#""version": "{}""#, version))
	}
}