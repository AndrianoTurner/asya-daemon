use derive_more::derive::{Deref, From, Into};
use serde::Serialize;
use std::{
    collections::HashMap,
    ffi::{CStr, CString, OsStr},
    fs, io,
    path::{Path, PathBuf},
    ptr::{self},
    thread,
};
use tokio::sync::{mpsc::Receiver, Mutex};
use tracing::*;

use shared::{
    configuration::{self, ConfigFieldType, CONFIG},
    event_system,
};

mod abstractions;
mod api_callbacks;

mod dotnet;
mod native;

#[derive(Debug, Serialize, Clone, Deref, From, Into)]
#[serde(into = "String")]
pub struct Sender(pub String);

#[derive(Debug, Serialize, Clone, Deref, From, Into)]
#[serde(into = "String")]
pub struct Event(pub String);

/// Event publishing from plugins.
#[derive(Debug, Serialize)]
pub struct PluginEvent {
    sender: Sender,
    data: Event,
}

#[derive(Debug, Clone)]
pub enum FoundedPlugin {
    Native {
        path: PathBuf,
    },
    Dotnet {
        dll_path: PathBuf,
        runtimeconfig_path: PathBuf,
    },
}

/// Loads plugins from path from config.
pub fn load_plugins(_receiver: Mutex<Receiver<String>>) {
    unsafe {
        let libraries_path = find_plugins();

        native::load_native_plugin_data(&libraries_path)
            .into_iter()
            .for_each(|native_plugin| {
                load_native(native_plugin);
            });

        let _dotnet_plugins_data = dotnet::load_dotnet_plugin_data(&libraries_path);
    };
}

unsafe fn load_native(el: native::NativePluginRuntimeInfo) {
    let callback = el.plugin_information.init_callback;
    let name = CStr::from_ptr(el.plugin_information.name).to_str().unwrap();
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let _lib = el._library;
            let config = CONFIG
                .plugins
                .config
                .get_key_value(name)
                .map(|(_, v)| crate::extract_config_ptr(v))
                .unwrap_or(ptr::null_mut())
                .cast_const();

            (callback)(config, api_callbacks::get_api());
        })
    });
}

/// Finds plugins for user's OS and returs their pathes.
fn find_plugins() -> Vec<FoundedPlugin> {
    let plugins_folder = &CONFIG.plugins.plugins_folder;

    let dir = Path::new(plugins_folder);

    let plugins = collect_files(dir).unwrap_or_default();
    let qualified_plugins = qualify_plugins(plugins);

    if qualified_plugins.is_empty() {
        info!(
            "No one plugins in folder '{}'.",
            dir.to_str().unwrap_or("ERROR DUE CASTING PLUGINS PATH")
        );
    } else {
        info!(
            "Found {} plugins: {:#?}",
            qualified_plugins.len(),
            qualified_plugins
        );
    }

    qualified_plugins
}

fn qualify_plugins(plugins_pathes: Vec<PathBuf>) -> Vec<FoundedPlugin> {
    let mut res = vec![];
    for p in &plugins_pathes {
        if p.extension() == Some(OsStr::new("so")) {
            res.push(FoundedPlugin::Native { path: p.clone() });
        } else if p.extension() == Some(OsStr::new("dll")) {
            let filename = p.file_name().and_then(|name| name.to_str()).unwrap();
            let filename = filename.replace(".dll", "");
            if let Some(founded_plugin) = plugins_pathes.iter().find(|el| {
                let file_name = el.file_name().unwrap();
                let file_name_str = file_name.to_str().unwrap();

                file_name_str.starts_with(&filename) && file_name_str.ends_with("json")
            }) {
                res.push(FoundedPlugin::Dotnet {
                    dll_path: p.clone(),
                    runtimeconfig_path: founded_plugin.clone(),
                });
            }
        }
    }

    res
}

fn collect_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files_with_extension = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            files_with_extension.push(path);
        }
    }

    Ok(files_with_extension)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadableRequest {
    pub request: String,
}

type ConfigEntry<'a> =
    &'a std::collections::HashMap<std::string::String, configuration::ConfigFieldType>;

fn extract_config_ptr(plugin_config: ConfigEntry) -> *mut i8 {
    let normalized_plugin_config = normalize_config(plugin_config);
    let stringified = serde_json::to_string(&normalized_plugin_config).unwrap();
    if let Ok(cstring) = CString::new(stringified.to_owned()) {
        CString::into_raw(cstring)
    } else {
        ptr::null_mut()
    }
}

fn normalize_config(
    plugin_config: &HashMap<String, configuration::ConfigFieldType>,
) -> HashMap<String, configuration::ConfigFieldType> {
    let mut res = HashMap::new();
    for (k, v) in plugin_config {
        let mut value_for_insert = v.to_owned();
        if let ConfigFieldType::Array(map) = v {
            let mut array_field = vec![String::new(); map.len()];
            for (i, element) in map {
                array_field[i - 1] = element.to_owned()
            }
            value_for_insert = ConfigFieldType::NormalizedArray(array_field);
        }
        res.insert(k.to_owned(), value_for_insert);
    }
    res
}
