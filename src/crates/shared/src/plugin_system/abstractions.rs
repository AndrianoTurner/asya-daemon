use std::ffi::c_char;
use std::ffi::CStr;
use std::ffi::CString;

use tracing::warn;

use crate::event_system;

use super::PluginEvent;

/// Sends event from plugin to event bus as `String`.
///
/// `sender` ptr must not be cosumed.
/// `event` ptr will be consumed and rust frees them.
pub async unsafe fn send_plugin_event_checked(sender: String, event: String) {
    let general_event = PluginEvent {
        sender,
        data: event,
    };
    event_system::publish(general_event).await;
}

/// Cast chars from C whithout owning them.
pub unsafe fn cstring_safety_cast(chars: *const c_char) -> Option<String> {
    match CStr::from_ptr(chars).to_str() {
        Ok(string) => Some(string.to_string()),
        Err(_utferr) => None,
    }
}

/// Consumes chars from C
pub unsafe fn cstring_safety_consume(chars: *mut c_char) -> Option<String> {
    match CString::from_raw(chars).to_str() {
        Ok(string) => Some(string.to_string()),
        Err(_utferr) => None,
    }
}

/// Just alias to `abstractions::cstring_safety_cast` and `abstractions::cstring_safety_consume`
/// for event and name
pub unsafe fn safe_cast_name_event(
    sender_ptr: *const i8,
    event_ptr: *mut i8,
) -> Option<(String, String)> {
    let sender_string = if let Some(sender) = cstring_safety_cast(sender_ptr) {
        sender
    } else {
        warn!(
            "Some plugin send event, but send corrupted 'sender_ptr'. 
                Pointer must be valid and must represent a valit UTF-8 string"
        );
        return None;
    };
    let event_string = if let Some(sender) = cstring_safety_consume(event_ptr) {
        sender
    } else {
        warn!(
            "Plugin '{}' send event, but send corrupted 'event_ptr'. 
                Pointer must be valid and must represent a valit UTF-8 string",
            sender_string
        );
        return None;
    };
    Some((event_string, sender_string))
}
