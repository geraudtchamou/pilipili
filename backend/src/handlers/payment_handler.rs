use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{User, Transaction};
use crate::utils::error::{AppError, AppResult};
use crate::services::payment_service;

#[derive(Debug, Deserialize)]
pub struct P2PPaymentRequest {
    pub recipient_phone: String,
    pub amount: i64,
    pub currency: Option<String>,
    pub pin: String,
    pub description: Option<String>,
    pub offline_tx_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub transaction_id: Uuid,
    pub status: String,
    pub balance_after: Option<i64>,
    pub receipt_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReversePaymentRequest {
    pub transaction_id: Uuid,
    pub reason: String,
}

pub async fn p2p_payment(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
    body: web::Json<P2PPaymentRequest>,
) -> AppResult<HttpResponse> {
    let transaction = payment_service::process_p2p_payment(
        &pool,
        user.id,
        &body.recipient_phone,
        body.amount,
        &body.pin,
        body.description.clone(),
        body.offline_tx_id.clone(),
    )
    .await?;

    let response = PaymentResponse {
        transaction_id: transaction.id,
        status: transaction.status,
        balance_after: None, // Could be fetched from wallet
        receipt_url: Some(format!("/api/v1/receipts/{}", transaction.id)),
    };

    if transaction.status == "pending" {
        Ok(HttpResponse::Accepted().json(response))
    } else {
        Ok(HttpResponse::Ok().json(response))
    }
}

pub async fn get_payment_status(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let transaction_id = path.into_inner();
    let transaction = payment_service::get_transaction(&pool, transaction_id, user.id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "transaction_id": transaction.id,
        "status": transaction.status,
        "amount": transaction.amount,
        "type": transaction.transaction_type,
        "created_at": transaction.created_at,
        "processed_at": transaction.processed_at,
    })))
}

pub async fn reverse_payment(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
    body: web::Json<ReversePaymentRequest>,
) -> AppResult<HttpResponse> {
    let transaction = payment_service::reverse_transaction(
        &pool,
        body.transaction_id,
        user.id,
        &body.reason,
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "transaction_id": transaction.id,
        "status": transaction.status,
        "message": "Payment reversed successfully"
    })))
}
