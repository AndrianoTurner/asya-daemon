use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use super::AiRequestError;

#[derive(Deserialize)]
struct OllamaResponse{
    model : String,
    created_at : String,
    pub response: String,
    done: bool,
    context: Vec<u32>,
    total_duration: usize,
    load_duration: usize,
    prompt_eval_count: usize,
    prompt_eval_duration: usize,
    eval_count: usize,
    eval_duration: usize
}

pub async fn send_to_ollama(req: &str) -> Result<String, AiRequestError> {
    let http_client = Client::new();
    let data = json!({
        "model" : "llama3.2",
        "prompt" : req,
        "stream" : false,
    });
    let response = http_client.post("http://localhost:11434/api/generate").json(&data).send().await;
    match response {
        Ok(r) => {
            if let Ok(data) = r.json::<OllamaResponse>().await{
                Ok(data.response)
            }
            else {
                Err(AiRequestError::OllamaRequest)
            }
        },
        Err(_) => Err(AiRequestError::OllamaRequest)
    }
}