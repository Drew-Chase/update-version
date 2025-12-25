use crate::parsers::Parser;
use regex::Regex;
use semver::Version;

pub struct TomlParser;
impl Parser for TomlParser {
    fn version_match_regex() -> anyhow::Result<Regex> {
        Ok(Regex::new(r#"(?m)^version\s*=\s*"([^"]*)""#)?)
    }

    fn filename_match_regex() -> anyhow::Result<Regex> {
        Ok(Regex::new(r#"(?i)[/\\]Cargo\.toml$"#)?)
    }

    fn version_line_format(version: &Version) -> anyhow::Result<String> {
        Ok(format!(r#"version="{}""#, version))
    }
}
