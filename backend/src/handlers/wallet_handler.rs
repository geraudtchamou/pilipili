use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{User, Wallet};
use crate::utils::error::{AppError, AppResult};
use crate::services::wallet_service;

#[derive(Debug, Serialize)]
pub struct WalletResponse {
    pub id: Uuid,
    pub balance: i64,
    pub currency: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct TopupRequest {
    pub amount: i64,
    pub mobile_money_provider: String,
    pub phone_number: String,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    pub amount: i64,
    pub agent_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct TopupResponse {
    pub topup_id: Uuid,
    pub status: String,
    pub ussd_code: Option<String>,
    pub expires_at: String,
}

pub async fn get_wallet(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
) -> AppResult<HttpResponse> {
    let wallet = wallet_service::get_user_wallet(&pool, user.id).await?;

    Ok(HttpResponse::Ok().json(WalletResponse {
        id: wallet.id,
        balance: wallet.balance,
        currency: wallet.currency,
        status: wallet.status,
    }))
}

pub async fn request_topup(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
    body: web::Json<TopupRequest>,
) -> AppResult<HttpResponse> {
    let topup = wallet_service::request_topup(
        &pool,
        user.id,
        body.amount,
        &body.mobile_money_provider,
        &body.phone_number,
    )
    .await?;

    Ok(HttpResponse::Accepted().json(TopupResponse {
        topup_id: topup.id,
        status: topup.status,
        ussd_code: topup.ussd_code,
        expires_at: topup.expires_at.to_rfc3339(),
    }))
}

pub async fn request_withdraw(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
    body: web::Json<WithdrawRequest>,
) -> AppResult<HttpResponse> {
    let transaction = wallet_service::request_withdraw(
        &pool,
        user.id,
        body.agent_id,
        body.amount,
    )
    .await?;

    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "transaction_id": transaction.id,
        "status": transaction.status,
        "message": "Withdrawal request queued"
    })))
}

pub async fn get_wallet_history(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
    query: web::Json<HistoryQuery>,
) -> AppResult<HttpResponse> {
    let transactions = wallet_service::get_transaction_history(
        &pool,
        user.id,
        query.limit.unwrap_or(20),
        query.offset.unwrap_or(0),
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "transactions": transactions,
        "total": transactions.len()
    })))
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}
