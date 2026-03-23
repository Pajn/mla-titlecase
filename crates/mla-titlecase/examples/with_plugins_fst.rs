#![allow(missing_docs)]

use std::{error::Error, fs};

use mla_titlecase::{
    fst_store, titlecase_with_options, ExternalLexicons, LexiconPlugin, MapEntry, PluginMetadata,
    PluginPayload, TitleCaseOptions,
};

fn main() -> Result<(), Box<dyn Error>> {
    let plugin = LexiconPlugin {
        metadata: PluginMetadata::new("example", "MIT"),
        payload: PluginPayload::CanonicalMap {
            entries: vec![MapEntry { key: "postgres".to_string(), value: "Postgres".to_string() }],
        },
    };

    let path = std::env::temp_dir()
        .join(format!("mla-titlecase-fst-example-{}.mlatl", std::process::id()));
    fst_store::save_fst_plugin(&path, &plugin)?;

    let mut external = ExternalLexicons::default();
    external.register_mmap_fst_plugin(&path)?;
    let options = TitleCaseOptions::with_external_lexicons(&external);

    println!("{}", titlecase_with_options("postgres for rust developers", &options));

    let _ = fs::remove_file(path);
    Ok(())
}
