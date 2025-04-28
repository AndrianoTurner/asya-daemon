use std::
    ffi::CStr
;
use tracing::*;

use libloading::Library;

use plugin_interface::NativePluginInformation;
use tracing::warn;

use crate::FoundedPlugin;

// todo: редизайн типов чтобы такой хуеты как с Library не было
// !! порядок полей менять НЕЛЬЗЯ тоже может быть сегфолт
pub(crate) struct NativePluginRuntimeInfo {
    pub(crate) plugin_information: Box<NativePluginInformation>,
    pub(crate) _library: Library, // это поле вообще никгде не юзается, но без него сегфолт.
}

pub unsafe fn load_native_plugin_data(libs: &[FoundedPlugin]) -> Vec<NativePluginRuntimeInfo> {
    const FN_PLUGIN_INFO: &[u8; 11] = b"plugin_info";
    let mut infos = vec![];
    for lib in libs {
        if let FoundedPlugin::Native { path: lib } = lib {
            let library = match Library::new(lib) {
                Ok(lib) => lib,
                Err(err) => {
                    warn!("Library {:?} wasn't loaded due error: {}", lib, err);
                    continue;
                }
            };
            let plugin_information_callback = match library
                .get::<*mut plugin_interface::NativePluginInfoCallback>(FN_PLUGIN_INFO)
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

            info!("Plugin loaded: {}", str_plugin_name,);

            infos.push(NativePluginRuntimeInfo {
                _library: library,
                plugin_information: boxed_plugin_information,
            });
        }
    }
    infos
}
