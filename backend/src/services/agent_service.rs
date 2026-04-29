use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Transaction, Wallet};
use crate::utils::{AppError, AppResult, verify_pin};
use super::wallet_service::get_user_wallet;

pub async fn find_nearby_agents(
    pool: &PgPool,
    lat: f64,
    lng: f64,
    radius_km: f64,
) -> AppResult<Vec<serde_json::Value>> {
    // Using Haversine formula for distance calculation
    let agents = sqlx::query(
        r#"
        SELECT 
            a.id,
            a.business_name,
            a.is_active,
            u.phone,
            ST_DistanceSphere(
                ST_MakePoint(a.longitude, a.latitude),
                ST_MakePoint($1, $2)
            ) / 1000.0 as distance_km
        FROM agents a
        JOIN users u ON a.user_id = u.id
        WHERE a.is_active = true
        AND ST_DWithin(
            ST_MakePoint(a.longitude, a.latitude)::geography,
            ST_MakePoint($1, $2)::geography,
            $3 * 1000.0
        )
        ORDER BY distance_km
        LIMIT 20
        "#
    )
    .bind(lng)
    .bind(lat)
    .bind(radius_km)
    .fetch_all(pool)
    .await?;

    let result: Vec<serde_json::Value> = agents
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<Uuid, _>("id"),
                "business_name": row.get::<Option<String>, _>("business_name"),
                "phone": row.get::<String, _>("phone"),
                "distance_km": row.get::<f64, _>("distance_km"),
                "is_active": row.get::<bool, _>("is_active")
            })
        })
        .collect();

    Ok(result)
}

pub async fn process_cash_in(
    pool: &PgPool,
    user_id: Uuid,
    agent_id: Uuid,
    amount: i64,
    pin: &str,
) -> AppResult<Transaction> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    // Verify PIN
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let pin_hash = user.pin_hash.as_ref()
        .ok_or_else(|| AppError::Auth("PIN not set".to_string()))?;

    if !verify_pin(pin, pin_hash)? {
        return Err(AppError::InvalidPIN);
    }

    // Get agent details
    let agent = sqlx::query(
        "SELECT * FROM agents WHERE id = $1 AND is_active = true"
    )
    .bind(agent_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Agent not found".to_string()))?;

    // Get or create user wallet
    let wallet = sqlx::query_as::<_, Wallet>(
        "SELECT * FROM wallets WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    let wallet = match wallet {
        Some(w) => w,
        None => {
            sqlx::query_as::<_, Wallet>(
                "INSERT INTO wallets (user_id, balance, currency, status) VALUES ($1, 0, 'UGX', 'active') RETURNING *"
            )
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?
        }
    };

    // Create cash-in transaction
    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (user_id, type, amount, currency, status, metadata)
        VALUES ($1, 'cash_in', $2, 'UGX', 'pending', $3)
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(amount)
    .bind(serde_json::json!({
        "agent_id": agent_id
    }))
    .fetch_one(&mut *tx)
    .await?;

    // Credit user wallet
    sqlx::query(
        "UPDATE wallets SET balance = balance + $1, version = version + 1 WHERE id = $2"
    )
    .bind(amount)
    .bind(wallet.id)
    .execute(&mut *tx)
    .await?;

    // Update agent float balance
    sqlx::query(
        "UPDATE agents SET float_balance = float_balance + $1 WHERE id = $2"
    )
    .bind(amount)
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    // Mark transaction as completed
    let completed_transaction = sqlx::query_as::<_, Transaction>(
        "UPDATE transactions SET status = 'completed', processed_at = NOW() WHERE id = $1 RETURNING *"
    )
    .bind(transaction.id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await.map_err(AppError::Database)?;

    Ok(completed_transaction)
}

pub async fn process_cash_out(
    pool: &PgPool,
    user_id: Uuid,
    agent_id: Uuid,
    amount: i64,
    pin: &str,
) -> AppResult<Transaction> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    // Verify PIN
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    let pin_hash = user.pin_hash.as_ref()
        .ok_or_else(|| AppError::Auth("PIN not set".to_string()))?;

    if !verify_pin(pin, pin_hash)? {
        return Err(AppError::InvalidPIN);
    }

    // Get user wallet and verify balance
    let wallet = get_user_wallet(pool, user_id).await?;
    
    if wallet.balance < amount {
        return Err(AppError::InsufficientFunds);
    }

    // Get agent details
    let agent = sqlx::query(
        "SELECT * FROM agents WHERE id = $1 AND is_active = true"
    )
    .bind(agent_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Agent not found".to_string()))?;

    // Create cash-out transaction
    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (user_id, type, amount, currency, status, metadata)
        VALUES ($1, 'cash_out', $2, 'UGX', 'pending', $3)
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(amount)
    .bind(serde_json::json!({
        "agent_id": agent_id
    }))
    .fetch_one(&mut *tx)
    .await?;

    // Debit user wallet
    sqlx::query(
        "UPDATE wallets SET balance = balance - $1, version = version + 1 WHERE id = $2"
    )
    .bind(amount)
    .bind(wallet.id)
    .execute(&mut *tx)
    .await?;

    // Update agent float balance
    sqlx::query(
        "UPDATE agents SET float_balance = float_balance - $1 WHERE id = $2"
    )
    .bind(amount)
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    // Mark transaction as completed
    let completed_transaction = sqlx::query_as::<_, Transaction>(
        "UPDATE transactions SET status = 'completed', processed_at = NOW() WHERE id = $1 RETURNING *"
    )
    .bind(transaction.id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await.map_err(AppError::Database)?;

    Ok(completed_transaction)
}