use std::sync::Arc;

use app::app_error::{AppError, AppResult};
use arrow_array::types::Float64Type;
use arrow_array::{ArrayRef, FixedSizeListArray, RecordBatch, RecordBatchIterator, StringArray};
use lancedb::arrow::arrow_schema::{DataType, Field, Fields, Schema};
use rig::client::EmbeddingsClient;
use rig::embeddings::EmbeddingModel;
use rig::lancedb::{LanceDbVectorIndex, SearchParams};
use rig::providers::openai;

const TABLE_NAME: &str = "agent_context";
const ID_FIELD: &str = "id";
const CONTENT_FIELD: &str = "content";
const SOURCE_FIELD: &str = "source";
const EMBEDDING_FIELD: &str = "embedding";

#[derive(Clone, Debug)]
pub struct RagConfig {
    pub lancedb_path: String,
    pub embedding_provider: String,
    pub embedding_base_url: String,
    pub embedding_api_key: String,
    pub embedding_model: String,
    pub embedding_ndims: usize,
    pub top_k: usize,
}

impl RagConfig {
    pub fn validate(&self) -> AppResult<()> {
        if self.lancedb_path.trim().is_empty() {
            return Err(AppError::internal("rag lancedb_path is required"));
        }
        if self.embedding_api_key.trim().is_empty() {
            return Err(AppError::internal("rag embedding api_key is required"));
        }
        if self.embedding_model.trim().is_empty() {
            return Err(AppError::internal("rag embedding model is required"));
        }
        if self.top_k == 0 {
            return Err(AppError::internal("rag top_k must be greater than 0"));
        }
        if self.embedding_ndims == 0 {
            return Err(AppError::internal(
                "rag embedding_ndims must be greater than 0",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct RagDocument {
    pub id: String,
    pub content: String,
    pub source: String,
}

pub type OpenAiCompatibleRagIndex = LanceDbVectorIndex<rig::providers::openai::EmbeddingModel>;

pub async fn build_rag_index(config: &RagConfig) -> AppResult<OpenAiCompatibleRagIndex> {
    config.validate()?;

    let model = build_embedding_model(config)?;
    let table = open_or_create_table(config).await?;

    let search_params = SearchParams::default()
        .distance_type(lancedb::DistanceType::Cosine)
        .column(EMBEDDING_FIELD);

    LanceDbVectorIndex::new(table, model, ID_FIELD, search_params)
        .await
        .map_err(|e| AppError::database(e.to_string()))
}

pub async fn embed_rag_document(config: &RagConfig, projection: RagDocument) -> AppResult<()> {
    config.validate()?;

    if projection.id.trim().is_empty() {
        return Err(AppError::internal("rag document id is required"));
    }
    if projection.content.trim().is_empty() {
        return Err(AppError::internal("rag document content is required"));
    }
    if projection.source.trim().is_empty() {
        return Err(AppError::internal("rag document source is required"));
    }

    tracing::info!(
        id = projection.id.as_str(),
        source = projection.source.as_str(),
        content = projection.content.as_str(),
        "embedding rag document content"
    );

    let model = build_embedding_model(config)?;
    tracing::info!(
        provider = config.embedding_provider.as_str(),
        model = config.embedding_model.as_str(),
        source = projection.source.as_str(),
        content.len = projection.content.len(),
        "embedding started"
    );
    let embedding = model
        .embed_text(&projection.content)
        .await
        .map_err(|e| AppError::upstream(e.to_string()))?;

    if embedding.vec.len() != config.embedding_ndims {
        return Err(AppError::internal(format!(
            "rag embedding dimension mismatch: expected {}, got {}",
            config.embedding_ndims,
            embedding.vec.len()
        )));
    }

    tracing::info!(
        provider = config.embedding_provider.as_str(),
        model = config.embedding_model.as_str(),
        source = projection.source.as_str(),
        embedding.dims = embedding.vec.len(),
        "embedding finished"
    );

    let table = open_or_create_table(config).await?;
    let batch = rag_record_batch(
        config.embedding_ndims,
        &projection.id,
        &projection.content,
        &projection.source,
        embedding.vec,
    )
    .map_err(|e| AppError::database(e.to_string()))?;
    let reader = RecordBatchIterator::new(
        vec![Ok(batch)],
        Arc::new(rag_schema(config.embedding_ndims)),
    );

    table
        .add(reader)
        .execute()
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    tracing::info!(
        id = projection.id.as_str(),
        source = projection.source.as_str(),
        "embedded rag document"
    );
    Ok(())
}

fn build_embedding_model(config: &RagConfig) -> AppResult<rig::providers::openai::EmbeddingModel> {
    let client = openai::Client::builder()
        .api_key(&config.embedding_api_key)
        .base_url(&config.embedding_base_url)
        .build()
        .map_err(|e| AppError::upstream(e.to_string()))?;

    // SiliconFlow and other OpenAI-compatible providers may reject OpenAI's
    // `dimensions` request field. Keep schema dimensions in config and do not
    // send a dimensions override to the embedding endpoint.
    Ok(client.embedding_model(&config.embedding_model))
}

async fn open_or_create_table(config: &RagConfig) -> AppResult<lancedb::Table> {
    let db = lancedb::connect(&config.lancedb_path)
        .execute()
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    let tables = db
        .table_names()
        .execute()
        .await
        .map_err(|e| AppError::database(e.to_string()))?;

    if tables.iter().any(|name| name == TABLE_NAME) {
        db.open_table(TABLE_NAME)
            .execute()
            .await
            .map_err(|e| AppError::database(e.to_string()))
    } else {
        db.create_empty_table(TABLE_NAME, Arc::new(rag_schema(config.embedding_ndims)))
            .execute()
            .await
            .map_err(|e| AppError::database(e.to_string()))
    }
}

fn rag_schema(dims: usize) -> Schema {
    Schema::new(Fields::from(vec![
        Field::new(ID_FIELD, DataType::Utf8, false),
        Field::new(CONTENT_FIELD, DataType::Utf8, false),
        Field::new(SOURCE_FIELD, DataType::Utf8, true),
        Field::new(
            EMBEDDING_FIELD,
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float64, true)),
                dims as i32,
            ),
            false,
        ),
    ]))
}

fn rag_record_batch(
    dims: usize,
    id: &str,
    content: &str,
    source: &str,
    embedding: Vec<f64>,
) -> Result<RecordBatch, lancedb::arrow::arrow_schema::ArrowError> {
    let id_array = StringArray::from(vec![id]);
    let content_array = StringArray::from(vec![content]);
    let source_array = StringArray::from(vec![source]);
    let embedding_array = FixedSizeListArray::from_iter_primitive::<Float64Type, _, _>(
        vec![Some(embedding.into_iter().map(Some).collect::<Vec<_>>())],
        dims as i32,
    );

    RecordBatch::try_new(
        Arc::new(rag_schema(dims)),
        vec![
            Arc::new(id_array) as ArrayRef,
            Arc::new(content_array) as ArrayRef,
            Arc::new(source_array) as ArrayRef,
            Arc::new(embedding_array) as ArrayRef,
        ],
    )
}
