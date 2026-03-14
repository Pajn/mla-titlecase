//! Command-line interface for `mla-titlecase`.

mod checksum;
mod cli;
mod cmd;
mod error;
mod fsutil;
mod manifest;
mod normalize;
mod plugin_build;
mod sources;

fn main() {
    if let Err(error) = cli::run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
