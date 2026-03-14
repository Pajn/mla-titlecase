//! JSON plugin serialization helpers.

use std::{fs, path::Path};

use crate::{error::Result, plugin::LexiconPlugin};

/// Loads a plugin from JSON.
pub fn load_json_plugin(path: impl AsRef<Path>) -> Result<LexiconPlugin> {
    let plugin = serde_json::from_slice::<LexiconPlugin>(&fs::read(path)?)?;
    plugin.validate()?;
    Ok(plugin)
}

/// Saves a plugin to pretty-printed JSON.
pub fn save_json_plugin(path: impl AsRef<Path>, plugin: &LexiconPlugin) -> Result<()> {
    plugin.validate()?;
    let bytes = serde_json::to_vec_pretty(plugin)?;
    fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{
        json_store::{load_json_plugin, save_json_plugin},
        plugin::{LexiconPlugin, MapEntry, PluginMetadata, PluginPayload},
    };

    #[test]
    fn round_trips_json_plugins() {
        let plugin = LexiconPlugin {
            metadata: PluginMetadata::new("fixture", "MIT"),
            payload: PluginPayload::WordSet {
                words: vec!["alpha".to_string(), "beta".to_string()],
            },
        };
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("plugin.json");

        save_json_plugin(&path, &plugin).unwrap();
        let loaded = load_json_plugin(&path).unwrap();

        assert_eq!(loaded, plugin);
    }

    #[test]
    fn round_trips_multiword_json_plugins() {
        let plugin = LexiconPlugin {
            metadata: PluginMetadata::new("fixture", "MIT"),
            payload: PluginPayload::MultiwordMap {
                entries: vec![MapEntry {
                    key: "new york city".to_string(),
                    value: "New York City".to_string(),
                }],
            },
        };
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("plugin.json");

        save_json_plugin(&path, &plugin).unwrap();
        let loaded = load_json_plugin(&path).unwrap();

        assert_eq!(loaded, plugin);
    }
}
