use reqwest::Client;
use serde_json::json;
use shared::{configuration, serde_extensions::get_json_value};
use super::AiRequestError;

pub async fn send_to_ollama(req: &str) -> Result<String, AiRequestError> {
    let http_client = Client::new();
    let data = json!({
        "model" : "mistral", // This should be configured
        "prompt": req,
        "stream" : false,
    });
    let req_url = format!("{}/api/generate",configuration::CONFIG.ai.ollama_base_url.trim_end_matches("/"));
    let response = http_client.post(&req_url).json(&data).send().await;
    match response {
        Ok(r) => {
            match r.text().await{
                Ok(response) =>{
                    get_json_value(&response, "response").ok_or(AiRequestError::OllamaRequest)
                },
                Err(e) => {
                    tracing::error!("Error deserializing OllamaResponse: {e}!");
                    Err(AiRequestError::OllamaRequest)
                },
            }
    
        },
        Err(e) => {
            tracing::error!("OllamaError {e}");
            Err(AiRequestError::OllamaRequest)
        }
    }
}