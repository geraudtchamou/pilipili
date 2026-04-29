use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::User;
use crate::utils::error::{AppError, AppResult};
use crate::services::agent_service;

#[derive(Debug, Deserialize)]
pub struct NearbyAgentsQuery {
    pub lat: f64,
    pub lng: f64,
    pub radius_km: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct AgentResponse {
    pub id: Uuid,
    pub business_name: Option<String>,
    pub distance_km: Option<f64>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CashInRequest {
    pub agent_id: Uuid,
    pub amount: i64,
    pub pin: String,
}

#[derive(Debug, Deserialize)]
pub struct CashOutRequest {
    pub agent_id: Uuid,
    pub amount: i64,
    pub pin: String,
}

pub async fn find_nearby_agents(
    pool: web::Data<PgPool>,
    query: web::Query<NearbyAgentsQuery>,
) -> AppResult<HttpResponse> {
    let agents = agent_service::find_nearby_agents(
        &pool,
        query.lat,
        query.lng,
        query.radius_km.unwrap_or(5.0),
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "agents": agents,
        "count": agents.len()
    })))
}

pub async fn record_cash_in(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
    body: web::Json<CashInRequest>,
) -> AppResult<HttpResponse> {
    let transaction = agent_service::process_cash_in(
        &pool,
        user.id,
        body.agent_id,
        body.amount,
        &body.pin,
    )
    .await?;

    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "transaction_id": transaction.id,
        "status": transaction.status,
        "message": "Cash-in processing"
    })))
}

pub async fn record_cash_out(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
    body: web::Json<CashOutRequest>,
) -> AppResult<HttpResponse> {
    let transaction = agent_service::process_cash_out(
        &pool,
        user.id,
        body.agent_id,
        body.amount,
        &body.pin,
    )
    .await?;

    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "transaction_id": transaction.id,
        "status": transaction.status,
        "message": "Cash-out processing"
    })))
}
