use derive_more::derive::{Deref, From, Into};
use plugin_interface::{EventState, State};
use serde::Serialize;
use std::{
    collections::HashMap,
    ffi::{CStr, CString, OsStr},
    fs, io,
    path::{Path, PathBuf},
    ptr::{self},
    thread,
    time::Duration,
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
pub fn load_plugins(receiver: Mutex<Receiver<String>>) {
    unsafe {
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let libraries_path = find_plugins();

                let mut native_plugins_data =
                    native::load_native_plugin_data(libraries_path.clone());
                let dotnet_plugins_data = dotnet::load_dotnet_plugin_data(libraries_path);

                do_loop(&mut native_plugins_data, receiver).await
            })
        })
    };
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

async unsafe fn do_loop(
    plugins_data: &mut [native::NativePluginRuntimeInfo],
    receiver: Mutex<Receiver<String>>,
) {
    let mut recv = receiver.lock().await;
    loop {
        for info in &mut *plugins_data {
            if !recv.is_empty() {
                check_event_for_send(info, &mut recv).await;
            } else {
                (info.plugin_information.execute_callback)(info.state, api_callbacks::get_api());
            }
            check_event_for_publish(info).await;
        }
        tokio::time::sleep(Duration::from_micros(100)).await;
    }
}

async unsafe fn check_event_for_send(
    info: &mut native::NativePluginRuntimeInfo,
    event_recv: &mut tokio::sync::MutexGuard<'_, Receiver<String>>,
) {
    let event_callback = info.plugin_information.event_callback;
    let recieved_event = event_recv.recv().await;

    // Maybe we should pass some set of events instead one?
    let ptr = extract_ptr(recieved_event);

    let event_state = Box::into_raw(Box::new(EventState {
        state: info.state,
        event: ptr,
    }));
    (event_callback)(event_state, api_callbacks::get_api());
}

fn extract_ptr(res: Option<String>) -> *const std::ffi::c_char {
    CString::new(res.expect("mpsc for events was closed. this is a bug."))
        // release ownership here because memory frees in free_event_memory()
        .map(|cstring| cstring.into_raw().cast_const())
        .unwrap_or_else(|_| {
            warn!("Event string representation contains zero byte, which is not allowed.");
            ptr::null()
        })
}

async unsafe fn check_event_for_publish(info: &mut native::NativePluginRuntimeInfo) {
    if let Some(plugin_state) = ptr::NonNull::new(info.state) {
        check_event(plugin_state, info).await;
        check_request(plugin_state).await;

        free_memory(info);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadableRequest(pub String);

async unsafe fn check_request(plugin_state: ptr::NonNull<State>) {
    if let Some(request_ptr) = ptr::NonNull::new(plugin_state.read().human_request) {
        let request_data = CStr::from_ptr(request_ptr.as_ptr()).to_str();
        if let Ok(str_data) = request_data {
            event_system::publish(ReadableRequest(str_data.to_string())).await;
        }
    }
}

async unsafe fn check_event(
    plugin_state: ptr::NonNull<State>,
    info: &mut native::NativePluginRuntimeInfo,
) {
    if let Some(published_event) = ptr::NonNull::new(plugin_state.read().published_event) {
        let (sender_string, event_string) = match abstractions::safe_cast_name_event(
            info.plugin_information.name,
            published_event.as_ptr(),
        ) {
            Some(value) => value,
            None => return,
        };
        _ = abstractions::send_plugin_event_checked(sender_string, event_string).await;
    }
}

unsafe fn free_memory(info: &mut native::NativePluginRuntimeInfo) {
    let event_raw = (*info.state).published_event;
    if !event_raw.is_null() {
        drop(Box::from_raw(event_raw)); // panics if plugins frees memory independently, e.g. if after
                                        // casting event to CString
        (*info.state).published_event = ptr::null_mut()
    }

    let message_raw = (*info.state).readable_message;
    if !message_raw.is_null() {
        drop(Box::from_raw(message_raw)); // same as above
        (*info.state).readable_message = ptr::null_mut()
    }

    let request_raw = (*info.state).human_request;
    if !message_raw.is_null() {
        drop(Box::from_raw(request_raw)); // same as above
        (*info.state).human_request = ptr::null_mut()
    }
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
