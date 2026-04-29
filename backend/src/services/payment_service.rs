use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Transaction, Wallet};
use crate::utils::{AppError, AppResult, verify_pin};
use super::wallet_service::{get_user_wallet, get_or_create_wallet, update_balance};

pub async fn process_p2p_payment(
    pool: &PgPool,
    sender_id: Uuid,
    recipient_phone: &str,
    amount: i64,
    pin: &str,
    description: Option<String>,
    offline_tx_id: Option<String>,
) -> AppResult<Transaction> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    // Check for duplicate offline_tx_id (idempotency)
    if let Some(ref off_id) = offline_tx_id {
        if let Some(existing) = sqlx::query_as::<_, Transaction>(
            "SELECT * FROM transactions WHERE offline_tx_id = $1"
        )
        .bind(off_id)
        .fetch_optional(&mut *tx)
        .await? {
            return Ok(existing);
        }
    }

    // Get sender wallet
    let sender_wallet = get_user_wallet(pool, sender_id).await?;

    // Verify sufficient balance
    if sender_wallet.balance < amount {
        return Err(AppError::InsufficientFunds);
    }

    // Verify PIN
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(sender_id)
    .fetch_one(pool)
    .await?;

    let pin_hash = user.pin_hash.as_ref()
        .ok_or_else(|| AppError::Auth("PIN not set".to_string()))?;

    if !verify_pin(pin, pin_hash)? {
        return Err(AppError::InvalidPIN);
    }

    // Find recipient by phone
    let recipient = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE phone = $1"
    )
    .bind(recipient_phone)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Recipient not found".to_string()))?;

    // Create transaction record
    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        INSERT INTO transactions (offline_tx_id, user_id, type, amount, currency, status, recipient_id, description)
        VALUES ($1, $2, 'p2p_send', $3, 'UGX', 'pending', $4, $5)
        RETURNING *
        "#
    )
    .bind(offline_tx_id)
    .bind(sender_id)
    .bind(amount)
    .bind(recipient.id)
    .bind(description)
    .fetch_one(&mut *tx)
    .await?;

    // Debit sender
    update_balance(pool, sender_wallet.id, -amount).await?;

    // Credit recipient (create wallet if doesn't exist)
    let recipient_wallet = get_or_create_wallet(pool, recipient.id).await?;
    update_balance(pool, recipient_wallet.id, amount).await?;

    // Update transaction status to completed
    let completed_transaction = sqlx::query_as::<_, Transaction>(
        r#"
        UPDATE transactions 
        SET status = 'completed', processed_at = NOW()
        WHERE id = $1
        RETURNING *
        "#
    )
    .bind(transaction.id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await.map_err(AppError::Database)?;

    // Send notifications (async, non-blocking)
    tokio::spawn(async move {
        // TODO: Implement notification service
        tracing::info!("P2P payment completed: {} -> {}", sender_id, recipient.id);
    });

    Ok(completed_transaction)
}

pub async fn get_transaction(
    pool: &PgPool,
    transaction_id: Uuid,
    user_id: Uuid,
) -> AppResult<Transaction> {
    let transaction = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions WHERE id = $1 AND user_id = $2"
    )
    .bind(transaction_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Transaction not found".to_string()))?;

    Ok(transaction)
}

pub async fn reverse_transaction(
    pool: &PgPool,
    transaction_id: Uuid,
    user_id: Uuid,
    reason: &str,
) -> AppResult<Transaction> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    let transaction = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions WHERE id = $1 AND user_id = $2"
    )
    .bind(transaction_id)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    if transaction.status == "reversed" {
        return Err(AppError::Validation("Transaction already reversed".to_string()));
    }

    // Reverse the transaction amounts
    match transaction.transaction_type.as_str() {
        "p2p_send" => {
            // Refund sender
            let sender_wallet = get_user_wallet(pool, user_id).await?;
            update_balance(pool, sender_wallet.id, transaction.amount).await?;

            // Debit recipient if they have a wallet
            if let Some(recipient_id) = transaction.recipient_id {
                if let Ok(recipient_wallet) = get_user_wallet(pool, recipient_id).await {
                    update_balance(pool, recipient_wallet.id, -transaction.amount).await?;
                }
            }
        }
        _ => {
            return Err(AppError::Validation("Cannot reverse this transaction type".to_string()));
        }
    }

    // Update transaction status
    let reversed_transaction = sqlx::query_as::<_, Transaction>(
        r#"
        UPDATE transactions 
        SET status = 'reversed', error_message = $1, processed_at = NOW()
        WHERE id = $2
        RETURNING *
        "#
    )
    .bind(reason)
    .bind(transaction_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await.map_err(AppError::Database)?;

    Ok(reversed_transaction)
}
