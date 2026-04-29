use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::User;
use crate::utils::error::{AppError, AppResult};
use crate::services::auth_service;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub phone: String,
    pub country_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyOtpRequest {
    pub phone: String,
    pub otp_code: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub phone: String,
    pub pin: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub phone: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub kyc_level: i32,
    pub is_verified: bool,
}

pub async fn register(
    pool: web::Data<PgPool>,
    body: web::Json<RegisterRequest>,
) -> AppResult<HttpResponse> {
    let user = auth_service::register_user(&pool, &body.phone, body.country_code.clone())
        .await?;

    Ok(HttpResponse::Created().json(web::Json(UserProfileResponse {
        id: user.id,
        phone: user.phone,
        first_name: user.first_name,
        last_name: user.last_name,
        kyc_level: user.kyc_level,
        is_verified: user.phone_verified,
    })))
}

pub async fn verify_otp(
    pool: web::Data<PgPool>,
    body: web::Json<VerifyOtpRequest>,
) -> AppResult<HttpResponse> {
    auth_service::verify_otp(&pool, &body.phone, &body.otp_code)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Phone verified successfully"
    })))
}

pub async fn login(
    pool: web::Data<PgPool>,
    body: web::Json<LoginRequest>,
) -> AppResult<HttpResponse> {
    let (user, token) = auth_service::authenticate(&pool, &body.phone, &body.pin)
        .await?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        user_id: user.id,
        token,
        expires_at: chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .unwrap()
            .to_rfc3339(),
    }))
}

pub async fn get_profile(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
) -> AppResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(UserProfileResponse {
        id: user.id,
        phone: user.phone.clone(),
        first_name: user.first_name.clone(),
        last_name: user.last_name.clone(),
        kyc_level: user.kyc_level,
        is_verified: user.phone_verified,
    }))
}

pub async fn update_profile(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
    body: web::Json<UpdateProfileRequest>,
) -> AppResult<HttpResponse> {
    let updated_user = auth_service::update_user_profile(
        &pool,
        user.id,
        body.first_name.clone(),
        body.last_name.clone(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(UserProfileResponse {
        id: updated_user.id,
        phone: updated_user.phone,
        first_name: updated_user.first_name,
        last_name: updated_user.last_name,
        kyc_level: updated_user.kyc_level,
        is_verified: updated_user.phone_verified,
    }))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

pub async fn set_pin(
    pool: web::Data<PgPool>,
    user: web::ReqData<User>,
    body: web::Json<SetPinRequest>,
) -> AppResult<HttpResponse> {
    auth_service::set_user_pin(&pool, user.id, &body.pin)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "PIN set successfully"
    })))
}

#[derive(Debug, Deserialize)]
pub struct SetPinRequest {
    pub pin: String,
    pub current_pin: Option<String>,
}
