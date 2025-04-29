use lazy_static::lazy_static;
use plugin_interface::{ApiCallbacksMap, ApiCallbacksPair};
use std::ffi::{c_char, c_void, CString};

use tracing::*;

use crate::event_system;

use super::{abstractions, ReadableRequest};

pub unsafe fn get_api() -> ApiCallbacksMap {
    let callbacks = vec![
        // ApiCallbacksPair::new("send_human_request", send_human_request as *const c_void),
        ApiCallbacksPair::new("subscribe_to_events", subscribe_to_events as *const c_void),
        ApiCallbacksPair::new("publish_event", publish_event as *const c_void),
    ];

    ApiCallbacksMap::new(callbacks)
}

lazy_static! {
    static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
}

// #[no_mangle]
// unsafe extern "C" fn send_human_request(human: *mut c_char) {
//     let from_raw = CString::from_raw(human);
//     RUNTIME.spawn(async move {
//         let cstring_cast = from_raw.to_str();
//         match cstring_cast {
//             Ok(casted_str) => {
//                 event_system::publish(ReadableRequest {
//                     request: casted_str.to_string(),
//                 })
//                 .await;
//             }
//             Err(err) => warn!("Error due send_human_response API call: {}", err),
//         }
//     });
// }

#[no_mangle]
unsafe extern "C" fn subscribe_to_events(callback: unsafe extern "C" fn(*const c_char)) {
    tracing::debug!("Subscribing to events");
    RUNTIME.spawn(async move {
        loop {
            let (_, rx) = event_system::get_channel().await;
            let mut lock = rx.lock().await;
            let res = lock.recv().await.unwrap();
            let s = CString::new(res).unwrap();
            callback(s.into_raw().cast_const());
        }
    });
}

#[no_mangle]
unsafe extern "C" fn publish_event(sender_ptr: *const c_char, event_ptr: *const c_char) {
    let Some((sender_string, event_string)) =
        abstractions::safe_cast_name_event(sender_ptr, event_ptr)
    else {
        return;
    };

    RUNTIME.spawn(async move {
        abstractions::send_plugin_event_checked(sender_string, event_string).await
    });
}
