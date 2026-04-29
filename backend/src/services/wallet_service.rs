use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use crate::models::{Wallet, Transaction};
use crate::utils::{AppError, AppResult};

pub async fn get_user_wallet(pool: &PgPool, user_id: Uuid) -> AppResult<Wallet> {
    let wallet = sqlx::query_as::<_, Wallet>(
        "SELECT * FROM wallets WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Wallet not found".to_string()))?;

    Ok(wallet)
}

pub async fn get_or_create_wallet(pool: &PgPool, user_id: Uuid) -> AppResult<Wallet> {
    let wallet = sqlx::query_as::<_, Wallet>(
        r#"
        INSERT INTO wallets (user_id, balance, currency, status)
        VALUES ($1, 0, 'UGX', 'active')
        ON CONFLICT (user_id) DO UPDATE SET updated_at = NOW()
        RETURNING *
        "#
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(wallet)
}

pub async fn request_topup(
    pool: &PgPool,
    user_id: Uuid,
    amount: i64,
    provider: &str,
    phone: &str,
) -> AppResult<Transaction> {
    // Create topup transaction record
    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (user_id, type, amount, currency, status, metadata)
        VALUES ($1, 'topup', $2, 'UGX', 'pending_provider', $3)
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(amount)
    .bind(serde_json::json!({
        "provider": provider,
        "phone": phone
    }))
    .fetch_one(pool)
    .await?;

    // In production, trigger USSD push to mobile money provider
    
    Ok(transaction)
}

pub async fn request_withdraw(
    pool: &PgPool,
    user_id: Uuid,
    agent_id: Uuid,
    amount: i64,
) -> AppResult<Transaction> {
    // Verify user has sufficient balance
    let wallet = get_user_wallet(pool, user_id).await?;
    
    if wallet.balance < amount {
        return Err(AppError::InsufficientFunds);
    }

    // Create withdrawal transaction
    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (user_id, type, amount, currency, status, metadata)
        VALUES ($1, 'withdrawal', $2, 'UGX', 'pending', $3)
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(amount)
    .bind(serde_json::json!({
        "agent_id": agent_id
    }))
    .fetch_one(pool)
    .await?;

    Ok(transaction)
}

pub async fn get_transaction_history(
    pool: &PgPool,
    user_id: Uuid,
    limit: u32,
    offset: u32,
) -> AppResult<Vec<Transaction>> {
    let transactions = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT * FROM transactions 
        WHERE user_id = $1 
        ORDER BY created_at DESC 
        LIMIT $2 OFFSET $3
        "#
    )
    .bind(user_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await?;

    Ok(transactions)
}

pub async fn update_balance(
    pool: &PgPool,
    wallet_id: Uuid,
    amount: i64,
) -> AppResult<Wallet> {
    let wallet = sqlx::query_as::<_, Wallet>(
        r#"
        UPDATE wallets 
        SET balance = balance + $1, version = version + 1, updated_at = NOW()
        WHERE id = $2
        RETURNING *
        "#
    )
    .bind(amount)
    .bind(wallet_id)
    .fetch_one(pool)
    .await?;

    Ok(wallet)
}
