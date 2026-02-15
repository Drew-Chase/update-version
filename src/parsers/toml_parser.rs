use crate::parsers::Parser;
use regex::Regex;
use semver::Version;

pub struct TomlParser;
impl Parser for TomlParser {
    fn version_match_regex() -> anyhow::Result<Regex> {
        Ok(Regex::new(r##"(?m)^(version\s*=\s*")(\d+\.\d+\.\d+[^"]*)""##)?)
    }

    fn filename_match_regex() -> anyhow::Result<Regex> {
        Ok(Regex::new(r#"(?i)[/\\]Cargo\.toml$"#)?)
    }

    fn version_line_format(version: &Version) -> anyhow::Result<String> {
        Ok(format!("${{1}}{version}\""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_regex_matches_simple() {
        let regex = TomlParser::version_match_regex().unwrap();
        let content = r#"version = "1.2.3""#;
        let captures = regex.captures(content).unwrap();
        assert_eq!(captures.get(2).unwrap().as_str(), "1.2.3");
    }

    #[test]
    fn test_version_regex_matches_no_spaces() {
        let regex = TomlParser::version_match_regex().unwrap();
        let content = r#"version="0.1.0""#;
        let captures = regex.captures(content).unwrap();
        assert_eq!(captures.get(2).unwrap().as_str(), "0.1.0");
    }

    #[test]
    fn test_version_regex_matches_in_file() {
        let regex = TomlParser::version_match_regex().unwrap();
        let content = r#"[package]
name = "my-crate"
version = "2.0.0-beta"
edition = "2021"
"#;
        let captures = regex.captures(content).unwrap();
        assert_eq!(captures.get(2).unwrap().as_str(), "2.0.0-beta");
    }

    #[test]
    fn test_version_regex_ignores_dependency_versions() {
        let regex = TomlParser::version_match_regex().unwrap();
        let content = r#"[package]
name = "test"
version = "1.0.0"

[dependencies]
serde = { version = "1.0" }
"#;
        let captures = regex.captures(content).unwrap();
        // Should match package version, not dependency version
        assert_eq!(captures.get(2).unwrap().as_str(), "1.0.0");
    }

    #[test]
    fn test_filename_regex_matches_cargo_toml() {
        let regex = TomlParser::filename_match_regex().unwrap();
        assert!(regex.is_match("/path/to/Cargo.toml"));
        assert!(regex.is_match("\\path\\to\\Cargo.toml"));
        assert!(regex.is_match("/Cargo.toml"));
    }

    #[test]
    fn test_filename_regex_case_insensitive() {
        let regex = TomlParser::filename_match_regex().unwrap();
        assert!(regex.is_match("/path/to/CARGO.TOML"));
        assert!(regex.is_match("/path/to/cargo.toml"));
    }

    #[test]
    fn test_filename_regex_no_false_positives() {
        let regex = TomlParser::filename_match_regex().unwrap();
        assert!(!regex.is_match("/path/to/pyproject.toml"));
        assert!(!regex.is_match("/path/to/Cargo.toml.bak"));
        assert!(!regex.is_match("/path/to/NotCargo.toml"));
    }

    #[test]
    fn test_version_line_format() {
        let version = Version::parse("1.2.3").unwrap();
        let formatted = TomlParser::version_line_format(&version).unwrap();
        assert_eq!(formatted, "${1}1.2.3\"");
    }

    #[test]
    fn test_version_line_format_with_prerelease() {
        let version = Version::parse("1.0.0-alpha.1").unwrap();
        let formatted = TomlParser::version_line_format(&version).unwrap();
        assert_eq!(formatted, "${1}1.0.0-alpha.1\"");
    }
}
