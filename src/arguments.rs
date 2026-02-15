use clap::{Parser, ValueEnum};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default)]
pub enum SupportedTypes {
    #[default]
    All,
    TOML,
    PackageJSON,
    TauriConfig,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default)]
pub enum GitMode {
    #[default]
    None,
    Commit,
    CommitPush,
    CommitPushTag,
    CommitTag,
}

#[derive(Debug, Parser)]
#[command(author, version, about, bin_name = "uv")]
pub struct Arguments {
    #[arg(long="types", short='t', value_enum, ignore_case = true, default_value_t = SupportedTypes::All)]
    pub supported_types: SupportedTypes,
    #[arg(long, short, value_enum, ignore_case = true, default_value_t = GitMode::None)]
    pub git_mode: GitMode,
    #[arg(long, short, default_value = "./")]
    pub path: String,
    #[arg(long, short)]
    pub verbose: bool,
    /// Disable .gitignore and .uvignore file processing during file discovery
    #[arg(long)]
    pub no_ignore: bool,
    pub new_version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let args = Arguments::parse_from(["uv"]);
        assert_eq!(args.supported_types, SupportedTypes::All);
        assert_eq!(args.git_mode, GitMode::None);
        assert_eq!(args.path, "./");
        assert!(!args.verbose);
        assert!(args.new_version.is_none());
    }

    #[test]
    fn test_parse_version() {
        let args = Arguments::parse_from(["uv", "1.2.3"]);
        assert_eq!(args.new_version, Some("1.2.3".to_string()));
    }

    #[test]
    fn test_parse_types_toml() {
        let args = Arguments::parse_from(["uv", "-t", "toml"]);
        assert_eq!(args.supported_types, SupportedTypes::TOML);
    }

    #[test]
    fn test_parse_types_case_insensitive() {
        let args = Arguments::parse_from(["uv", "-t", "TOML"]);
        assert_eq!(args.supported_types, SupportedTypes::TOML);

        let args = Arguments::parse_from(["uv", "-t", "PACKAGE-JSON"]);
        assert_eq!(args.supported_types, SupportedTypes::PackageJSON);

        let args = Arguments::parse_from(["uv", "-t", "tauri-config"]);
        assert_eq!(args.supported_types, SupportedTypes::TauriConfig);
    }

    #[test]
    fn test_parse_git_mode_commit() {
        let args = Arguments::parse_from(["uv", "-g", "commit"]);
        assert_eq!(args.git_mode, GitMode::Commit);
    }

    #[test]
    fn test_parse_git_mode_commit_push() {
        let args = Arguments::parse_from(["uv", "-g", "commit-push"]);
        assert_eq!(args.git_mode, GitMode::CommitPush);
    }

    #[test]
    fn test_parse_git_mode_commit_push_tag() {
        let args = Arguments::parse_from(["uv", "-g", "commit-push-tag"]);
        assert_eq!(args.git_mode, GitMode::CommitPushTag);
    }

    #[test]
    fn test_parse_git_mode_commit_tag() {
        let args = Arguments::parse_from(["uv", "-g", "commit-tag"]);
        assert_eq!(args.git_mode, GitMode::CommitTag);
    }

    #[test]
    fn test_parse_path() {
        let args = Arguments::parse_from(["uv", "-p", "/some/path"]);
        assert_eq!(args.path, "/some/path");
    }

    #[test]
    fn test_parse_verbose() {
        let args = Arguments::parse_from(["uv", "-v"]);
        assert!(args.verbose);
    }

    #[test]
    fn test_parse_long_flags() {
        let args = Arguments::parse_from([
            "uv",
            "--types",
            "toml",
            "--git-mode",
            "commit",
            "--path",
            "/test",
            "--verbose",
            "2.0.0",
        ]);
        assert_eq!(args.supported_types, SupportedTypes::TOML);
        assert_eq!(args.git_mode, GitMode::Commit);
        assert_eq!(args.path, "/test");
        assert!(args.verbose);
        assert_eq!(args.new_version, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_parse_combined_short_flags() {
        let args = Arguments::parse_from(["uv", "-t", "toml", "-g", "commit-push-tag", "1.0.0"]);
        assert_eq!(args.supported_types, SupportedTypes::TOML);
        assert_eq!(args.git_mode, GitMode::CommitPushTag);
        assert_eq!(args.new_version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_git_mode_equality() {
        assert_eq!(GitMode::None, GitMode::None);
        assert_ne!(GitMode::None, GitMode::Commit);
        assert_ne!(GitMode::Commit, GitMode::CommitPush);
    }

    #[test]
    fn test_supported_types_equality() {
        assert_eq!(SupportedTypes::All, SupportedTypes::All);
        assert_ne!(SupportedTypes::All, SupportedTypes::TOML);
    }
}
