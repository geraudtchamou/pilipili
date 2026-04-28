# Backend Engineer Agent - API & Database Design

## 1. API Endpoints

### Authentication & Users
| Method | Route | Description |
|--------|-------|-------------|
| POST | `/api/v1/auth/register` | Register new user with phone |
| POST | `/api/v1/auth/verify-otp` | Verify OTP code |
| POST | `/api/v1/auth/login` | Login with phone + PIN |
| GET | `/api/v1/users/profile` | Get user profile |
| PUT | `/api/v1/users/profile` | Update user profile |
| POST | `/api/v1/users/pin` | Set/Change transaction PIN |

### Wallets
| Method | Route | Description |
|--------|-------|-------------|
| GET | `/api/v1/wallets` | Get user wallet balance |
| POST | `/api/v1/wallets/topup` | Request mobile money top-up |
| POST | `/api/v1/wallets/withdraw` | Request cash-out to agent |
| GET | `/api/v1/wallets/history` | Get transaction history |

### P2P Payments
| Method | Route | Description |
|--------|-------|-------------|
| POST | `/api/v1/payments/p2p` | Send money to another user |
| GET | `/api/v1/payments/p2p/{id}` | Get payment status |
| POST | `/api/v1/payments/p2p/reverse` | Reverse failed payment |

### Bill Payments
| Method | Route | Description |
|--------|-------|-------------|
| GET | `/api/v1/bills/providers` | List bill providers |
| GET | `/api/v1/bills/validate` | Validate bill account number |
| POST | `/api/v1/bills/pay` | Pay a bill |
| GET | `/api/v1/bills/history` | Get bill payment history |

### Agents
| Method | Route | Description |
|--------|-------|-------------|
| GET | `/api/v1/agents/nearby` | Find nearby agents (lat/lng) |
| POST | `/api/v1/agents/cash-in` | Record cash-in transaction |
| POST | `/api/v1/agents/cash-out` | Record cash-out transaction |

### Sync (Offline-First)
| Method | Route | Description |
|--------|-------|-------------|
| POST | `/api/v1/sync/push` | Push pending offline transactions |
| GET | `/api/v1/sync/pull` | Pull latest server state |

---

## 2. Request/Response JSON Examples

### P2P Payment Request
```json
// POST /api/v1/payments/p2p
{
  "sender_id": "usr_abc123",
  "recipient_phone": "+256701234567",
  "amount": 50000,
  "currency": "UGX",
  "pin": "1234",
  "description": "Lunch money",
  "offline_tx_id": "off_tx_98765",
  "timestamp": "2024-01-15T10:30:00Z"
}

// Response 202 Accepted
{
  "transaction_id": "txn_xyz789",
  "status": "pending",
  "message": "Transaction queued for processing"
}

// Response 200 Success
{
  "transaction_id": "txn_xyz789",
  "status": "completed",
  "balance_after": 145000,
  "receipt_url": "https://api.example.com/receipts/txn_xyz789"
}
```

### Wallet Top-up Request
```json
// POST /api/v1/wallets/topup
{
  "user_id": "usr_abc123",
  "amount": 100000,
  "currency": "UGX",
  "mobile_money_provider": "MTN",
  "phone_number": "+256701234567",
  "callback_url": "https://api.example.com/webhooks/mtn"
}

// Response
{
  "topup_id": "top_456def",
  "status": "pending_provider",
  "ussd_code": "*165*1*256701234567*100000#",
  "expires_at": "2024-01-15T10:40:00Z"
}
```

---

## 3. Database Schema (PostgreSQL)

### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone VARCHAR(20) UNIQUE NOT NULL,
    phone_verified BOOLEAN DEFAULT FALSE,
    first_name VARCHAR(50),
    last_name VARCHAR(50),
    pin_hash VARCHAR(255),
    kyc_level INTEGER DEFAULT 0, -- 0: none, 1: basic, 2: full
    country_code VARCHAR(2) DEFAULT 'UG',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    last_sync_at TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX idx_users_phone ON users(phone);
CREATE INDEX idx_users_country ON users(country_code);
```

### Wallets Table
```sql
CREATE TABLE wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    balance BIGINT DEFAULT 0, -- stored in smallest currency unit
    currency VARCHAR(3) DEFAULT 'UGX',
    status VARCHAR(20) DEFAULT 'active', -- active, frozen, closed
    version INTEGER DEFAULT 0, -- for optimistic locking
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_wallets_user ON wallets(user_id);
```

### Transactions Table
```sql
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    offline_tx_id VARCHAR(100), -- client-generated ID for sync
    user_id UUID REFERENCES users(id),
    type VARCHAR(30) NOT NULL, -- p2p_send, p2p_receive, topup, withdrawal, bill_payment
    amount BIGINT NOT NULL,
    currency VARCHAR(3) DEFAULT 'UGX',
    status VARCHAR(20) DEFAULT 'pending', -- pending, completed, failed, reversed
    recipient_id UUID REFERENCES users(id),
    recipient_phone VARCHAR(20),
    description TEXT,
    provider_reference VARCHAR(100), -- mobile money reference
    error_code VARCHAR(50),
    error_message TEXT,
    metadata JSONB, -- flexible storage for bill details, agent info, etc.
    created_at TIMESTAMP DEFAULT NOW(),
    processed_at TIMESTAMP,
    synced_at TIMESTAMP,
    version INTEGER DEFAULT 0
);

CREATE INDEX idx_transactions_user ON transactions(user_id);
CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_transactions_created ON transactions(created_at);
CREATE INDEX idx_transactions_offline ON transactions(offline_tx_id);
```

### Bill Providers Table
```sql
CREATE TABLE bill_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    category VARCHAR(50), -- electricity, water, internet
    country_code VARCHAR(2),
    api_endpoint VARCHAR(255),
    api_key_encrypted TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    config JSONB -- provider-specific configuration
);

CREATE TABLE bill_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    provider_id UUID REFERENCES bill_providers(id),
    account_number VARCHAR(100) NOT NULL,
    account_name VARCHAR(100),
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);
```

### Agents Table
```sql
CREATE TABLE agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    business_name VARCHAR(100),
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    float_balance BIGINT DEFAULT 0,
    commission_rate DECIMAL(5, 4) DEFAULT 0.01,
    is_verified BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_agents_location ON agents USING GIST (ll_to_earth(latitude, longitude));
```

### Sync Queue Table
```sql
CREATE TABLE sync_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    operation_type VARCHAR(50),
    payload JSONB NOT NULL,
    priority INTEGER DEFAULT 5,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMP DEFAULT NOW(),
    processed_at TIMESTAMP,
    error_message TEXT
);

CREATE INDEX idx_sync_queue_user ON sync_queue(user_id);
CREATE INDEX idx_sync_queue_status ON sync_queue(status);
```

---

## 4. Key Logic (Pseudocode)

### P2P Payment Processing
```rust
async fn process_p2p_payment(request: P2PRequest) -> Result<Transaction> {
    // 1. Acquire lock on sender wallet (optimistic locking)
    let mut wallet = get_wallet_with_lock(request.sender_id).await?;
    
    // 2. Verify sufficient balance
    if wallet.balance < request.amount {
        return Err(Error::InsufficientFunds);
    }
    
    // 3. Verify PIN
    if !verify_pin(request.sender_id, request.pin).await? {
        return Err(Error::InvalidPIN);
    }
    
    // 4. Check for duplicate offline_tx_id (idempotency)
    if let Some(existing) = find_by_offline_id(request.offline_tx_id).await? {
        return Ok(existing); // Return existing transaction
    }
    
    // 5. Find recipient by phone
    let recipient = find_user_by_phone(request.recipient_phone)
        .await?
        .ok_or(Error::RecipientNotFound)?;
    
    // 6. Create transaction record
    let tx = Transaction {
        id: generate_id(),
        offline_tx_id: request.offline_tx_id,
        user_id: request.sender_id,
        type: "p2p_send",
        amount: request.amount,
        status: "pending",
        recipient_id: Some(recipient.id),
        ..Default::default()
    };
    save_transaction(&tx).await?;
    
    // 7. Debit sender (atomic)
    wallet.balance -= request.amount;
    wallet.version += 1;
    update_wallet(wallet).await?;
    
    // 8. Credit recipient (atomic)
    let mut recipient_wallet = get_wallet(recipient.id).await?;
    recipient_wallet.balance += request.amount;
    recipient_wallet.version += 1;
    update_wallet(recipient_wallet).await?;
    
    // 9. Update transaction status
    tx.status = "completed";
    tx.processed_at = Some(now());
    update_transaction(tx).await?;
    
    // 10. Send notifications (async, non-blocking)
    send_sms_notification(request.sender_id, "Money sent successfully").await;
    send_sms_notification(recipient.id, "Money received").await;
    
    Ok(tx)
}
```

### Offline Sync Handler
```rust
async fn handle_sync_push(user_id: UUID, pending_txs: Vec<OfflineTransaction>) -> Result<SyncResponse> {
    let mut results = Vec::new();
    
    for tx in pending_txs {
        // Check idempotency
        if let Some(existing) = find_by_offline_id(tx.offline_id).await? {
            results.push(SyncResult {
                offline_id: tx.offline_id,
                status: "already_processed",
                server_tx_id: existing.id,
            });
            continue;
        }
        
        // Process based on transaction type
        let result = match tx.tx_type {
            "p2p" => process_p2p_payment(tx.into()).await,
            "bill" => process_bill_payment(tx.into()).await,
            _ => Err(Error::UnknownTransactionType),
        };
        
        results.push(match result {
            Ok(tx) => SyncResult {
                offline_id: tx.offline_tx_id.unwrap(),
                status: "success",
                server_tx_id: tx.id,
            },
            Err(e) => SyncResult {
                offline_id: tx.offline_id,
                status: "failed",
                error: e.message,
                ..Default::default()
            },
        });
    }
    
    // Return server state updates
    let server_updates = get_server_changes_since(user_id, user.last_sync_at).await?;
    
    Ok(SyncResponse {
        results,
        server_updates,
        new_last_sync: now(),
    })
}
```

### Mobile Money Callback Handler
```rust
async fn handle_mobile_money_callback(provider: String, callback: WebhookPayload) -> Result<()> {
    // 1. Verify webhook signature
    if !verify_webhook_signature(provider, &callback).await? {
        return Err(Error::InvalidSignature);
    }
    
    // 2. Find transaction by provider reference
    let mut tx = find_transaction_by_provider_ref(callback.reference)
        .await?
        .ok_or(Error::TransactionNotFound)?;
    
    // 3. Update transaction status based on callback
    match callback.status {
        "completed" => {
            tx.status = "completed";
            tx.processed_at = Some(now());
        },
        "failed" => {
            tx.status = "failed";
            tx.error_code = Some(callback.error_code);
            tx.error_message = Some(callback.error_message);
            
            // Refund if money was debited
            if tx.type == "topup" {
                refund_topup(tx.user_id, tx.amount).await?;
            }
        },
        _ => return Err(Error::UnknownStatus),
    }
    
    // 4. Save updated transaction
    update_transaction(tx).await?;
    
    // 5. Notify user via SMS
    send_sms_notification(tx.user_id, format!("Payment {}", callback.status)).await;
    
    Ok(())
}
```

---

## Tech Stack Implementation Notes

- **Framework**: Rust with Actix-web (high performance, low memory)
- **Database**: PostgreSQL with connection pooling (deadpool-postgres)
- **Cache**: Redis for session management and rate limiting
- **Queue**: Redis Streams for async job processing
- **Serialization**: serde_json for all JSON handling
- **Validation**: validator crate for input validation
- **Security**: argon2 for PIN hashing, aes-gcm for sensitive data encryption
- **Logging**: tracing crate with structured logging
- **Metrics**: prometheus metrics exposed on /metrics endpoint

## Next Steps for Backend Team

1. Set up project structure with Cargo workspaces
2. Implement database migrations with sqlx
3. Build authentication middleware
4. Create API endpoint skeletons
5. Implement idempotency layer
6. Set up Redis connection pool
7. Configure structured logging
8. Write unit tests for core logic
