use crate::parsers::Parser;
use crate::parsers::package_json_parser::PackageJsonParser;
use regex::Regex;
use semver::Version;

pub struct TauriConfigParser;

impl Parser for TauriConfigParser {
    fn version_match_regex() -> anyhow::Result<Regex> {
        PackageJsonParser::version_match_regex()
    }

    fn filename_match_regex() -> anyhow::Result<Regex> {
        Ok(Regex::new(r#"(?i)[/\\]tauri\.conf\.json$"#)?)
    }

    fn version_line_format(version: &Version) -> anyhow::Result<String> {
        Ok(format!(
            r#""version": "{}.{}.{}""#,
            version.major, version.minor, version.patch
        ))
    }
}
