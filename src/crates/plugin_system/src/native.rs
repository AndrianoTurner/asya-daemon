use std::{
    ffi::{CStr, CString},
    ptr,
};
use tracing::*;

use libloading::Library;

use plugin_interface::{NativePluginInformation, State};
use shared::configuration::CONFIG;
use tracing::warn;

use crate::{api_callbacks, FoundedPlugin};

// todo: редизайн типов чтобы такой хуеты как с Library не было
// !! порядок полей менять НЕЛЬЗЯ тоже может быть сегфолт
pub(crate) struct NativePluginRuntimeInfo {
    pub(crate) plugin_information: Box<NativePluginInformation>,
    pub(crate) _library: Library, // это поле вообще никгде не юзается, но без него сегфолт.
    pub(crate) state: *mut State,
}

pub unsafe fn load_native_plugin_data(libs: Vec<FoundedPlugin>) -> Vec<NativePluginRuntimeInfo> {
    const FN_PLUGIN_INFO: &[u8; 11] = b"plugin_info";
    let mut infos = vec![];
    for lib in libs {
        if let FoundedPlugin::Native { path: lib } = lib {
            let library = match Library::new(&lib) {
                Ok(lib) => lib,
                Err(err) => {
                    warn!("Library {:?} wasn't loaded due error: {}", lib, err);
                    continue;
                }
            };
            let plugin_information_callback = match library
                .get::<*mut plugin_interface::PluginInfoCallback>(FN_PLUGIN_INFO)
            {
                Ok(callback) => callback.read(),
                Err(err) => {
                    warn!(
                "Library {:?} wasn't loaded 
                    because lib doesn't containt valid FN_PLUGIN_INFO function or / and it's signature is incorrect. | {}",
                lib,
                err
            );
                    continue;
                }
            };

            let boxed_plugin_information = Box::from_raw(plugin_information_callback().cast_mut());

            let str_plugin_name = match CStr::from_ptr(boxed_plugin_information.name).to_str() {
                Ok(res) => res,
                Err(_) => {
                    warn!("Plugin not loaded: file '{:?}' represents a plugin with name, that contains non utf-8 characters.", lib);
                    continue;
                }
            };
            let config_ptr = CONFIG
                .plugins
                .config
                .get_key_value(str_plugin_name)
                .map(|(_, v)| crate::extract_config_ptr(v))
                .unwrap_or(ptr::null_mut());

            // тут сегфолтит
            let state = (boxed_plugin_information.init_callback)(
                config_ptr.cast_const(),
                dbg!(api_callbacks::get_api()),
            );
            info!("Plugin loaded: {}", str_plugin_name,);

            infos.push(NativePluginRuntimeInfo {
                _library: library,
                state,
                plugin_information: boxed_plugin_information,
            });

            if !config_ptr.is_null() {
                let _ = CString::from_raw(config_ptr);
            }
        }
    }
    infos
}
