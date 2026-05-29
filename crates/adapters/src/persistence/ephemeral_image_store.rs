use std::collections::HashMap;
use std::sync::Arc;

use app::app_error::{AppError, AppResult};
use app::conversation::{
    EphemeralImageData, EphemeralImageStorePort, StoreEphemeralImageInput, StoredEphemeralImage,
};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use ring::rand::{SecureRandom, SystemRandom};
use tokio::sync::RwLock;

const DEFAULT_TTL_HOURS: i64 = 24;

#[derive(Clone)]
pub struct InMemoryEphemeralImageStore {
    entries: Arc<RwLock<HashMap<String, ImageEntry>>>,
    ttl: Duration,
}

#[derive(Clone)]
struct ImageEntry {
    mime_type: String,
    bytes: Vec<u8>,
    expires_at: DateTime<Utc>,
}

impl InMemoryEphemeralImageStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::hours(DEFAULT_TTL_HOURS),
        }
    }

    fn next_asset_id() -> AppResult<String> {
        let random = SystemRandom::new();
        let mut bytes = [0_u8; 8];
        random
            .fill(&mut bytes)
            .map_err(|err| AppError::internal(format!("failed to generate image asset id: {err}")))?;
        let suffix = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        Ok(format!("img_{}_{}", Utc::now().timestamp_millis(), suffix))
    }
}

#[async_trait]
impl EphemeralImageStorePort for InMemoryEphemeralImageStore {
    async fn put_image(
        &self,
        input: StoreEphemeralImageInput,
    ) -> AppResult<StoredEphemeralImage> {
        let asset_id = Self::next_asset_id()?;
        let size_bytes = input.bytes.len() as u64;
        let entry = ImageEntry {
            mime_type: input.mime_type.clone(),
            bytes: input.bytes,
            expires_at: Utc::now() + self.ttl,
        };
        self.entries.write().await.insert(asset_id.clone(), entry);
        Ok(StoredEphemeralImage {
            asset_id,
            mime_type: input.mime_type,
            size_bytes,
            width: input.width,
            height: input.height,
        })
    }

    async fn load_image(&self, asset_id: &str) -> AppResult<EphemeralImageData> {
        let now = Utc::now();
        let maybe_entry = self.entries.read().await.get(asset_id).cloned();
        let Some(entry) = maybe_entry else {
            return Err(AppError::NotFound(format!(
                "ephemeral image not found: {asset_id}"
            )));
        };
        if entry.expires_at <= now {
            self.entries.write().await.remove(asset_id);
            return Err(AppError::NotFound(format!(
                "ephemeral image expired: {asset_id}"
            )));
        }
        Ok(EphemeralImageData {
            asset_id: asset_id.to_string(),
            mime_type: entry.mime_type,
            bytes: entry.bytes,
        })
    }

    async fn delete_image(&self, asset_id: &str) -> AppResult<()> {
        self.entries.write().await.remove(asset_id);
        Ok(())
    }

    async fn cleanup_expired(&self) -> AppResult<()> {
        let now = Utc::now();
        self.entries
            .write()
            .await
            .retain(|_, entry| entry.expires_at > now);
        Ok(())
    }
}
