use core::fmt;
use std::{
    ffi::{c_char, c_uint, c_void, CStr, CString},
    ptr, slice,
    str::FromStr,
};

pub type EventCallbalck = unsafe extern "C" fn(*const EventState, ApiCallbacksMap);
pub type ExecuteCallback = unsafe extern "C" fn(*mut State, ApiCallbacksMap);
pub type InitCallback = unsafe extern "C" fn(*const c_char, ApiCallbacksMap);

pub type NativePluginInfoCallback = unsafe extern "C" fn() -> *const NativePluginInformation;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EventState {
    pub state: *const State,
    pub event: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct State {
    pub published_event: *mut c_char,
    pub readable_message: *mut c_char,
    pub human_request: *mut c_char,
    pub data: *const c_void,
}

impl Default for State {
    fn default() -> Self {
        Self {
            published_event: ptr::null_mut(),
            readable_message: ptr::null_mut(),
            human_request: ptr::null_mut(),
            data: ptr::null_mut(),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct NativePluginInformation {
    pub name: *const c_char,
    // pub event_callback: EventCallbalck,
    pub init_callback: InitCallback,
    // pub execute_callback: ExecuteCallback,
}

#[repr(C)]
pub struct ApiCallbacksMap {
    callbacks: *const ApiCallbacksPair,
    callbacks_len: c_uint,
}

impl fmt::Debug for ApiCallbacksMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let slice: &[ApiCallbacksPair] =
                slice::from_raw_parts(self.callbacks, self.callbacks_len as usize);

            let mut debug_struct = f.debug_struct("ApiCallbacksMap");
            debug_struct.field("callbacks_len", &self.callbacks_len);

            let mut callbacks_info = Vec::new();
            for el in slice {
                let name = match CStr::from_ptr(el.callback_name).to_str() {
                    Ok(name) => name.to_string(),
                    Err(_) => "<invalid UTF-8>".to_string(),
                };
                let callback_ptr = format!("{:p}", el.callback);
                callbacks_info.push(format!(
                    "{{ name: \"{}\", callback: {} }}",
                    name, callback_ptr
                ));
            }

            // Добавляем callbacks как список
            debug_struct.field("callbacks", &callbacks_info);
            debug_struct.finish()
        }
    }
}

impl ApiCallbacksMap {
    pub fn new(callbacks: Vec<ApiCallbacksPair>) -> Self {
        let leaked = Box::leak(callbacks.into_boxed_slice());
        Self {
            callbacks_len: leaked.len() as u32,
            callbacks: leaked.as_ptr(),
        }
    }

    #[no_mangle]
    pub unsafe fn callback(&self, name: &str) -> *const c_void {
        for i in 0..self.callbacks_len {
            let current = self.callbacks.add(i as usize);
            let c_name = match CStr::from_ptr((*current).callback_name).to_str() {
                Ok(name) => name,
                Err(_) => continue,
            };

            if c_name == name {
                println!("калбеsaddк {}", name);
                return (*current).callback;
            }
        }
        ptr::null()
    }
}

#[repr(C)]
pub struct ApiCallbacksPair {
    callback_name: *const c_char,
    callback: *const c_void,
}

impl ApiCallbacksPair {
    pub fn new(name: &str, c_ptr: *const c_void) -> Self {
        Self {
            callback_name: CString::from_str(name).unwrap().into_raw(),
            callback: c_ptr,
        }
    }
}
