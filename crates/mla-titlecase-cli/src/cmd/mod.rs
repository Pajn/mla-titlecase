pub(crate) mod build_plugin;
pub(crate) mod diff_plugin;
pub(crate) mod fetch;
pub(crate) mod inspect_plugin;
pub(crate) mod list_sources;
pub(crate) mod prepare;
pub(crate) mod show_license;

use crate::{
    cli::{Cli, Command, LexiconCommand},
    error::Result,
};

pub(crate) fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Lexicon { command } => match command {
            LexiconCommand::ListSources => list_sources::run(),
            LexiconCommand::ShowLicense(args) => show_license::run(args),
            LexiconCommand::Fetch(args) => fetch::run(args),
            LexiconCommand::Prepare(args) => prepare::run(args),
            LexiconCommand::BuildPlugin(args) => build_plugin::run(args),
            LexiconCommand::InspectPlugin(args) => inspect_plugin::run(args),
            LexiconCommand::DiffPlugin(args) => diff_plugin::run(args),
        },
    }
}
