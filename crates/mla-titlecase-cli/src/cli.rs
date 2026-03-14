use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::{cmd, error::Result, sources::SourceId};

#[derive(Debug, Parser)]
#[command(name = "mla-titlecase", about = "Build and inspect MLA titlecase lexicon plugins")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Lexicon-related commands.
    Lexicon {
        /// Lexicon subcommands.
        #[command(subcommand)]
        command: LexiconCommand,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum LexiconCommand {
    /// List supported sources.
    ListSources,
    /// Show licensing details for a source.
    ShowLicense(ShowLicenseArgs),
    /// Fetch a raw source artifact.
    Fetch(FetchArgs),
    /// Prepare a raw source into normalized JSON.
    Prepare(PrepareArgs),
    /// Build a JSON or FST plugin from prepared data.
    BuildPlugin(BuildPluginArgs),
    /// Inspect a JSON or FST plugin.
    InspectPlugin(InspectPluginArgs),
    /// Diff two plugins.
    DiffPlugin(DiffPluginArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum OutputFormat {
    Json,
    Fst,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub(crate) enum PreparePayloadKind {
    WordSet,
    CanonicalMap,
    MultiwordMap,
    RankedWords,
    ProtectedSpellings,
}

#[derive(Debug, Args)]
pub(crate) struct ShowLicenseArgs {
    #[arg(value_enum)]
    pub(crate) source: SourceId,
}

#[derive(Debug, Args)]
pub(crate) struct FetchArgs {
    #[arg(value_enum)]
    pub(crate) source: SourceId,
    #[arg(long)]
    pub(crate) output: PathBuf,
    #[arg(long)]
    pub(crate) manifest: Option<PathBuf>,
    #[arg(long)]
    pub(crate) url: Option<String>,
    #[arg(long)]
    pub(crate) from_file: Option<PathBuf>,
    #[arg(long)]
    pub(crate) query: Option<String>,
    #[arg(long)]
    pub(crate) language: Option<String>,
    #[arg(long)]
    pub(crate) limit: Option<usize>,
    #[arg(long)]
    pub(crate) acknowledge_cc_by_sa: bool,
}

#[derive(Debug, Args)]
pub(crate) struct PrepareArgs {
    #[arg(value_enum)]
    pub(crate) source: SourceId,
    #[arg(long)]
    pub(crate) input: PathBuf,
    #[arg(long)]
    pub(crate) manifest: Option<PathBuf>,
    #[arg(long)]
    pub(crate) output: PathBuf,
    #[arg(long)]
    pub(crate) source_url: Option<String>,
    #[arg(long, value_enum)]
    pub(crate) payload_kind: Option<PreparePayloadKind>,
    #[arg(long)]
    pub(crate) acknowledge_cc_by_sa: bool,
}

#[derive(Debug, Args)]
pub(crate) struct BuildPluginArgs {
    #[arg()]
    pub(crate) input: PathBuf,
    #[arg(long)]
    pub(crate) output: PathBuf,
    #[arg(long, value_enum)]
    pub(crate) format: OutputFormat,
}

#[derive(Debug, Args)]
pub(crate) struct InspectPluginArgs {
    #[arg()]
    pub(crate) path: PathBuf,
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct DiffPluginArgs {
    #[arg()]
    pub(crate) left: PathBuf,
    #[arg()]
    pub(crate) right: PathBuf,
    #[arg(long)]
    pub(crate) json: bool,
}

pub(crate) fn run() -> Result<()> {
    cmd::run(Cli::parse())
}
