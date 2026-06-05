use std::sync::Arc;

use app::app_error::{AppError, AppResult};
use arrow_array::types::Float64Type;
use arrow_array::{
    ArrayRef, FixedSizeListArray, Float32Array, Float64Array, RecordBatch, RecordBatchIterator,
    StringArray,
};
use futures_util::TryStreamExt;
use lancedb::arrow::arrow_schema::{DataType, Field, Fields, Schema};
use lancedb::database::CreateTableMode;
use lancedb::query::{ExecutableQuery, QueryBase};
use rig::client::EmbeddingsClient;
use rig::embeddings::EmbeddingModel;
use rig::providers::openai;
use rig::vector_store::request::SearchFilter;
use rig::vector_store::{VectorSearchRequest, VectorStoreError, VectorStoreIndex};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::agent::memory::long_term::index::MemoryIndex;
use crate::agent::memory::types::{LongTermMemoryForm, MemoryRecord, MemoryScope, MemoryStatus};

const TABLE_NAME: &str = "memory_index";
const ID_FIELD: &str = "memory_id";
const USER_ID_FIELD: &str = "user_id";
const STATUS_FIELD: &str = "status";
const FORM_FIELD: &str = "form";
const SCOPE_FIELD: &str = "scope";
const CONTENT_FIELD: &str = "content";
const SOURCE_FIELD: &str = "source";
const UPDATED_AT_FIELD: &str = "updated_at";
const EMBEDDING_FIELD: &str = "embedding";

#[derive(Clone, Debug)]
pub struct RagConfig {
    pub lancedb_path: String,
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

pub type OpenAiCompatibleRagIndex =
    WarmmyLanceDbVectorIndex<rig::providers::openai::EmbeddingModel>;

pub struct WarmmyLanceDbVectorIndex<M: EmbeddingModel> {
    table: lancedb::Table,
    model: M,
}

#[derive(Clone)]
pub struct LanceDbMemoryIndex {
    config: RagConfig,
}

impl LanceDbMemoryIndex {
    pub fn new(config: RagConfig) -> Self {
        Self { config }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LanceDbFilter(Result<String, String>);

impl LanceDbFilter {
    fn into_inner(self) -> Result<String, VectorStoreError> {
        self.0
            .map_err(|err| VectorStoreError::BuilderError(err.to_string()))
    }
}

impl SearchFilter for LanceDbFilter {
    type Value = Value;

    fn eq(key: impl AsRef<str>, value: Self::Value) -> Self {
        Self(sql_comparison(key.as_ref(), "=", value))
    }

    fn gt(key: impl AsRef<str>, value: Self::Value) -> Self {
        Self(sql_comparison(key.as_ref(), ">", value))
    }

    fn lt(key: impl AsRef<str>, value: Self::Value) -> Self {
        Self(sql_comparison(key.as_ref(), "<", value))
    }

    fn and(self, rhs: Self) -> Self {
        Self(match (self.0, rhs.0) {
            (Ok(left), Ok(right)) => Ok(format!("({left}) AND ({right})")),
            (Err(err), _) | (_, Err(err)) => Err(err),
        })
    }

    fn or(self, rhs: Self) -> Self {
        Self(match (self.0, rhs.0) {
            (Ok(left), Ok(right)) => Ok(format!("({left}) OR ({right})")),
            (Err(err), _) | (_, Err(err)) => Err(err),
        })
    }
}

pub async fn build_rag_index(config: &RagConfig) -> AppResult<OpenAiCompatibleRagIndex> {
    config.validate()?;

    let model = build_embedding_model(config)?;
    let table = open_or_create_table(config).await?;

    Ok(WarmmyLanceDbVectorIndex { table, model })
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
    match db.open_table(TABLE_NAME).execute().await {
        Ok(table) => Ok(table),
        Err(lancedb::Error::TableNotFound { .. }) => create_empty_memory_table(&db, config).await,
        Err(err) => Err(AppError::database(err.to_string())),
    }
}

async fn create_empty_memory_table(
    db: &lancedb::Connection,
    config: &RagConfig,
) -> AppResult<lancedb::Table> {
    db.create_empty_table(TABLE_NAME, Arc::new(rag_schema(config.embedding_ndims)))
        .mode(CreateTableMode::Overwrite)
        .execute()
        .await
        .map_err(|e| AppError::database(e.to_string()))
}

#[async_trait::async_trait]
impl MemoryIndex for LanceDbMemoryIndex {
    async fn put(&self, record: &MemoryRecord) -> AppResult<()> {
        self.config.validate()?;
        if record.id.trim().is_empty() {
            return Err(AppError::internal("memory record id is required"));
        }
        if record.search_text.trim().is_empty() {
            tracing::info!(
                memory.id = record.id.as_str(),
                "memory indexing skipped: empty search text"
            );
            return Ok(());
        }

        tracing::info!(
            memory.id = record.id.as_str(),
            embedding.model = self.config.embedding_model.as_str(),
            embedding.ndims = self.config.embedding_ndims,
            "memory embedding requested"
        );
        let search_text = indexed_search_text(record);
        let model = build_embedding_model(&self.config)?;
        let embedding = model
            .embed_text(&search_text)
            .await
            .map_err(|e| AppError::upstream(e.to_string()))?;

        if embedding.vec.len() != self.config.embedding_ndims {
            return Err(AppError::internal(format!(
                "memory embedding dimension mismatch: expected {}, got {}",
                self.config.embedding_ndims,
                embedding.vec.len()
            )));
        }

        put_memory_record(&self.config, record, embedding.vec).await?;
        tracing::info!(
            memory.id = record.id.as_str(),
            path = self.config.lancedb_path.as_str(),
            "memory embedding indexed"
        );

        Ok(())
    }
}

async fn put_memory_record(
    config: &RagConfig,
    record: &MemoryRecord,
    embedding: Vec<f64>,
) -> AppResult<()> {
    let table = open_or_create_table(config).await?;
    let batch = rag_record_batch(config.embedding_ndims, record, embedding)
        .map_err(|e| AppError::database(e.to_string()))?;
    let reader: Box<dyn arrow_array::RecordBatchReader + Send> =
        Box::new(RecordBatchIterator::new(
            vec![Ok(batch)],
            Arc::new(rag_schema(config.embedding_ndims)),
        ));

    table
        .add(reader)
        .execute()
        .await
        .map(|_| ())
        .map_err(|e| AppError::database(e.to_string()))
}

fn rag_schema(dims: usize) -> Schema {
    Schema::new(Fields::from(vec![
        Field::new(ID_FIELD, DataType::Utf8, false),
        Field::new(USER_ID_FIELD, DataType::Utf8, false),
        Field::new(STATUS_FIELD, DataType::Utf8, false),
        Field::new(FORM_FIELD, DataType::Utf8, false),
        Field::new(SCOPE_FIELD, DataType::Utf8, false),
        Field::new(CONTENT_FIELD, DataType::Utf8, false),
        Field::new(SOURCE_FIELD, DataType::Utf8, true),
        Field::new(UPDATED_AT_FIELD, DataType::Utf8, false),
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
    record: &MemoryRecord,
    embedding: Vec<f64>,
) -> Result<RecordBatch, lancedb::arrow::arrow_schema::ArrowError> {
    let id_array = StringArray::from(vec![record.id.as_str()]);
    let user_id_array = StringArray::from(vec![record.user_id.as_str()]);
    let status_array = StringArray::from(vec![status_value(&record.status)]);
    let form_array = StringArray::from(vec![form_value(&record.form)]);
    let scope_array = StringArray::from(vec![scope_value(&record.scope)]);
    let content_array = StringArray::from(vec![record.content.as_str()]);
    let source_array = StringArray::from(vec!["memory_record"]);
    let updated_at_array = StringArray::from(vec![record.updated_at.as_str()]);
    let embedding_array = FixedSizeListArray::from_iter_primitive::<Float64Type, _, _>(
        vec![Some(embedding.into_iter().map(Some).collect::<Vec<_>>())],
        dims as i32,
    );

    RecordBatch::try_new(
        Arc::new(rag_schema(dims)),
        vec![
            Arc::new(id_array) as ArrayRef,
            Arc::new(user_id_array) as ArrayRef,
            Arc::new(status_array) as ArrayRef,
            Arc::new(form_array) as ArrayRef,
            Arc::new(scope_array) as ArrayRef,
            Arc::new(content_array) as ArrayRef,
            Arc::new(source_array) as ArrayRef,
            Arc::new(updated_at_array) as ArrayRef,
            Arc::new(embedding_array) as ArrayRef,
        ],
    )
}

impl<M> VectorStoreIndex for WarmmyLanceDbVectorIndex<M>
where
    M: EmbeddingModel + Send + Sync,
{
    type Filter = LanceDbFilter;

    async fn top_n<T: for<'a> Deserialize<'a> + Send>(
        &self,
        req: VectorSearchRequest<Self::Filter>,
    ) -> Result<Vec<(f64, String, T)>, VectorStoreError> {
        let table = &self.table;

        tracing::info!(
            rag.samples = req.samples(),
            rag.threshold = req.threshold().unwrap_or_default(),
            "memory rag search requested"
        );
        let embedding = self.model.embed_text(req.query()).await?;
        let mut query = table
            .vector_search(embedding.vec)
            .map_err(lancedb_to_vector_store_error)?
            .distance_type(lancedb::DistanceType::Cosine)
            .column(EMBEDDING_FIELD)
            .limit(req.samples() as usize);

        if let Some(threshold) = req.threshold() {
            query = query.distance_range(None, Some(threshold as f32));
        }

        if let Some(filter) = req.filter().clone() {
            query = query.only_if(filter.into_inner()?);
        }
        let stream = query
            .execute()
            .await
            .map_err(lancedb_to_vector_store_error)?;
        let batches = stream
            .try_collect::<Vec<_>>()
            .await
            .map_err(lancedb_to_vector_store_error)?;

        let mut results = Vec::new();
        for batch in batches {
            for row in 0..batch.num_rows() {
                let value = row_to_json(&batch, row);
                let id = value
                    .get(ID_FIELD)
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let distance = value
                    .get("_distance")
                    .and_then(Value::as_f64)
                    .unwrap_or_default();
                let document = serde_json::from_value(value)?;
                results.push((distance, id, document));
            }
        }

        tracing::info!(
            rag.result_count = results.len(),
            "memory rag search finished"
        );
        Ok(results)
    }

    async fn top_n_ids(
        &self,
        req: VectorSearchRequest<Self::Filter>,
    ) -> Result<Vec<(f64, String)>, VectorStoreError> {
        Ok(self
            .top_n::<Value>(req)
            .await?
            .into_iter()
            .map(|(score, id, _)| (score, id))
            .collect())
    }
}

fn row_to_json(batch: &RecordBatch, row: usize) -> Value {
    json!({
        ID_FIELD: string_value(batch, ID_FIELD, row).unwrap_or_default(),
        USER_ID_FIELD: string_value(batch, USER_ID_FIELD, row).unwrap_or_default(),
        STATUS_FIELD: string_value(batch, STATUS_FIELD, row).unwrap_or_default(),
        FORM_FIELD: string_value(batch, FORM_FIELD, row).unwrap_or_default(),
        SCOPE_FIELD: string_value(batch, SCOPE_FIELD, row).unwrap_or_default(),
        CONTENT_FIELD: string_value(batch, CONTENT_FIELD, row).unwrap_or_default(),
        SOURCE_FIELD: string_value(batch, SOURCE_FIELD, row).unwrap_or_default(),
        UPDATED_AT_FIELD: string_value(batch, UPDATED_AT_FIELD, row).unwrap_or_default(),
        "_distance": float_value(batch, "_distance", row).unwrap_or_default(),
    })
}

fn status_value(status: &MemoryStatus) -> &'static str {
    match status {
        MemoryStatus::Candidate => "candidate",
        MemoryStatus::Confirmed => "confirmed",
        MemoryStatus::Superseded => "superseded",
        MemoryStatus::Rejected => "rejected",
        MemoryStatus::Expired => "expired",
    }
}

fn form_value(form: &LongTermMemoryForm) -> &'static str {
    match form {
        LongTermMemoryForm::Semantic => "semantic",
        LongTermMemoryForm::Episodic => "episodic",
        LongTermMemoryForm::Procedural => "procedural",
    }
}

fn scope_value(scope: &MemoryScope) -> String {
    match scope {
        MemoryScope::Day { date } => format!("day:{date}"),
        MemoryScope::UserGlobal => "user_global".to_string(),
        MemoryScope::Temporary { expires_at } => format!("temporary:{expires_at}"),
    }
}

fn indexed_search_text(record: &MemoryRecord) -> String {
    match &record.scope {
        MemoryScope::Day { date } => format!("date: {date}\n{}", record.search_text),
        MemoryScope::UserGlobal => record.search_text.clone(),
        MemoryScope::Temporary { expires_at } => {
            format!("expires_at: {expires_at}\n{}", record.search_text)
        }
    }
}

fn string_value(batch: &RecordBatch, field: &str, row: usize) -> Option<String> {
    batch
        .column_by_name(field)?
        .as_any()
        .downcast_ref::<StringArray>()
        .map(|array| array.value(row).to_string())
}

fn float_value(batch: &RecordBatch, field: &str, row: usize) -> Option<f64> {
    let column = batch.column_by_name(field)?;
    if let Some(array) = column.as_any().downcast_ref::<Float32Array>() {
        return Some(array.value(row) as f64);
    }
    column
        .as_any()
        .downcast_ref::<Float64Array>()
        .map(|array| array.value(row))
}

fn lancedb_to_vector_store_error(err: lancedb::Error) -> VectorStoreError {
    VectorStoreError::DatastoreError(Box::new(err))
}

fn sql_comparison(field: &str, operator: &str, value: Value) -> Result<String, String> {
    let field = sql_identifier(field)?;
    let value = sql_literal(value)?;
    Ok(format!("{field} {operator} {value}"))
}

fn sql_identifier(value: &str) -> Result<String, String> {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        Ok(value.to_string())
    } else {
        Err(format!("unsupported lancedb filter field: {value}"))
    }
}

fn sql_literal(value: Value) -> Result<String, String> {
    match value {
        Value::String(value) => Ok(format!("'{}'", value.replace('\'', "''"))),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Null => Ok("NULL".to_string()),
        other => Err(format!("unsupported lancedb filter value: {other}")),
    }
}
