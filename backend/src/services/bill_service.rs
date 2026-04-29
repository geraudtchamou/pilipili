use sqlx::PgPool;
use uuid::Uuid;
use crate::models::Transaction;
use crate::utils::{AppError, AppResult};

pub async fn get_active_providers(pool: &PgPool) -> AppResult<Vec<serde_json::Value>> {
    let providers = sqlx::query(
        "SELECT id, name, category FROM bill_providers WHERE is_active = true"
    )
    .fetch_all(pool)
    .await?;

    let result: Vec<serde_json::Value> = providers
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<Uuid, _>("id"),
                "name": row.get::<String, _>("name"),
                "category": row.get::<Option<String>, _>("category")
            })
        })
        .collect();

    Ok(result)
}

pub async fn validate_bill_account(
    pool: &PgPool,
    provider_id: Uuid,
    account_number: &str,
) -> AppResult<serde_json::Value> {
    // In production, call the provider's API to validate
    // For now, return a mock response
    
    Ok(serde_json::json!({
        "valid": true,
        "account_name": "Mock Account Name",
        "account_number": account_number,
        "provider_id": provider_id
    }))
}

pub async fn process_bill_payment(
    pool: &PgPool,
    user_id: Uuid,
    provider_id: Uuid,
    account_number: &str,
    amount: i64,
    pin: &str,
    description: Option<String>,
) -> AppResult<Transaction> {
    // Verify user has sufficient balance (similar to P2P)
    let wallet = sqlx::query_as::<_, crate::models::Wallet>(
        "SELECT * FROM wallets WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Wallet not found".to_string()))?;

    if wallet.balance < amount {
        return Err(AppError::InsufficientFunds);
    }

    // Create bill payment transaction
    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (user_id, type, amount, currency, status, metadata)
        VALUES ($1, 'bill_payment', $2, 'UGX', 'pending_provider', $3)
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(amount)
    .bind(serde_json::json!({
        "provider_id": provider_id,
        "account_number": account_number,
        "description": description
    }))
    .fetch_one(pool)
    .await?;

    // Debit user immediately (could also wait for provider confirmation)
    sqlx::query(
        "UPDATE wallets SET balance = balance - $1, version = version + 1 WHERE id = $2"
    )
    .bind(amount)
    .bind(wallet.id)
    .execute(pool)
    .await?;

    // In production, call provider API here
    
    Ok(transaction)
}

pub async fn get_bill_history(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Transaction>> {
    let transactions = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT * FROM transactions 
        WHERE user_id = $1 AND type = 'bill_payment'
        ORDER BY created_at DESC
        LIMIT 50
        "#
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(transactions)
}
