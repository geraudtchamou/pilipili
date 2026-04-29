use sqlx::PgPool;
use uuid::Uuid;
use crate::models::User;
use crate::utils::{AppError, AppResult, hash_pin, verify_pin, generate_otp};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub async fn register_user(
    pool: &PgPool,
    phone: &str,
    country_code: Option<String>,
) -> AppResult<User> {
    // Check if user already exists
    let existing = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE phone = $1"
    )
    .bind(phone)
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::Validation("User with this phone already exists".to_string()));
    }

    // Generate OTP (in production, send via SMS)
    let otp = generate_otp();
    tracing::info!("OTP for {}: {}", phone, otp); // TODO: Remove in production

    // Create user
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (phone, country_code, phone_verified)
        VALUES ($1, $2, false)
        RETURNING *
        "#
    )
    .bind(phone)
    .bind(country_code.unwrap_or_else(|| "UG".to_string()))
    .fetch_one(pool)
    .await?;

    // Store OTP in Redis (TODO: implement)
    
    Ok(user)
}

pub async fn verify_otp(
    pool: &PgPool,
    phone: &str,
    otp_code: &str,
) -> AppResult<()> {
    // In production, verify OTP from Redis
    // For now, accept any 6-digit code
    
    if otp_code.len() != 6 {
        return Err(AppError::Validation("Invalid OTP code".to_string()));
    }

    sqlx::query(
        "UPDATE users SET phone_verified = true WHERE phone = $1"
    )
    .bind(phone)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn authenticate(
    pool: &PgPool,
    phone: &str,
    pin: &str,
) -> AppResult<(User, String)> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE phone = $1 AND is_active = true"
    )
    .bind(phone)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Auth("User not found".to_string()))?;

    if !user.phone_verified {
        return Err(AppError::Auth("Phone not verified".to_string()));
    }

    let pin_hash = user.pin_hash.as_ref()
        .ok_or_else(|| AppError::Auth("PIN not set".to_string()))?;

    if !verify_pin(pin, pin_hash)? {
        return Err(AppError::InvalidPIN);
    }

    // Generate JWT token
    let claims = Claims {
        sub: user.id.to_string(),
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;

    Ok((user, token))
}

pub async fn update_user_profile(
    pool: &PgPool,
    user_id: Uuid,
    first_name: Option<String>,
    last_name: Option<String>,
) -> AppResult<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users 
        SET first_name = COALESCE($1, first_name),
            last_name = COALESCE($2, last_name),
            updated_at = NOW()
        WHERE id = $3
        RETURNING *
        "#
    )
    .bind(first_name)
    .bind(last_name)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn set_user_pin(
    pool: &PgPool,
    user_id: Uuid,
    pin: &str,
) -> AppResult<()> {
    if pin.len() < 4 {
        return Err(AppError::Validation("PIN must be at least 4 digits".to_string()));
    }

    let pin_hash = hash_pin(pin)
        .map_err(|e| AppError::Internal(format!("PIN hashing failed: {}", e)))?;

    sqlx::query(
        "UPDATE users SET pin_hash = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(pin_hash)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}
