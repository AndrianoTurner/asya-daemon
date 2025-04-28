use crate::usecases::InternalUsecases;
use plugin_system::ReadableRequest;
use serde::{Deserialize, Serialize};
use services::llm_api;
use shared::event_system;
use std::sync::Arc;
use tokio::task;
use tracing::*;

pub mod scenarios;
pub mod shared_workers;
mod tools;
pub mod usecases;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub struct UsecaseMeta {
    name: String,
    id_receiver: String, // uuid
    payload_type: String,
    payload: serde_json::Value,
}

impl UsecaseMeta {
    pub fn new(name: String, id_receiver: String, payload_type: String, payload: String) -> Self {
        UsecaseMeta {
            name,
            id_receiver,
            payload_type,
            payload: serde_json::from_str(&payload).unwrap(),
        }
    }

    pub async fn execute(self, message: String) {
        if self.id_receiver == "asya-daemon" {
            match serde_json::from_value::<InternalUsecases>(self.payload.clone()) {
                Ok(usecase_internal) => {
                    usecase_internal.dispatch(message).await;
                }
                Err(err) => warn!("Error parsing usecase: {:?}", err),
            }
        }
        event_system::publish(self).await;
    }
}

// fn process_response(llm_response: &str) -> Result<InternalUsecases, Box<dyn std::error::Error>> {
//     let llm_response = llm_response.replace("`json", "");
//     let llm_response = llm_response.replace("`", "");
//     let usecase = serde_json::from_str::<InternalUsecases>(&llm_response.clone())?;
//     Ok(usecase)
// }
//
pub async fn subscribe_for_plugins() {
    event_system::subscribe_once({
        move |event: Arc<ReadableRequest>| {
            task::spawn(async move {
                dispatch_by_user_message(event.request.clone()).await;
            })
        }
    })
    .await
}

pub async fn dispatch_by_user_message(message: String) {
    let schema = schemars::schema_for!(InternalUsecases);
    if let Ok(usecase_json_string) = llm_api::send_request(
        format!(
        "Translate this userinput \"{}\" into json value following by following json schemes and send me only generated json without any other text. 
                Generated json will be used as value of existing json object. 
            If value doesn't have an key return value without brases, but save json syntax please. {}", message, serde_json::to_string(&schema).unwrap()
        )
    ).await {
        let usecase = serde_json::from_str::<InternalUsecases>(&usecase_json_string)
            .unwrap_or(InternalUsecases::Answer);
        usecase.dispatch(message).await;
    };
}

// general purpose events

/// General response event. Use it to send responses to the client.
/// How event works see [`shared::event_system`].
#[derive(Debug, parse_display::Display, Serialize)]
#[serde(tag = "asyaResponse")]
#[serde(rename_all = "camelCase")]
pub enum AsyaResponse {
    /// Success response with message from Asya.
    ///
    /// # Arguments
    ///     * `message` - human readable message from Asya, e.g.
    ///         "I've turned off the music. Don't listen this shit anymore."
    ///
    /// # Example
    ///
    /// ```
    /// event_system::publish(AsyaResponse::Ok {
    ///     message: "Hi, Vitaliy! I heard that u like thinkpads? Me too!"
    /// }
    ///
    /// ```
    #[display("{message}")]
    Ok { message: String },
}
