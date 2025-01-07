use std::ffi::c_char;
use std::ffi::CStr;

use tracing::warn;

use crate::event_system;

use super::{Event, PluginEvent, Sender};

/// Sends event from plugin to event bus as `String`.
///
/// `sender` ptr must not be cosumed.
/// `event` ptr will be consumed and rust frees them.
pub async unsafe fn send_plugin_event_checked(sender: Sender, event: Event) {
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

// TODO: refactor from tuple `(sender: String, name: String)` to tuple of structs: `(Sender(String), Event(String))`

/// Just alias to `abstractions::cstring_safety_cast` and `abstractions::cstring_safety_consume`
/// for event and name
///
/// returns (sender, event)
pub unsafe fn safe_cast_name_event(
    sender_ptr: *const i8,
    event_ptr: *const i8,
) -> Option<(Sender, Event)> {
    let Some(sender) = cstring_safety_cast(sender_ptr) else {
        warn!(
            "Some plugin has send event with corrupted 'sender_ptr'. 
                The pointer must be valid and must represent a valit UTF-8 string"
        );
        return None;
    };

    let Some(event) = cstring_safety_cast(event_ptr) else {
        warn!(
            "Plugin '{}' has send event with corrupted 'event_ptr'. 
                The pointer must be valid and must represent a valit UTF-8 string",
            sender
        );
        return None;
    };
    Some((sender.into(), event.into()))
}
