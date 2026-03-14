#![allow(missing_docs)]

use std::{error::Error, fs};

use mla_titlecase::{
    json_store, titlecase_with_options, ExternalLexicons, LexiconPlugin, MapEntry, PluginMetadata,
    PluginPayload, TitleCaseOptions,
};

fn main() -> Result<(), Box<dyn Error>> {
    let plugin = LexiconPlugin {
        metadata: PluginMetadata::new("example", "MIT"),
        payload: PluginPayload::ProtectedSpellings {
            entries: vec![MapEntry { key: "github".to_string(), value: "GitHub".to_string() }],
        },
    };

    let path = std::env::temp_dir()
        .join(format!("mla-titlecase-json-example-{}.json", std::process::id()));
    json_store::save_json_plugin(&path, &plugin)?;
    let loaded = json_store::load_json_plugin(&path)?;

    let mut external = ExternalLexicons::default();
    loaded.register_into(&mut external)?;
    let options = TitleCaseOptions::with_external_lexicons(&external);

    println!("{}", titlecase_with_options("github in practice", &options));

    let _ = fs::remove_file(path);
    Ok(())
}
