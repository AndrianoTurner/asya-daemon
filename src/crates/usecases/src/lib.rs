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

#[derive(Serialize, Deserialize, Debug)]
#[repr(C)]
pub struct UsecaseMeta {
    name: String,
    id_receiver: String, // uuid
    payload_type: String,
    ai_desc: String, // todo: change to hashmap for diff localizations
    payload: serde_json::Value,
}

impl UsecaseMeta {
    pub fn new(name: String, id_receiver: String, ai_desc: String, payload_type: String, payload: String) -> Self {
        UsecaseMeta {
            name,
            id_receiver,
            payload_type,
            payload: serde_json::from_str(&payload).unwrap(),
            ai_desc,
        }
    }

    pub async fn execute(&self, message: String) {
        match serde_json::from_value::<InternalUsecases>(self.payload.clone()) {
            Ok(usecase_internal) => {
                println!("executing usecase: {:#?}", usecase_internal);
                usecase_internal.execute(message).await;
            }
            Err(err) => println!("Error parsing usecase: {:?}", err),
        }
    }
}

fn process_response(llm_response: &str) -> Result<InternalUsecases, Box<dyn std::error::Error>> {
    let llm_response = llm_response.replace("`json", "");
    let llm_response = llm_response.replace("`", "");
    let usecase = serde_json::from_str::<InternalUsecases>(&llm_response.clone())?;
    Ok(usecase)
}

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

    let c_req = format!(
        "
            Determine whether the following user input: {} is similar to any of the commands below. 
            {}
            If it is similar, respond with a single word: TRUE. 
            If it is not similar, respond with a single word: FALSE.
        ",
        message,
        serde_json::to_string_pretty(&schema).unwrap()
    );

    let c_llm_response = llm_api::send_request(c_req).await;
    let usecase = if c_llm_response.unwrap() == "TRUE" {
        let g_req = format!(
            "
                Generate json representation of command from this user input: {}
                by this json schema fo available commands: {}

                SEND ME ONLY GENERATED JSON

            ",
            message,
            serde_json::to_string_pretty(&schema).unwrap()
        );

        let g_llm_response = llm_api::send_request(g_req).await;

        if g_llm_response.is_err() {
            warn!("Error sending request to LLM: {:?}", g_llm_response.err());
            return;
        }
        let llm_response = g_llm_response.unwrap();
        debug!("LLM RESPONSE: {}", llm_response);
        let usecase = process_response(&llm_response);
        if let Err(err) = usecase {
            warn!("Error parsing response from LLM: {:?}", err);
            return;
        }
        usecase.unwrap()
    } else {
        usecases::InternalUsecases::Answer
    };
    usecase.execute(message).await;
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
