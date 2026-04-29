use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub phone: String,
    pub phone_verified: bool,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub pin_hash: Option<String>,
    pub kyc_level: i32,
    pub country_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub balance: i64,
    pub currency: String,
    pub status: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub offline_tx_id: Option<String>,
    pub user_id: Uuid,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub recipient_id: Option<Uuid>,
    pub recipient_phone: Option<String>,
    pub description: Option<String>,
    pub provider_reference: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub synced_at: Option<DateTime<Utc>>,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub business_name: Option<String>,
    pub latitude: Option<rust_decimal::Decimal>,
    pub longitude: Option<rust_decimal::Decimal>,
    pub float_balance: i64,
    pub commission_rate: rust_decimal::Decimal,
    pub is_verified: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillProvider {
    pub id: Uuid,
    pub name: String,
    pub category: Option<String>,
    pub country_code: String,
    pub api_endpoint: Option<String>,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncQueueItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub operation_type: String,
    pub payload: serde_json::Value,
    pub priority: i32,
    pub retry_count: i32,
    pub max_retries: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}
