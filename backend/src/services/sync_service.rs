use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::utils::{AppError, AppResult};
use super::payment_service::process_p2p_payment;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingOperation {
    pub op_id: Uuid,
    pub operation_type: String,
    pub entity: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncResult {
    pub offline_id: Uuid,
    pub status: String,
    pub server_tx_id: Option<Uuid>,
    pub error: Option<String>,
}

pub async fn process_pending_operations(
    pool: &PgPool,
    user_id: Uuid,
    operations: Vec<PendingOperation>,
) -> AppResult<Vec<SyncResult>> {
    let mut results = Vec::new();

    for op in operations {
        let result = match op.operation_type.as_str() {
            "p2p" => {
                // Extract payment details from payload
                let recipient_phone = op.payload["recipient_phone"].as_str().unwrap_or("");
                let amount = op.payload["amount"].as_i64().unwrap_or(0);
                let pin = op.payload["pin"].as_str().unwrap_or("");
                let description = op.payload["description"].as_str().map(String::from);

                match process_p2p_payment(
                    pool,
                    user_id,
                    recipient_phone,
                    amount,
                    pin,
                    description,
                    Some(op.op_id.to_string()),
                ).await {
                    Ok(tx) => SyncResult {
                        offline_id: op.op_id,
                        status: "success".to_string(),
                        server_tx_id: Some(tx.id),
                        error: None,
                    },
                    Err(e) => SyncResult {
                        offline_id: op.op_id,
                        status: "failed".to_string(),
                        server_tx_id: None,
                        error: Some(e.to_string()),
                    },
                }
            }
            _ => SyncResult {
                offline_id: op.op_id,
                status: "unknown_type".to_string(),
                server_tx_id: None,
                error: Some(format!("Unknown operation type: {}", op.operation_type)),
            },
        };

        results.push(result);
    }

    Ok(results)
}

pub async fn get_server_changes(
    pool: &PgPool,
    user_id: Uuid,
    since: DateTime<Utc>,
) -> AppResult<serde_json::Value> {
    // Get transactions changed since last sync
    let transactions = sqlx::query(
        r#"
        SELECT id, type, amount, status, created_at, processed_at
        FROM transactions
        WHERE user_id = $1 AND (created_at > $2 OR processed_at > $2)
        ORDER BY created_at DESC
        LIMIT 100
        "#
    )
    .bind(user_id)
    .bind(since)
    .fetch_all(pool)
    .await?;

    // Get wallet balance changes
    let wallet = sqlx::query_as::<_, serde_json::Value>(
        r#"
        SELECT json_build_object('id', id, 'balance', balance, 'updated_at', updated_at) as wallet
        FROM wallets
        WHERE user_id = $1
        "#
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(serde_json::json!({
        "transactions": transactions,
        "wallet": wallet
    }))
}
