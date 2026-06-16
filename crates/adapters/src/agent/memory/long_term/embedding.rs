use app::app_error::{AppError, AppResult};
use domain::AIProviderKind;
use rig::client::EmbeddingsClient;
use rig::embeddings::EmbeddingModel;
use rig::http_client::HttpClientExt;
use rig::providers::openai;
use serde_json::{json, Value};

const DOUBAO_MULTIMODAL_EMBEDDINGS_PATH: &str = "/embeddings/multimodal";

pub(crate) fn build_openai_compatible_embedding_model(
    base_url: &str,
    api_key: &str,
    model: &str,
) -> AppResult<rig::providers::openai::EmbeddingModel> {
    let client = openai::Client::builder()
        .api_key(api_key)
        .base_url(base_url)
        .build()
        .map_err(|error| AppError::upstream(error.to_string()))?;

    // SiliconFlow and other OpenAI-compatible providers may reject OpenAI's
    // `dimensions` request field. Keep schema dimensions in config and do not
    // send a dimensions override to the embedding endpoint.
    Ok(client.embedding_model(model))
}

pub(crate) async fn embed_text_with_provider(
    provider: &AIProviderKind,
    base_url: &str,
    api_key: &str,
    model: &str,
    text: &str,
) -> AppResult<Vec<f64>> {
    match provider {
        AIProviderKind::Doubao => {
            embed_text_with_doubao_multimodal(base_url, api_key, model, text).await
        }
        _ => {
            let embedding_model = build_openai_compatible_embedding_model(base_url, api_key, model)?;
            let embedding = embedding_model
                .embed_text(text)
                .await
                .map_err(|error| AppError::upstream(error.to_string()))?;
            Ok(embedding.vec)
        }
    }
}

async fn embed_text_with_doubao_multimodal(
    base_url: &str,
    api_key: &str,
    model: &str,
    text: &str,
) -> AppResult<Vec<f64>> {
    let client = openai::Client::builder()
        .api_key(api_key)
        .base_url(base_url)
        .build()
        .map_err(|error| AppError::upstream(error.to_string()))?;
    let request_body = json!({
        "model": model,
        "input": [
            {
                "type": "text",
                "text": text,
            }
        ],
    });
    let request_body =
        serde_json::to_vec(&request_body).map_err(|error| AppError::upstream(error.to_string()))?;
    let request = client
        .post(DOUBAO_MULTIMODAL_EMBEDDINGS_PATH)
        .map_err(|error| AppError::upstream(error.to_string()))?
        .body(request_body)
        .map_err(|error| AppError::upstream(error.to_string()))?;
    let response = client
        .send(request)
        .await
        .map_err(|error| AppError::upstream(error.to_string()))?;
    let status = response.status();
    let response_body: Vec<u8> = response
        .into_body()
        .await
        .map_err(|error| AppError::upstream(error.to_string()))?;
    if !status.is_success() {
        let body_text = String::from_utf8_lossy(&response_body).to_string();
        tracing::warn!(
            embedding.provider = "doubao",
            http.status = %status,
            http.body = %compact_for_log(&body_text, 1000),
            "doubao multimodal embedding request failed"
        );
        return Err(AppError::upstream(format!(
            "doubao multimodal embedding request failed with status {status}: {}",
            compact_for_log(&body_text, 1000)
        )));
    }
    parse_doubao_embedding_response(response_body.as_slice())
}

fn parse_doubao_embedding_response(body: &[u8]) -> AppResult<Vec<f64>> {
    let response_json: Value =
        serde_json::from_slice(body).map_err(|error| AppError::upstream(error.to_string()))?;
    let Some(data) = response_json.get("data") else {
        return Err(AppError::upstream(format!(
            "doubao multimodal embedding response missing data: {}",
            compact_error_body(&response_json)
        )));
    };

    let embedding = if let Some(data_array) = data.as_array() {
        let Some(first) = data_array.first() else {
            return Err(AppError::upstream(
                "doubao multimodal embedding response has empty data array".to_string(),
            ));
        };
        first.get("embedding").and_then(Value::as_array)
    } else if data.is_object() {
        data.get("embedding").and_then(Value::as_array)
    } else {
        None
    };

    let Some(embedding) = embedding else {
        return Err(AppError::upstream(format!(
            "doubao multimodal embedding response missing embedding vector in data payload: {}",
            compact_error_body(&response_json)
        )));
    };
    let mut vector = Vec::with_capacity(embedding.len());
    for value in embedding {
        let Some(number) = value.as_f64() else {
            return Err(AppError::upstream(
                "doubao multimodal embedding response contains non-number item".to_string(),
            ));
        };
        vector.push(number);
    }
    Ok(vector)
}

fn compact_error_body(response_json: &Value) -> String {
    if let Some(message) = response_json
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
    {
        return message.to_string();
    }

    serde_json::to_string(response_json).unwrap_or_else(|_| "<invalid-json>".to_string())
}

fn compact_for_log(value: &str, max_chars: usize) -> String {
    let compact = value.trim().replace('\n', "\\n");
    if compact.chars().count() <= max_chars {
        return compact;
    }
    let prefix = compact.chars().take(max_chars).collect::<String>();
    format!("{prefix}…")
}
