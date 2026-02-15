use crate::parsers::Parser;
use regex::Regex;
use semver::Version;

pub struct PackageJsonParser;

impl Parser for PackageJsonParser {
    fn version_match_regex() -> anyhow::Result<Regex> {
        Ok(Regex::new(r##"(?m)^(\s*"version"\s*:\s*")([^"]*)""##)?)
    }

    fn filename_match_regex() -> anyhow::Result<Regex> {
        Ok(Regex::new(r#"(?i)[/\\]package\.json$"#)?)
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
        let regex = PackageJsonParser::version_match_regex().unwrap();
        let content = r#""version": "1.2.3""#;
        let captures = regex.captures(content).unwrap();
        assert_eq!(captures.get(2).unwrap().as_str(), "1.2.3");
    }

    #[test]
    fn test_version_regex_matches_with_indentation() {
        let regex = PackageJsonParser::version_match_regex().unwrap();
        let content = r#"  "version": "0.1.0""#;
        let captures = regex.captures(content).unwrap();
        assert_eq!(captures.get(2).unwrap().as_str(), "0.1.0");
    }

    #[test]
    fn test_version_regex_matches_in_file() {
        let regex = PackageJsonParser::version_match_regex().unwrap();
        let content = r#"{
  "name": "my-package",
  "version": "2.0.0",
  "description": "A test package"
}"#;
        let captures = regex.captures(content).unwrap();
        assert_eq!(captures.get(2).unwrap().as_str(), "2.0.0");
    }

    #[test]
    fn test_version_regex_various_spacing() {
        let regex = PackageJsonParser::version_match_regex().unwrap();

        let content1 = r#""version":"1.0.0""#;
        let captures1 = regex.captures(content1).unwrap();
        assert_eq!(captures1.get(2).unwrap().as_str(), "1.0.0");

        let content2 = r#""version" : "2.0.0""#;
        let captures2 = regex.captures(content2).unwrap();
        assert_eq!(captures2.get(2).unwrap().as_str(), "2.0.0");
    }

    #[test]
    fn test_filename_regex_matches_package_json() {
        let regex = PackageJsonParser::filename_match_regex().unwrap();
        assert!(regex.is_match("/path/to/package.json"));
        assert!(regex.is_match("\\path\\to\\package.json"));
        assert!(regex.is_match("/package.json"));
    }

    #[test]
    fn test_filename_regex_case_insensitive() {
        let regex = PackageJsonParser::filename_match_regex().unwrap();
        assert!(regex.is_match("/path/to/PACKAGE.JSON"));
        assert!(regex.is_match("/path/to/Package.Json"));
    }

    #[test]
    fn test_filename_regex_no_false_positives() {
        let regex = PackageJsonParser::filename_match_regex().unwrap();
        assert!(!regex.is_match("/path/to/package-lock.json"));
        assert!(!regex.is_match("/path/to/package.json.bak"));
        assert!(!regex.is_match("/path/to/my-package.json"));
    }

    #[test]
    fn test_version_line_format() {
        let version = Version::parse("1.2.3").unwrap();
        let formatted = PackageJsonParser::version_line_format(&version).unwrap();
        assert_eq!(formatted, "${1}1.2.3\"");
    }

    #[test]
    fn test_version_line_format_with_prerelease() {
        let version = Version::parse("1.0.0-beta.2").unwrap();
        let formatted = PackageJsonParser::version_line_format(&version).unwrap();
        assert_eq!(formatted, "${1}1.0.0-beta.2\"");
    }
}