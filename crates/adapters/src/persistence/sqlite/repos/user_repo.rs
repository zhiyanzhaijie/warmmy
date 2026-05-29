use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use app::user::{
    DiningCompanionRepositoryPort, SecretStorePort, UserAIConfigRepositoryPort,
    UserHealthExpectationRepositoryPort, UserPreferencesRepositoryPort, UserProfileRepositoryPort,
};
use domain::{
    AICapability, AIProviderKind, AppPreferences, DietaryPreferences, DiningCompanion,
    DiningCompanionId, ExpectationSource, HealthExpectationId, HealthExpectationKind,
    HealthExpectationStatus, UserAIProvider, UserAIRoute, UserHealthExpectation, UserId,
    UserPreferences, UserProfile,
};

use crate::crypto::SharedSecretCipher;
use crate::persistence::sqlite::models::{
    DiningCompanionRow, UserAIProviderRow, UserAIRouteRow, UserHealthExpectationRow,
    UserPreferencesRow, UserProfileRow, UserSecretRow,
};

#[derive(Clone)]
pub struct SqliteUserRepo {
    db: Arc<Mutex<toasty::Db>>,
    secret_cipher: SharedSecretCipher,
}

impl SqliteUserRepo {
    pub fn new(db: Arc<Mutex<toasty::Db>>, secret_cipher: SharedSecretCipher) -> Self {
        Self { db, secret_cipher }
    }

    pub async fn upsert_profile(&self, profile: &UserProfile) -> Result<(), String> {
        let id = profile.id.as_str().to_string();
        let mut db = self.db.lock().await;
        match UserProfileRow::get_by_id(&mut *db, &id).await {
            Ok(mut current) => {
                current
                    .update()
                    .display_name(profile.display_name.clone())
                    .introduction(profile.introduction.clone())
                    .allergies_json("[]".to_string())
                    .gender(profile.gender.clone())
                    .age(profile.age.map(i32::from))
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
                Ok(())
            }
            Err(err) if err.is_record_not_found() => {
                toasty::create!(UserProfileRow {
                    id: id,
                    display_name: profile.display_name.clone(),
                    introduction: profile.introduction.clone(),
                    allergies_json: "[]".to_string(),
                    gender: profile.gender.clone(),
                    age: profile.age.map(i32::from),
                })
                .exec(&mut *db)
                .await
                .map_err(|create_err| create_err.to_string())?;
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    pub async fn upsert_expectation(
        &self,
        expectation: &UserHealthExpectation,
    ) -> Result<(), String> {
        let id = expectation.id.as_str().to_string();
        let source_json =
            serde_json::to_string(&expectation.source).map_err(|err| err.to_string())?;
        let mut db = self.db.lock().await;
        match UserHealthExpectationRow::get_by_id(&mut *db, &id).await {
            Ok(mut current) => {
                current
                    .update()
                    .user_id(expectation.user_id.as_str().to_string())
                    .title(expectation.title.clone())
                    .summary(expectation.summary.clone())
                    .kind(expectation.kind.as_str().to_string())
                    .status(Self::expectation_status_to_str(&expectation.status).to_string())
                    .source_json(source_json)
                    .priority(i32::from(expectation.priority))
                    .created_at(expectation.created_at.clone())
                    .updated_at(expectation.updated_at.clone())
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
                Ok(())
            }
            Err(err) if err.is_record_not_found() => {
                toasty::create!(UserHealthExpectationRow {
                    id: id,
                    user_id: expectation.user_id.as_str().to_string(),
                    title: expectation.title.clone(),
                    summary: expectation.summary.clone(),
                    kind: expectation.kind.as_str().to_string(),
                    status: Self::expectation_status_to_str(&expectation.status).to_string(),
                    source_json: source_json,
                    priority: i32::from(expectation.priority),
                    created_at: expectation.created_at.clone(),
                    updated_at: expectation.updated_at.clone(),
                })
                .exec(&mut *db)
                .await
                .map_err(|create_err| create_err.to_string())?;
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    pub async fn upsert_preferences(&self, preferences: &UserPreferences) -> Result<(), String> {
        let user_id = preferences.user_id.as_str().to_string();
        let app_preferences_json =
            serde_json::to_string(&preferences.app).map_err(|err| err.to_string())?;
        let dietary_preferences_json =
            serde_json::to_string(&preferences.diet).map_err(|err| err.to_string())?;
        let updated_at = chrono::Utc::now().to_rfc3339();
        let mut db = self.db.lock().await;
        let mut rows =
            UserPreferencesRow::filter(UserPreferencesRow::fields().user_id().eq(&user_id))
                .exec(&mut *db)
                .await
                .map_err(|err| err.to_string())?;

        match rows.pop() {
            Some(mut current) => {
                current
                    .update()
                    .app_preferences_json(app_preferences_json)
                    .dietary_preferences_json(dietary_preferences_json)
                    .updated_at(updated_at)
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
                Ok(())
            }
            None => {
                toasty::create!(UserPreferencesRow {
                    user_id: user_id,
                    app_preferences_json: app_preferences_json,
                    dietary_preferences_json: dietary_preferences_json,
                    updated_at: updated_at,
                })
                .exec(&mut *db)
                .await
                .map_err(|create_err| create_err.to_string())?;
                Ok(())
            }
        }
    }

    pub async fn upsert_companion(&self, companion: &DiningCompanion) -> Result<(), String> {
        let id = companion.id.as_str().to_string();
        let dietary_preferences_json =
            serde_json::to_string(&companion.diet).map_err(|err| err.to_string())?;
        let health_notes_json =
            serde_json::to_string(&companion.health_notes).map_err(|err| err.to_string())?;
        let updated_at = chrono::Utc::now().to_rfc3339();
        let mut db = self.db.lock().await;

        match DiningCompanionRow::get_by_id(&mut *db, &id).await {
            Ok(mut current) => {
                current
                    .update()
                    .owner_user_id(companion.owner_user_id.as_str().to_string())
                    .display_name(companion.display_name.clone())
                    .relationship(companion.relationship.clone())
                    .introduction(companion.introduction.clone())
                    .dietary_preferences_json(dietary_preferences_json)
                    .health_notes_json(health_notes_json)
                    .updated_at(updated_at)
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
                Ok(())
            }
            Err(err) if err.is_record_not_found() => {
                toasty::create!(DiningCompanionRow {
                    id: id,
                    owner_user_id: companion.owner_user_id.as_str().to_string(),
                    display_name: companion.display_name.clone(),
                    relationship: companion.relationship.clone(),
                    introduction: companion.introduction.clone(),
                    dietary_preferences_json: dietary_preferences_json,
                    health_notes_json: health_notes_json,
                    updated_at: updated_at,
                })
                .exec(&mut *db)
                .await
                .map_err(|create_err| create_err.to_string())?;
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    fn row_to_ai_provider(row: UserAIProviderRow) -> Result<UserAIProvider, String> {
        Ok(UserAIProvider {
            id: row.id,
            user_id: UserId::parse(&row.user_id).map_err(|err| err.to_string())?,
            kind: AIProviderKind::parse(&row.kind)
                .ok_or_else(|| format!("unknown ai provider kind: {}", row.kind))?,
            name: row.name,
            base_url: row.base_url,
            secret_ref: row.secret_ref,
            enabled: row.enabled,
            updated_at: row.updated_at,
        })
    }

    fn row_to_ai_route(row: UserAIRouteRow) -> Result<UserAIRoute, String> {
        Ok(UserAIRoute {
            id: row.id,
            user_id: UserId::parse(&row.user_id).map_err(|err| err.to_string())?,
            capability: AICapability::parse(&row.capability)
                .ok_or_else(|| format!("unknown ai capability: {}", row.capability))?,
            provider_id: row.provider_id,
            model: row.model,
            embedding_ndims: row
                .embedding_ndims
                .map(|value| usize::try_from(value).unwrap_or_default()),
            enabled: row.enabled,
            updated_at: row.updated_at,
        })
    }

    fn row_to_profile(row: UserProfileRow) -> Result<UserProfile, String> {
        Ok(UserProfile {
            id: UserId::parse(&row.id).map_err(|err| err.to_string())?,
            display_name: row.display_name,
            introduction: row.introduction,
            gender: row.gender,
            age: row.age.map(|value| u8::try_from(value).unwrap_or_default()),
        })
    }

    fn row_to_expectation(row: UserHealthExpectationRow) -> Result<UserHealthExpectation, String> {
        Ok(UserHealthExpectation {
            id: HealthExpectationId::parse(&row.id).map_err(|err| err.to_string())?,
            user_id: UserId::parse(&row.user_id).map_err(|err| err.to_string())?,
            title: row.title,
            summary: row.summary,
            kind: Self::expectation_kind_from_str(&row.kind),
            status: Self::expectation_status_from_str(&row.status)?,
            source: serde_json::from_str::<ExpectationSource>(&row.source_json)
                .map_err(|err| err.to_string())?,
            priority: u8::try_from(row.priority).map_err(|err| err.to_string())?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    fn row_to_preferences(row: UserPreferencesRow) -> Result<UserPreferences, String> {
        Ok(UserPreferences {
            user_id: UserId::parse(&row.user_id).map_err(|err| err.to_string())?,
            app: serde_json::from_str::<AppPreferences>(&row.app_preferences_json)
                .map_err(|err| err.to_string())?,
            diet: serde_json::from_str::<DietaryPreferences>(&row.dietary_preferences_json)
                .map_err(|err| err.to_string())?,
        })
    }

    fn row_to_companion(row: DiningCompanionRow) -> Result<DiningCompanion, String> {
        Ok(DiningCompanion {
            id: DiningCompanionId::parse(&row.id).map_err(|err| err.to_string())?,
            owner_user_id: UserId::parse(&row.owner_user_id).map_err(|err| err.to_string())?,
            display_name: row.display_name,
            relationship: row.relationship,
            introduction: row.introduction,
            diet: serde_json::from_str::<DietaryPreferences>(&row.dietary_preferences_json)
                .map_err(|err| err.to_string())?,
            health_notes: serde_json::from_str::<Vec<String>>(&row.health_notes_json)
                .map_err(|err| err.to_string())?,
        })
    }

    fn expectation_status_to_str(status: &HealthExpectationStatus) -> &'static str {
        match status {
            HealthExpectationStatus::Proposed => "proposed",
            HealthExpectationStatus::Active => "active",
            HealthExpectationStatus::Archived => "archived",
        }
    }

    fn expectation_status_from_str(value: &str) -> Result<HealthExpectationStatus, String> {
        match value {
            "proposed" => Ok(HealthExpectationStatus::Proposed),
            "active" => Ok(HealthExpectationStatus::Active),
            "archived" => Ok(HealthExpectationStatus::Archived),
            _ => Err(format!("unknown health expectation status: {value}")),
        }
    }

    fn expectation_kind_from_str(value: &str) -> HealthExpectationKind {
        match value {
            "weight_loss" => HealthExpectationKind::WeightLoss,
            "energy_boost" => HealthExpectationKind::EnergyBoost,
            "better_sleep" => HealthExpectationKind::BetterSleep,
            "blood_sugar_control" => HealthExpectationKind::BloodSugarControl,
            other => HealthExpectationKind::Custom(other.to_string()),
        }
    }
}

#[async_trait]
impl UserProfileRepositoryPort for SqliteUserRepo {
    async fn find_profile(&self, user_id: &UserId) -> Result<Option<UserProfile>, String> {
        let mut db = self.db.lock().await;
        let id = user_id.as_str().to_string();
        match UserProfileRow::get_by_id(&mut *db, &id).await {
            Ok(row) => Self::row_to_profile(row).map(Some),
            Err(err) if err.is_record_not_found() => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }

    async fn list_profiles(&self) -> Result<Vec<UserProfile>, String> {
        let mut db = self.db.lock().await;
        let rows = UserProfileRow::filter(UserProfileRow::fields().id().ne(""))
            .exec(&mut *db)
            .await
            .map_err(|err| err.to_string())?;

        let mut profiles = rows
            .into_iter()
            .map(Self::row_to_profile)
            .collect::<Result<Vec<_>, _>>()?;
        profiles.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        Ok(profiles)
    }

    async fn save_profile(&self, profile: &UserProfile) -> Result<(), String> {
        self.upsert_profile(profile).await
    }
}

#[async_trait]
impl UserHealthExpectationRepositoryPort for SqliteUserRepo {
    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<UserHealthExpectation>, String> {
        let mut db = self.db.lock().await;
        let rows = UserHealthExpectationRow::filter(
            UserHealthExpectationRow::fields()
                .user_id()
                .eq(user_id.as_str()),
        )
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;

        rows.into_iter()
            .map(Self::row_to_expectation)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn list_active_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<UserHealthExpectation>, String> {
        let expectations = self.list_by_user(user_id).await?;
        Ok(expectations
            .into_iter()
            .filter(|item| item.status == HealthExpectationStatus::Active)
            .collect())
    }

    async fn save_expectation(&self, expectation: &UserHealthExpectation) -> Result<(), String> {
        self.upsert_expectation(expectation).await
    }

    async fn delete_expectation(
        &self,
        user_id: &UserId,
        expectation_id: &HealthExpectationId,
    ) -> Result<(), String> {
        let mut db = self.db.lock().await;
        let row = UserHealthExpectationRow::get_by_id(&mut *db, expectation_id.as_str())
            .await
            .map_err(|err| err.to_string())?;

        if row.user_id != user_id.as_str() {
            return Err("health expectation does not belong to user".to_string());
        }

        row.delete()
            .exec(&mut *db)
            .await
            .map_err(|err| err.to_string())
    }
}

#[async_trait]
impl UserPreferencesRepositoryPort for SqliteUserRepo {
    async fn find_preferences(&self, user_id: &UserId) -> Result<Option<UserPreferences>, String> {
        let mut db = self.db.lock().await;
        let rows =
            UserPreferencesRow::filter(UserPreferencesRow::fields().user_id().eq(user_id.as_str()))
                .exec(&mut *db)
                .await
                .map_err(|err| err.to_string())?;

        rows.into_iter()
            .next()
            .map(Self::row_to_preferences)
            .transpose()
    }

    async fn save_preferences(&self, preferences: &UserPreferences) -> Result<(), String> {
        self.upsert_preferences(preferences).await
    }
}

#[async_trait]
impl DiningCompanionRepositoryPort for SqliteUserRepo {
    async fn list_companions(
        &self,
        owner_user_id: &UserId,
    ) -> Result<Vec<DiningCompanion>, String> {
        let mut db = self.db.lock().await;
        let rows = DiningCompanionRow::filter(
            DiningCompanionRow::fields()
                .owner_user_id()
                .eq(owner_user_id.as_str()),
        )
        .exec(&mut *db)
        .await
        .map_err(|err| err.to_string())?;

        let mut companions = rows
            .into_iter()
            .map(Self::row_to_companion)
            .collect::<Result<Vec<_>, _>>()?;
        companions.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        Ok(companions)
    }

    async fn save_companion(&self, companion: &DiningCompanion) -> Result<(), String> {
        self.upsert_companion(companion).await
    }

    async fn delete_companion(
        &self,
        owner_user_id: &UserId,
        companion_id: &DiningCompanionId,
    ) -> Result<(), String> {
        let mut db = self.db.lock().await;
        let row = DiningCompanionRow::get_by_id(&mut *db, companion_id.as_str())
            .await
            .map_err(|err| err.to_string())?;

        if row.owner_user_id != owner_user_id.as_str() {
            return Err("dining companion does not belong to owner user".to_string());
        }

        row.delete()
            .exec(&mut *db)
            .await
            .map_err(|err| err.to_string())
    }
}

#[async_trait]
impl UserAIConfigRepositoryPort for SqliteUserRepo {
    async fn list_providers(&self, user_id: &UserId) -> Result<Vec<UserAIProvider>, String> {
        let mut db = self.db.lock().await;
        let rows =
            UserAIProviderRow::filter(UserAIProviderRow::fields().user_id().eq(user_id.as_str()))
                .exec(&mut *db)
                .await
                .map_err(|err| err.to_string())?;

        let mut providers = rows
            .into_iter()
            .map(Self::row_to_ai_provider)
            .collect::<Result<Vec<_>, _>>()?;
        providers.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(providers)
    }

    async fn save_provider(&self, provider: &UserAIProvider) -> Result<(), String> {
        let mut db = self.db.lock().await;
        match UserAIProviderRow::get_by_id(&mut *db, &provider.id).await {
            Ok(mut current) => {
                current
                    .update()
                    .user_id(provider.user_id.as_str().to_string())
                    .kind(provider.kind.as_str().to_string())
                    .name(provider.name.clone())
                    .base_url(provider.base_url.clone())
                    .secret_ref(provider.secret_ref.clone())
                    .enabled(provider.enabled)
                    .updated_at(provider.updated_at.clone())
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
            }
            Err(err) if err.is_record_not_found() => {
                toasty::create!(UserAIProviderRow {
                    id: provider.id.clone(),
                    user_id: provider.user_id.as_str().to_string(),
                    kind: provider.kind.as_str().to_string(),
                    name: provider.name.clone(),
                    base_url: provider.base_url.clone(),
                    secret_ref: provider.secret_ref.clone(),
                    enabled: provider.enabled,
                    updated_at: provider.updated_at.clone(),
                })
                .exec(&mut *db)
                .await
                .map_err(|create_err| create_err.to_string())?;
            }
            Err(err) => return Err(err.to_string()),
        }
        Ok(())
    }

    async fn delete_provider(&self, user_id: &UserId, provider_id: &str) -> Result<(), String> {
        let mut db = self.db.lock().await;
        let row = UserAIProviderRow::get_by_id(&mut *db, provider_id)
            .await
            .map_err(|err| err.to_string())?;
        if row.user_id != user_id.as_str() {
            return Err("ai provider does not belong to user".to_string());
        }
        row.delete()
            .exec(&mut *db)
            .await
            .map_err(|err| err.to_string())
    }

    async fn list_routes(&self, user_id: &UserId) -> Result<Vec<UserAIRoute>, String> {
        let mut db = self.db.lock().await;
        let rows = UserAIRouteRow::filter(UserAIRouteRow::fields().user_id().eq(user_id.as_str()))
            .exec(&mut *db)
            .await
            .map_err(|err| err.to_string())?;

        rows.into_iter()
            .map(Self::row_to_ai_route)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn find_route(
        &self,
        user_id: &UserId,
        capability: AICapability,
    ) -> Result<Option<UserAIRoute>, String> {
        let routes = self.list_routes(user_id).await?;
        Ok(routes
            .into_iter()
            .filter(|route| route.capability == capability && route.enabled)
            .max_by(|left, right| left.updated_at.cmp(&right.updated_at)))
    }

    async fn save_route(&self, route: &UserAIRoute) -> Result<(), String> {
        let embedding_ndims = route
            .embedding_ndims
            .map(|value| i32::try_from(value).unwrap_or_default());
        let mut db = self.db.lock().await;
        match UserAIRouteRow::get_by_id(&mut *db, &route.id).await {
            Ok(mut current) => {
                current
                    .update()
                    .user_id(route.user_id.as_str().to_string())
                    .capability(route.capability.as_str().to_string())
                    .provider_id(route.provider_id.clone())
                    .model(route.model.clone())
                    .embedding_ndims(embedding_ndims)
                    .enabled(route.enabled)
                    .updated_at(route.updated_at.clone())
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
            }
            Err(err) if err.is_record_not_found() => {
                toasty::create!(UserAIRouteRow {
                    id: route.id.clone(),
                    user_id: route.user_id.as_str().to_string(),
                    capability: route.capability.as_str().to_string(),
                    provider_id: route.provider_id.clone(),
                    model: route.model.clone(),
                    embedding_ndims: embedding_ndims,
                    enabled: route.enabled,
                    updated_at: route.updated_at.clone(),
                })
                .exec(&mut *db)
                .await
                .map_err(|create_err| create_err.to_string())?;
            }
            Err(err) => return Err(err.to_string()),
        }
        Ok(())
    }
}

#[async_trait]
impl SecretStorePort for SqliteUserRepo {
    async fn put_secret(&self, scope: &str, value: &str) -> Result<String, String> {
        let id = format!("secret:{}", scope);
        let updated_at = chrono::Utc::now().to_rfc3339();
        let encrypted_value = self.secret_cipher.encrypt(value)?;
        let mut db = self.db.lock().await;
        match UserSecretRow::get_by_id(&mut *db, &id).await {
            Ok(mut current) => {
                current
                    .update()
                    .scope(scope.to_string())
                    .secret_value(encrypted_value.clone())
                    .updated_at(updated_at)
                    .exec(&mut *db)
                    .await
                    .map_err(|err| err.to_string())?;
            }
            Err(err) if err.is_record_not_found() => {
                toasty::create!(UserSecretRow {
                    id: id.clone(),
                    scope: scope.to_string(),
                    secret_value: encrypted_value,
                    updated_at: updated_at,
                })
                .exec(&mut *db)
                .await
                .map_err(|create_err| create_err.to_string())?;
            }
            Err(err) => return Err(err.to_string()),
        }
        Ok(id)
    }

    async fn get_secret(&self, secret_ref: &str) -> Result<Option<String>, String> {
        let mut db = self.db.lock().await;
        match UserSecretRow::get_by_id(&mut *db, secret_ref).await {
            Ok(row) => self.secret_cipher.decrypt(&row.secret_value).map(Some),
            Err(err) if err.is_record_not_found() => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }

    async fn delete_secret(&self, secret_ref: &str) -> Result<(), String> {
        let mut db = self.db.lock().await;
        match UserSecretRow::get_by_id(&mut *db, secret_ref).await {
            Ok(row) => row
                .delete()
                .exec(&mut *db)
                .await
                .map_err(|err| err.to_string()),
            Err(err) if err.is_record_not_found() => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }
}
