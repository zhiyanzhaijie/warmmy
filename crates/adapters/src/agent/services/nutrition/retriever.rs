use std::collections::HashSet;
use std::sync::Arc;

use app::agents::NutritionReferenceRetriever;
use app::app_error::{AppError, AppResult};
use app::meal::FoodNutritionReferenceRepositoryPort;
use arrow_array::types::Float64Type;
use arrow_array::{ArrayRef, FixedSizeListArray, RecordBatch, RecordBatchIterator, StringArray};
use async_trait::async_trait;
use futures_util::TryStreamExt;
use lancedb::arrow::arrow_schema::{DataType, Field, Fields, Schema};
use lancedb::database::CreateTableMode;
use lancedb::query::{ExecutableQuery, QueryBase, Select};
use tokio::sync::OnceCell;

use crate::agent::memory::long_term::embedding::embed_text_with_provider;
use crate::agent::memory::long_term::rag::RagConfig;
use domain::FoodNutritionReference;

const TABLE_NAME: &str = "nutrition_reference_index";
const ID_FIELD: &str = "reference_id";
const NAME_FIELD: &str = "name";
const SEARCH_TEXT_FIELD: &str = "search_text";
const STATUS_FIELD: &str = "status";
const SOURCE_FIELD: &str = "source";
const EMBEDDING_FIELD: &str = "embedding";

#[derive(Clone)]
pub struct LanceDbNutritionReferenceRetriever {
    repo: Arc<dyn FoodNutritionReferenceRepositoryPort>,
    config: RagConfig,
    indexed: Arc<OnceCell<()>>,
}

impl LanceDbNutritionReferenceRetriever {
    pub fn new(repo: Arc<dyn FoodNutritionReferenceRepositoryPort>, config: RagConfig) -> Self {
        Self {
            repo,
            config,
            indexed: Arc::new(OnceCell::new()),
        }
    }

    async fn ensure_indexed(&self) -> AppResult<()> {
        self.indexed
            .get_or_try_init(|| async {
                let references = self
                    .repo
                    .list_references()
                    .await
                    .map_err(AppError::database)?;
                let indexed_ids = list_indexed_reference_ids(&self.config).await?;
                let mut missing_count = 0usize;
                for reference in references {
                    if indexed_ids.contains(&reference.id) {
                        continue;
                    }
                    missing_count += 1;
                    self.put(&reference).await?;
                }
                tracing::info!(
                    nutrition.reference_index.missing_count = missing_count,
                    "nutrition reference index ensured"
                );
                Ok(())
            })
            .await
            .map(|_| ())
    }
}

#[async_trait]
impl NutritionReferenceRetriever for LanceDbNutritionReferenceRetriever {
    async fn retrieve(&self, query: &str) -> AppResult<Option<FoodNutritionReference>> {
        if query.trim().is_empty() {
            return Ok(None);
        }

        self.ensure_indexed().await?;
        let ids = search_reference_ids(&self.config, query).await?;
        tracing::info!(
            query = %query,
            nutrition.hit_count = ids.len(),
            "nutrition reference vector hits"
        );

        for id in ids {
            if let Some(reference) = self
                .repo
                .get_reference(&id)
                .await
                .map_err(AppError::database)?
            {
                if reference_matches_query(&reference, query) {
                    return Ok(Some(reference));
                }
                tracing::info!(
                    query = %query,
                    reference.id = %reference.id,
                    reference.name = %reference.name,
                    "nutrition reference vector hit rejected as semantic neighbor"
                );
            }
        }

        Ok(None)
    }

    async fn put(&self, reference: &FoodNutritionReference) -> AppResult<()> {
        put_reference(&self.config, reference).await
    }
}

async fn search_reference_ids(config: &RagConfig, query: &str) -> AppResult<Vec<String>> {
    config.validate()?;
    let table = open_or_create_table(config).await?;
    let embedding = embed_text_with_provider(
        &config.embedding_provider,
        &config.embedding_base_url,
        &config.embedding_api_key,
        &config.embedding_model,
        query,
    )
    .await?;
    if embedding.len() != config.embedding_ndims {
        return Err(AppError::internal(format!(
            "nutrition embedding dimension mismatch: expected {}, got {}",
            config.embedding_ndims,
            embedding.len()
        )));
    }

    let stream = table
        .vector_search(embedding)
        .map_err(|err| AppError::database(err.to_string()))?
        .distance_type(lancedb::DistanceType::Cosine)
        .column(EMBEDDING_FIELD)
        .limit(config.top_k)
        .execute()
        .await
        .map_err(|err| AppError::database(err.to_string()))?;

    let batches = stream
        .try_collect::<Vec<_>>()
        .await
        .map_err(|err| AppError::database(err.to_string()))?;
    let mut ids = Vec::new();
    for batch in batches {
        for row in 0..batch.num_rows() {
            if let Some(id) = string_value(&batch, ID_FIELD, row) {
                if !ids.iter().any(|existing| existing == &id) {
                    ids.push(id);
                }
            }
        }
    }
    Ok(ids)
}

async fn put_reference(config: &RagConfig, reference: &FoodNutritionReference) -> AppResult<()> {
    config.validate()?;
    if reference.id.trim().is_empty() {
        return Ok(());
    }

    let search_text = reference_search_text(reference);
    if search_text.trim().is_empty() {
        return Ok(());
    }

    tracing::info!(
        reference.id = reference.id.as_str(),
        embedding.model = config.embedding_model.as_str(),
        "nutrition reference embedding requested"
    );
    let embedding = embed_text_with_provider(
        &config.embedding_provider,
        &config.embedding_base_url,
        &config.embedding_api_key,
        &config.embedding_model,
        &search_text,
    )
    .await?;
    if embedding.len() != config.embedding_ndims {
        return Err(AppError::internal(format!(
            "nutrition embedding dimension mismatch: expected {}, got {}",
            config.embedding_ndims,
            embedding.len()
        )));
    }

    let table = open_or_create_table(config).await?;
    delete_reference_id(&table, &reference.id).await?;
    let batch = reference_record_batch(config.embedding_ndims, reference, search_text, embedding)
        .map_err(|err| AppError::database(err.to_string()))?;
    let reader: Box<dyn arrow_array::RecordBatchReader + Send> =
        Box::new(RecordBatchIterator::new(
            vec![Ok(batch)],
            Arc::new(reference_schema(config.embedding_ndims)),
        ));

    table
        .add(reader)
        .execute()
        .await
        .map(|_| ())
        .map_err(|err| AppError::database(err.to_string()))
}

async fn list_indexed_reference_ids(config: &RagConfig) -> AppResult<HashSet<String>> {
    let table = open_or_create_table(config).await?;
    let stream = table
        .query()
        .select(Select::columns(&[ID_FIELD]))
        .execute()
        .await
        .map_err(|err| AppError::database(err.to_string()))?;

    let batches = stream
        .try_collect::<Vec<_>>()
        .await
        .map_err(|err| AppError::database(err.to_string()))?;
    let mut ids = HashSet::new();
    for batch in batches {
        for row in 0..batch.num_rows() {
            if let Some(id) = string_value(&batch, ID_FIELD, row) {
                ids.insert(id);
            }
        }
    }
    Ok(ids)
}

async fn delete_reference_id(table: &lancedb::Table, id: &str) -> AppResult<()> {
    let predicate = format!("{ID_FIELD} = {}", sql_string_literal(id));
    table
        .delete(predicate.as_str())
        .await
        .map(|_| ())
        .map_err(|err| AppError::database(err.to_string()))
}

async fn open_or_create_table(config: &RagConfig) -> AppResult<lancedb::Table> {
    let db = lancedb::connect(&config.lancedb_path)
        .execute()
        .await
        .map_err(|err| AppError::database(err.to_string()))?;
    match db.open_table(TABLE_NAME).execute().await {
        Ok(table) => Ok(table),
        Err(lancedb::Error::TableNotFound { .. }) => create_empty_table(&db, config).await,
        Err(err) => Err(AppError::database(err.to_string())),
    }
}

async fn create_empty_table(
    db: &lancedb::Connection,
    config: &RagConfig,
) -> AppResult<lancedb::Table> {
    db.create_empty_table(
        TABLE_NAME,
        Arc::new(reference_schema(config.embedding_ndims)),
    )
    .mode(CreateTableMode::Overwrite)
    .execute()
    .await
    .map_err(|err| AppError::database(err.to_string()))
}

fn reference_schema(dims: usize) -> Schema {
    Schema::new(Fields::from(vec![
        Field::new(ID_FIELD, DataType::Utf8, false),
        Field::new(NAME_FIELD, DataType::Utf8, false),
        Field::new(SEARCH_TEXT_FIELD, DataType::Utf8, false),
        Field::new(STATUS_FIELD, DataType::Utf8, false),
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

fn reference_record_batch(
    dims: usize,
    reference: &FoodNutritionReference,
    search_text: String,
    embedding: Vec<f64>,
) -> Result<RecordBatch, lancedb::arrow::arrow_schema::ArrowError> {
    let schema = Arc::new(reference_schema(dims));
    let embedding_array = FixedSizeListArray::from_iter_primitive::<Float64Type, _, _>(
        vec![Some(embedding.into_iter().map(Some).collect::<Vec<_>>())],
        dims as i32,
    );

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec![reference.id.clone()])) as ArrayRef,
            Arc::new(StringArray::from(vec![reference.name.clone()])) as ArrayRef,
            Arc::new(StringArray::from(vec![search_text])) as ArrayRef,
            Arc::new(StringArray::from(vec![reference.status.as_str()])) as ArrayRef,
            Arc::new(StringArray::from(vec![reference.source.clone()])) as ArrayRef,
            Arc::new(embedding_array) as ArrayRef,
        ],
    )
}

fn reference_search_text(reference: &FoodNutritionReference) -> String {
    reference.search_texts().join("\n")
}

fn reference_matches_query(reference: &FoodNutritionReference, query: &str) -> bool {
    let query = normalize_match_text(query);
    if query.is_empty() {
        return false;
    }

    reference
        .search_texts()
        .into_iter()
        .map(|text| normalize_match_text(&text))
        .any(|text| {
            !text.is_empty()
                && (text == query
                    || text.contains(&query)
                    || query.contains(&text) && text.chars().count() >= 2)
        })
}

fn normalize_match_text(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '_' && *ch != '-')
        .collect()
}

fn string_value(batch: &RecordBatch, field: &str, row: usize) -> Option<String> {
    batch
        .column_by_name(field)?
        .as_any()
        .downcast_ref::<StringArray>()
        .map(|array| array.value(row).to_string())
}

fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}