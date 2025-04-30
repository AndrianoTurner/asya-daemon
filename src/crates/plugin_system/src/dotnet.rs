use crate::api_callbacks;

use super::FoundedPlugin;
use netcorehost::{nethost, pdcstr, pdcstring::PdCString};
use plugin_interface::ApiCallbacksMap;

pub struct DotnetRuntimePluginInfo {}

pub(crate) fn load_dotnet_plugin_data(
    libraries_path: &[FoundedPlugin],
) -> Vec<DotnetRuntimePluginInfo> {
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
                .get_function_with_unmanaged_callers_only::<fn(bebra: *const ApiCallbacksMap)>(
                    pdcstr!("AsyaDotnetPlugin.Program, AsyaDotnetPlugin"),
                    pdcstr!("Run"),
                )
                .unwrap();

            entry_point(Box::into_raw(Box::new(api_callbacks::get_api())));
        }
    }
    vec![]
}
