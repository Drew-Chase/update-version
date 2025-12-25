use crate::parsers::package_json_parser::PackageJsonParser;
use crate::parsers::Parser;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_regex_matches_in_tauri_config() {
        let regex = TauriConfigParser::version_match_regex().unwrap();
        let content = r#"{
  "productName": "My App",
  "version": "1.0.0",
  "identifier": "com.example.app"
}"#;
        let captures = regex.captures(content).unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "1.0.0");
    }

    #[test]
    fn test_filename_regex_matches_tauri_conf_json() {
        let regex = TauriConfigParser::filename_match_regex().unwrap();
        assert!(regex.is_match("/path/to/tauri.conf.json"));
        assert!(regex.is_match("\\path\\to\\tauri.conf.json"));
        assert!(regex.is_match("/src-tauri/tauri.conf.json"));
    }

    #[test]
    fn test_filename_regex_case_insensitive() {
        let regex = TauriConfigParser::filename_match_regex().unwrap();
        assert!(regex.is_match("/path/to/TAURI.CONF.JSON"));
        assert!(regex.is_match("/path/to/Tauri.Conf.Json"));
    }

    #[test]
    fn test_filename_regex_no_false_positives() {
        let regex = TauriConfigParser::filename_match_regex().unwrap();
        assert!(!regex.is_match("/path/to/tauri.conf.json.bak"));
        assert!(!regex.is_match("/path/to/my-tauri.conf.json"));
        assert!(!regex.is_match("/path/to/tauri.json"));
    }

    #[test]
    fn test_version_line_format_strips_prerelease() {
        // Tauri config only uses major.minor.patch
        let version = Version::parse("1.2.3-beta.1").unwrap();
        let formatted = TauriConfigParser::version_line_format(&version).unwrap();
        assert_eq!(formatted, r#""version": "1.2.3""#);
    }

    #[test]
    fn test_version_line_format_simple() {
        let version = Version::parse("2.0.0").unwrap();
        let formatted = TauriConfigParser::version_line_format(&version).unwrap();
        assert_eq!(formatted, r#""version": "2.0.0""#);
    }
}
