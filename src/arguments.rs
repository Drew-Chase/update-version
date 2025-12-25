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
    pub new_version: Option<String>
}
