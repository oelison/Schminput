use std::{fs, path::PathBuf};

use bevy::prelude::*;

use crate::persistent_bindings::{
    DeserializeSchminputConfig, FinnishedSchminputConfigSerialization, PersistentBindingsSet,
    SerializeSchminputConfig,
};

pub struct SchminputConfigPlugin;

#[derive(Resource, Clone, Debug)]
pub enum ConfigFilePath {
    Config {
        app_name: &'static str,
        file_name: &'static str,
    },
    Path(PathBuf),
}
impl ConfigFilePath {
    pub fn path_buf(&self) -> Option<PathBuf> {
        let path = match self {
            ConfigFilePath::Config {
                app_name,
                file_name,
            } => {
                let mut config_dir = dirs::config_dir()?;
                config_dir.push(app_name);
                config_dir.push(file_name);
                config_dir
            }
            ConfigFilePath::Path(p) => p.clone(),
        };
        Some(path)
    }
}

#[derive(Message, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct LoadSchminputConfig;
#[derive(Message, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct SaveSchminputConfig;
#[derive(Message, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct FinnishedSavingSchminputConfig;

impl Plugin for SchminputConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LoadSchminputConfig>();
        app.add_message::<SaveSchminputConfig>();
        app.add_message::<FinnishedSavingSchminputConfig>();
        app.add_systems(
            PostUpdate,
            load_config
                .run_if(on_message::<LoadSchminputConfig>)
                .before(PersistentBindingsSet::Deserialize),
        );
        app.add_systems(
            PostUpdate,
            request_save_config
                .run_if(on_message::<SaveSchminputConfig>)
                .before(PersistentBindingsSet::Serialize),
        );
        app.add_systems(
            PostUpdate,
            save_config
                .run_if(on_message::<FinnishedSchminputConfigSerialization>)
                .after(PersistentBindingsSet::Serialize),
        );
    }
}

fn request_save_config(
    config_path: Res<ConfigFilePath>,
    mut request_serialize: MessageWriter<SerializeSchminputConfig>,
) {
    let Some(path) = config_path.path_buf() else {
        error!("unable to get config path");
        return;
    };
    let text = 'load_string: {
        if !path.is_file() {
            error!("{} is not a file", path.to_string_lossy());
            break 'load_string "".to_string();
        }

        match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(err) => {
                error!("unable to read text from {}: {err}", path.to_string_lossy());
                "".to_string()
            }
        }
    };
    request_serialize.write(SerializeSchminputConfig { base_config: text });
}
fn save_config(
    config_path: Res<ConfigFilePath>,
    mut serialized: MessageReader<FinnishedSchminputConfigSerialization>,
    mut finnish_signal: MessageWriter<FinnishedSavingSchminputConfig>,
) {
    let Some(path) = config_path.path_buf() else {
        error!("unable to get config path");
        return;
    };

    if let Some(dir) = path.parent() {
        if let Err(err) = fs::create_dir_all(dir) {
            error!("unable to create parrent dirs for config file: {err}");
        }
    }

    for output in serialized.read() {
        if let Err(err) = fs::write(&path, &output.output) {
            error!("unable to write config file: {err}");
        }
        finnish_signal.write_default();
    }
}
fn load_config(
    config_path: Res<ConfigFilePath>,
    mut request_deserialize: MessageWriter<DeserializeSchminputConfig>,
) {
    let Some(path) = config_path.path_buf() else {
        error!("unable to get config path");
        return;
    };
    if !path.is_file() {
        error!("{} is not a file", path.to_string_lossy());
        return;
    }
    let contents = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(err) => {
            error!("unable to read text from {}: {err}", path.to_string_lossy());
            return;
        }
    };
    request_deserialize.write(DeserializeSchminputConfig { config: contents });

    // if let Some(dir) = path.parent() {
    //     fs::create_dir_all(dir);
    // }
}
