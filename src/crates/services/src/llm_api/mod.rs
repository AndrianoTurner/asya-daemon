use alta_s_api::send_to_altas;
use async_trait::async_trait;
use groq_api::send_to_groq;
use reqwest::Client;
use shared::{configuration::CONFIG, types::AiRecognizeMethod};
use tracing::*;

use crate::llm_api::{alta_s_api::AltaSBackend, groq_api::GroqBackend};

mod alta_s_api;
mod groq_api;

// todo: покрыть все ошибки, а не те которые мне по кайфу щас
#[derive(Debug)]
pub enum AiRequestError {
    GroqApiKey,
    GroqRequest,
    AltaSUrl,
    AltaSRequest,
}
#[async_trait::async_trait]
pub trait LlmBackend: Send + Sync {
    async fn request(&self, request: String) -> Result<String, AiRequestError>;
}
pub enum LLMBackendKind {
    Groq(GroqBackend),
    AltaS(AltaSBackend),
}
#[async_trait]
impl LlmBackend for LLMBackendKind {
    async fn request(&self, request: String) -> Result<String, AiRequestError> {
        match self {
            LLMBackendKind::Groq(backend) => backend.request(request).await,
            LLMBackendKind::AltaS(backend) => backend.request(request).await,
        }
    }
}

pub struct LLMApiBuilder;
impl LLMApiBuilder {
    pub fn new(backend_type: AiRecognizeMethod) -> Option<LLMBackendKind> {
        match backend_type {
            AiRecognizeMethod::Groq => Some(LLMBackendKind::Groq(GroqBackend)),
            AiRecognizeMethod::AltaS => Some(LLMBackendKind::AltaS(AltaSBackend)),
            AiRecognizeMethod::None => None,
        }
    }
}

async fn request(
    client: Client,
    url: &str,
    api_key: String,
    body: serde_json::Value,
) -> reqwest::Response {
    client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .unwrap()
}
