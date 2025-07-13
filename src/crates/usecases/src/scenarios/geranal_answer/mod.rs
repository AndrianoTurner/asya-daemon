use services::llm_api::{self, LlmBackend};
use shared::event_system;

use crate::AsyaResponse;

pub async fn answer(api: impl LlmBackend, user_message: String) {
    let answer = api.request(user_message).await.unwrap();
    event_system::publish(AsyaResponse::Ok {
        message: answer.to_string(),
    })
    .await;
}
