use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::User;
use crate::utils::error::{AppError, AppResult};
use crate::services::sync_service;

#[derive(Debug, Deserialize)]
pub struct SyncPushRequest {
    pub last_sync_timestamp: DateTime<Utc>,
    pub pending_operations: Vec<PendingOperation>,
}

#[derive(Debug, Deserialize)]
pub struct PendingOperation {
    pub op_id: Uuid,
    #[serde(rename = "type")]
    pub operation_type: String,
    pub entity: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub results: Vec<SyncResult>,
    pub server_updates: serde_json::Value,
    pub new_last_sync: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub offline_id: Uuid,
    pub status: String,
    pub server_tx_id: Option<Uuid>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SyncPullQuery {
    pub since: Option<DateTime<Utc>>,
}

pub async fn push_sync(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
    body: web::Json<SyncPushRequest>,
) -> AppResult<HttpResponse> {
    let results = sync_service::process_pending_operations(
        &pool,
        user.id,
        body.pending_operations.clone(),
    )
    .await?;

    let server_updates = sync_service::get_server_changes(&pool, user.id, body.last_sync_timestamp)
        .await?;

    Ok(HttpResponse::Ok().json(SyncResponse {
        results,
        server_updates,
        new_last_sync: Utc::now(),
    }))
}

pub async fn pull_sync(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
    query: web::Query<SyncPullQuery>,
) -> AppResult<HttpResponse> {
    let since = query.since.unwrap_or_else(|| Utc::now() - chrono::Duration::hours(24));
    let updates = sync_service::get_server_changes(&pool, user.id, since).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "updates": updates,
        "synced_at": Utc::now()
    })))
}
