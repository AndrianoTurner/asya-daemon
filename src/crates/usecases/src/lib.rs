use crate::usecases::InternalUsecases;
use plugin_system::ReadableRequest;
use serde::{Deserialize, Serialize};
use services::llm_api::{self, LlmBackend};
use shared::event_system;
use std::{ops::Deref, sync::Arc};
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

    pub async fn execute(self, api: impl LlmBackend, message: String) {
        if self.id_receiver == "asya-daemon" {
            match serde_json::from_value::<InternalUsecases>(self.payload.clone()) {
                Ok(usecase_internal) => {
                    usecase_internal.dispatch(api, message).await;
                }
                Err(err) => warn!("Error parsing usecase: {:?}", err),
            }
        }
        event_system::publish(self).await;
    }
}

pub async fn subscribe_for_plugins(api: Arc<impl LlmBackend + 'static>) {
    let api = api.clone();
    event_system::subscribe_once({
        move |event: Arc<ReadableRequest>| {
            let api = api.clone();
            task::spawn(async move {
                dispatch_by_user_message(api, event.request.clone()).await;
            })
        }
    })
    .await
}

pub async fn dispatch_by_user_message(api: Arc<impl LlmBackend>, message: String) {
    let schema = schemars::schema_for!(InternalUsecases);
    if let Ok(usecase_json_string) =api.request(
        format!(
            "Translate the user input stored in the variable USERINPUT into JSON format.
                You must determine which JSON object corresponds to the user input based on its meaning and the provided JSON schemas of possible JSON objects.
                You must choose only from the list of JSON objects from JSON Schemes.
                You must be certain that the schema you choose truly matches the description of the JSON object.
                You must ensure that the JSON object is valid and conforms to its schema.
                Provide only correct answers, as incorrect ones could harm people.
                You must not write curly braces around the resulting JSON object if the object does not represent a key-value pair.
                SEND ME ONLY RESULT JSON OBJECT WITHOUT ANY TEXT.
                USERINPUT: {} \n\n
                JSON Schemes: {}",
            message,
            serde_json::to_string(&schema).unwrap()
        )
    ).await {
        let usecase = dbg!(serde_json::from_str::<InternalUsecases>(&dbg!(usecase_json_string)))
            .unwrap_or(InternalUsecases::Answer);
        usecase.dispatch(api,message).await;
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
