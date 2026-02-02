use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "notenest", version, about = "Clinical Note Prep Toolkit")]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Parser)]
#[command(name = "notenest", version, about = "Clinical Note Prep Toolkit")]
pub struct LegacyArgs {
    #[arg(value_name = "INPUT", default_value = "-")]
    pub input: String,

    #[arg(
        long,
        default_value = "protected",
        value_parser = ["protected", "masked", "hidden", "removed", "angle"],
        alias = "style"
    )]
    pub placeholder_style: String,

    #[arg(long, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum Command {
    Convert(ConvertArgs),
    Batch(BatchArgs),
    Cloak(CloakArgs),
    SelfCheck(SelfCheckArgs),
}

#[derive(Args)]
pub struct ConvertArgs {
    #[arg(value_name = "INPUT")]
    pub input: String,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub patient: bool,

    #[arg(long, value_name = "STYLE")]
    pub clinician: Option<String>,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub redact: bool,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub all: bool,

    #[arg(long, default_value = "text")]
    pub format: OutputFormat,

    #[arg(long)]
    pub out: Option<String>,

    #[arg(
        long,
        default_value = "protected",
        value_parser = ["protected", "masked", "hidden", "removed", "angle"],
        alias = "style"
    )]
    pub placeholder_style: String,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub offline: bool,
}

#[derive(Args)]
pub struct BatchArgs {
    pub input_dir: String,

    #[arg(long, default_value = "*.txt")]
    pub glob: String,

    #[arg(long)]
    pub out: Option<String>,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub all: bool,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub offline: bool,
}

#[derive(Args)]
pub struct CloakArgs {
    #[command(subcommand)]
    pub command: CloakCommand,
}

#[derive(Subcommand)]
pub enum CloakCommand {
    Scan(CloakScanArgs),
    Protect(CloakProtectArgs),
    Config(CloakConfigArgs),
    Report(CloakReportArgs),
    GitHook(CloakGitHookArgs),
}

#[derive(Args)]
pub struct CloakScanArgs {
    pub path: String,

    #[arg(long)]
    pub config: Option<String>,

    #[arg(long)]
    pub csv: Option<String>,

    #[arg(long)]
    pub col: Vec<String>,
}

#[derive(Args)]
pub struct CloakProtectArgs {
    pub path: String,

    #[arg(short = 'o', long)]
    pub out: String,

    #[arg(long)]
    pub config: Option<String>,

    #[arg(long)]
    pub mapping: Option<String>,

    #[arg(long, value_name = "ENV")]
    pub mapping_pass: Option<String>,

    #[arg(long, default_value = "text")]
    pub format: CloakOutputFormat,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub emit_structured: bool,
}

#[derive(Args)]
pub struct CloakConfigArgs {
    #[command(subcommand)]
    pub command: CloakConfigCommand,
}

#[derive(Subcommand)]
pub enum CloakConfigCommand {
    Init {
        #[arg(short = 'o', long)]
        out: Option<String>,
    },
    Validate {
        #[arg(long)]
        path: Option<String>,
    },
}

#[derive(Args)]
pub struct CloakReportArgs {
    pub run: String,

    #[arg(long)]
    pub out: Option<String>,

    #[arg(long, default_value = "md")]
    pub format: ReportFormat,
}

#[derive(Args)]
pub struct CloakGitHookArgs {
    #[command(subcommand)]
    pub command: CloakGitHookCommand,
}

#[derive(Args)]
pub struct SelfCheckArgs {}

#[derive(Subcommand)]
pub enum CloakGitHookCommand {
    Install {
        #[arg(long, default_value = "pre-commit")]
        mode: String,
        #[arg(long)]
        config: Option<String>,
        #[arg(long)]
        paths: Vec<String>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Both,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CloakOutputFormat {
    Text,
    Json,
    Both,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ReportFormat {
    Csv,
    Json,
    Md,
}
