use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::User;
use crate::utils::error::{AppError, AppResult};
use crate::services::bill_service;

#[derive(Debug, Serialize)]
pub struct BillProviderResponse {
    pub id: Uuid,
    pub name: String,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateBillRequest {
    pub provider_id: Uuid,
    pub account_number: String,
}

#[derive(Debug, Deserialize)]
pub struct PayBillRequest {
    pub provider_id: Uuid,
    pub account_number: String,
    pub amount: i64,
    pub pin: String,
    pub description: Option<String>,
}

pub async fn get_bill_providers(
    pool: web::Data<PgPool>,
) -> AppResult<HttpResponse> {
    let providers = bill_service::get_active_providers(&pool).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "providers": providers
    })))
}

pub async fn validate_bill_account(
    pool: web::Data<PgPool>,
    query: web::Json<ValidateBillRequest>,
) -> AppResult<HttpResponse> {
    let result = bill_service::validate_bill_account(&pool, query.provider_id, &query.account_number)
        .await?;

    Ok(HttpResponse::Ok().json(result))
}

pub async fn pay_bill(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
    body: web::Json<PayBillRequest>,
) -> AppResult<HttpResponse> {
    let transaction = bill_service::process_bill_payment(
        &pool,
        user.id,
        body.provider_id,
        &body.account_number,
        body.amount,
        &body.pin,
        body.description.clone(),
    )
    .await?;

    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "transaction_id": transaction.id,
        "status": transaction.status,
        "message": "Bill payment processing"
    })))
}

pub async fn get_bill_history(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
) -> AppResult<HttpResponse> {
    let transactions = bill_service::get_bill_history(&pool, user.id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "transactions": transactions
    })))
}
