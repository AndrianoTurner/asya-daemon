use std::ffi::c_char;

use super::FoundedPlugin;
use netcorehost::{hostfxr::ManagedFunction, nethost, pdcstr, pdcstring::PdCString};
use plugin_interface::ApiCallbacksMap;

pub struct DotnetRuntimePluginInfo {
    pub name: String,
    pub entry_point: ManagedFunction<extern "system" fn(*const c_char, *const ApiCallbacksMap)>,
}

pub(crate) fn load_dotnet_plugin_data(
    libraries_path: &[FoundedPlugin],
) -> Vec<DotnetRuntimePluginInfo> {
    let mut res = Vec::with_capacity(libraries_path.len());
    for lib in libraries_path {
        if let FoundedPlugin::Dotnet {
            dll_path,
            runtimeconfig_path,
        } = lib
        {
            let hostfxr = nethost::load_hostfxr().unwrap();
            let context = hostfxr
                .initialize_for_runtime_config(
                    PdCString::from_os_str(runtimeconfig_path.as_os_str()).unwrap(),
                )
                .unwrap();
            let delegate_loader = context
                .get_delegate_loader_for_assembly(
                    PdCString::from_os_str(dll_path.as_os_str()).unwrap(),
                )
                .unwrap();

            let entry_point = delegate_loader
                .get_function_with_unmanaged_callers_only::<fn(config: *const c_char, callbacks: *const ApiCallbacksMap)>(
                    pdcstr!("AsyaDotnetPlugin.Program, AsyaDotnetPlugin"),
                    pdcstr!("Run"),
                )
                .unwrap();

            res.push(DotnetRuntimePluginInfo {
                name: dll_path.file_name().unwrap().to_str().unwrap().to_string(),
                entry_point,
            });
        }
    }
    res
}
