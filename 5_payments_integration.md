# Payments Integration Agent - Mobile Money Integration

## 1. Supported Providers (MVP)

| Provider | Countries | API Type | Settlement |
|----------|-----------|----------|------------|
| MTN Mobile Money | Uganda, Ghana, Cameroon, Zambia | REST API | T+1 |
| Orange Money | Cameroon, Ivory Coast, Mali, Senegal | REST API | T+1 |
| Airtel Money | Uganda, Kenya, Tanzania, Malawi | REST API | T+1 |

---

## 2. API Integration Flow

### MTN Mobile Money Integration

#### Authentication
```rust
// Get access token
POST https://sandbox.momodeveloper.mtn.com/oauth/token/
Headers:
  Authorization: Basic base64(api_key:api_secret)
  Content-Type: application/x-www-form-urlencoded
Body:
  grant_type=client_credentials

Response:
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "collection payment"
}
```

#### Collection Request (Cash-In)
```rust
// Request to collect money from user
POST https://sandbox.momodeveloper.mtn.com/collection/v1_0/requesttopay
Headers:
  Authorization: Bearer {access_token}
  X-Reference-Id: {unique_uuid}
  X-Target-Environment: {environment}
  Ocp-Apim-Subscription-Key: {subscription_key}
  Content-Type: application/json
Body:
{
  "amount": "100000",
  "currency": "UGX",
  "externalId": "usr_abc123_topup_12345",
  "payer": {
    "partyIdType": "MSISDN",
    "partyId": "256701234567"
  },
  "payerMessage": "Top up wallet",
  "payeeNote": "Wallet top-up via PayFlow"
}

Response: 202 Accepted
{
  "transactionId": "MTN_1234567890",
  "status": "PENDING"
}
```

#### Transaction Status Check
```rust
GET https://sandbox.momodeveloper.mtn.com/collection/v1_0/requesttopay/{referenceId}
Headers:
  Authorization: Bearer {access_token}
  X-Target-Environment: {environment}
  Ocp-Apim-Subscription-Key: {subscription_key}

Response:
{
  "financialTransactionId": "FT_9876543210",
  "referenceId": "ref_uuid_here",
  "status": "SUCCESSFUL",
  "amount": "100000",
  "currency": "UGX",
  "feeAmount": "0",
  "externalId": "usr_abc123_topup_12345",
  "payer": {
    "partyIdType": "MSISDN",
    "partyId": "256701234567"
  },
  "createdDateTime": "2024-01-15T10:30:00Z"
}
```

---

### Orange Money Integration

#### Authentication
```rust
POST https://api.orange.com/oauth/v3/token
Headers:
  Authorization: Basic base64(client_id:client_secret)
  Content-Type: application/x-www-form-urlencoded
Body:
  grant_type=client_credentials

Response:
{
  "access_token": "orange_token_here",
  "token_type": "bearer",
  "expires_in": 3600
}
```

#### Payment Request
```rust
POST https://api.orange.com/orange-money-webpay/dev/v1/webpayment
Headers:
  Authorization: Bearer {access_token}
  Content-Type: application/json
Body:
{
  "webPaymentToken": "generated_token",
  "merchantKey": "merchant_key",
  "orderRef": "order_ref_12345",
  "amount": "100000",
  "currency": "XAF",
  "returnUrl": "https://app.example.com/callback/orange",
  "cancelUrl": "https://app.example.com/cancel/orange",
  "notifUrl": "https://api.example.com/webhooks/orange",
  "lang": "en",
  "customerMobile": "237601234567"
}
```

---

### Airtel Money Integration

#### Authentication
```rust
POST https://openapi.airtel.africa/oauth/v1/token
Headers:
  Content-Type: application/json
Body:
{
  "appId": "your_app_id",
  "secret": "your_secret"
}

Response:
{
  "token": "airtel_token_here",
  "expires_in": 3600
}
```

#### Payment Request
```rust
POST https://openapi.airtel.africa/standard/v2/payments
Headers:
  X-Country: UG
  X-Currency: UGX
  Authorization: Bearer {token}
  Content-Type: application/json
Body:
{
  "request_id": "unique_request_id",
  "service_id": "collections",
  "amount": 100000,
  "subscriber_country": "UG",
  "msisdn": "256701234567",
  "narration": "Wallet top-up via PayFlow"
}

Response:
{
  "status": "pending_authorization",
  "message": "Awaiting customer authorization",
  "data": {
    "transaction_id": "AIRTEL_TXN_123456"
  }
}
```

---

## 3. Webhook Handling

### Unified Webhook Endpoint Structure

```rust
// POST /api/v1/webhooks/{provider}
// Providers: mtn, orange, airtel

#[derive(Deserialize)]
struct MtnWebhook {
    reference_id: String,
    status: String, // SUCCESSFUL, FAILED
    amount: String,
    currency: String,
    external_id: String,
    financial_transaction_id: String,
    created_date_time: String,
}

#[derive(Deserialize)]
struct OrangeWebhook {
    orderRef: String,
    status: String, // success, failed, cancelled
    amount: String,
    currency: String,
    transactionId: String,
    customerMobile: String,
}

#[derive(Deserialize)]
struct AirtelWebhook {
    request_id: String,
    status: String, // completed, failed
    transaction_id: String,
    msisdn: String,
    amount: f64,
}
```

### Webhook Handler Implementation

```rust
async fn handle_webhook(
    provider: String,
    payload: Json<serde_json::Value>,
    headers: HeaderMap,
) -> Result<Json<WebhookResponse>> {
    
    // 1. Verify webhook signature
    match provider.as_str() {
        "mtn" => verify_mtn_signature(&headers, &payload)?,
        "orange" => verify_orange_signature(&headers, &payload)?,
        "airtel" => verify_airtel_signature(&headers, &payload)?,
        _ => return Err(Error::UnknownProvider),
    }
    
    // 2. Parse and normalize the webhook
    let normalized = normalize_webhook(provider, &payload)?;
    
    // 3. Find transaction by reference
    let mut transaction = db::find_transaction_by_provider_ref(
        &normalized.reference_id
    ).await?
    .ok_or(Error::TransactionNotFound)?;
    
    // 4. Update transaction status
    match normalized.status.as_str() {
        "SUCCESSFUL" | "success" | "completed" => {
            transaction.status = "completed";
            transaction.processed_at = Some(Utc::now());
            transaction.provider_reference = Some(normalized.financial_tx_id);
            
            // Credit user wallet
            db::credit_wallet(
                transaction.user_id,
                transaction.amount,
            ).await?;
        },
        "FAILED" | "failed" => {
            transaction.status = "failed";
            transaction.error_code = Some(normalized.error_code);
            transaction.error_message = Some(normalized.error_message);
            
            // If money was debited but failed, initiate refund
            if transaction.type == "topup" {
                queue_refund(transaction).await?;
            }
        },
        _ => return Err(Error::UnknownStatus),
    }
    
    // 5. Save updated transaction
    db::update_transaction(transaction).await?;
    
    // 6. Send notification to user
    notifications::send_sms(
        transaction.user_id,
        format!("Your {} of {} is {}", 
            transaction.type,
            format_amount(transaction.amount),
            transaction.status
        )
    ).await;
    
    // 7. Return acknowledgment
    Ok(Json(WebhookResponse {
        status: "acknowledged",
        message: "Webhook processed successfully",
    }))
}

fn normalize_webhook(
    provider: String,
    payload: &serde_json::Value,
) -> Result<NormalizedWebhook> {
    match provider.as_str() {
        "mtn" => {
            let data: MtnWebhook = serde_json::from_value(payload.clone())?;
            Ok(NormalizedWebhook {
                reference_id: data.reference_id,
                status: data.status,
                amount: data.amount.parse()?,
                currency: data.currency,
                financial_tx_id: data.financial_transaction_id,
                error_code: None,
                error_message: None,
            })
        },
        "orange" => {
            let data: OrangeWebhook = serde_json::from_value(payload.clone())?;
            Ok(NormalizedWebhook {
                reference_id: data.orderRef,
                status: data.status.to_uppercase(),
                amount: data.amount.parse()?,
                currency: data.currency,
                financial_tx_id: data.transactionId,
                error_code: None,
                error_message: None,
            })
        },
        "airtel" => {
            let data: AirtelWebhook = serde_json::from_value(payload.clone())?;
            Ok(NormalizedWebhook {
                reference_id: data.request_id,
                status: data.status.to_uppercase(),
                amount: data.amount as i64,
                currency: "UGX".to_string(),
                financial_tx_id: data.transaction_id,
                error_code: None,
                error_message: None,
            })
        },
        _ => Err(Error::UnknownProvider),
    }
}
```

---

## 4. Error Handling Strategy

### Error Code Mapping

| Provider Error | Internal Code | User Message | Retry? |
|----------------|---------------|--------------|--------|
| INSUFFICIENT_FUNDS | E001 | "Insufficient balance in mobile money account" | No |
| INVALID_MSISDN | E002 | "Invalid phone number" | No |
| TRANSACTION_LIMIT_EXCEEDED | E003 | "Amount exceeds transaction limit" | No |
| DUPLICATE_TRANSACTION | E004 | "Transaction already in progress" | No |
| NETWORK_TIMEOUT | E005 | "Network timeout, please check status later" | Yes |
| PROVIDER_UNAVAILABLE | E006 | "Service temporarily unavailable" | Yes |
| CUSTOMER_REJECTED | E007 | "Transaction cancelled by customer" | No |
| EXPIRED_TRANSACTION | E008 | "Transaction expired, please try again" | Yes |

### Retry Logic

```rust
struct RetryConfig {
    max_retries: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
    multiplier: f64,
}

impl RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            multiplier: 2.0,
        }
    }
}

async fn execute_with_retry<T, F, Fut>(
    operation: F,
    config: RetryConfig,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay_ms;
    
    loop {
        attempt += 1;
        
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt >= config.max_retries || !e.is_retryable() {
                    return Err(e);
                }
                
                // Log retry attempt
                tracing::warn!(
                    "Retry attempt {} after error: {:?}",
                    attempt,
                    e
                );
                
                // Exponential backoff with jitter
                tokio::time::sleep(
                    Duration::from_millis(delay + rand::random::<u64>() % 500)
                ).await;
                
                delay = (delay as f64 * config.multiplier) as u64;
                delay = delay.min(config.max_delay_ms);
            }
        }
    }
}

// Usage in payment flow
async fn initiate_collection(payment: CollectionRequest) -> Result<CollectionResponse> {
    execute_with_retry(
        || async {
            let token = get_access_token().await?;
            mtn_collection_api::request_to_pay(&token, &payment).await
        },
        RetryConfig::default(),
    ).await
}
```

---

## 5. Reconciliation Process

### Daily Reconciliation Job

```rust
async fn daily_reconciliation(provider: String, date: NaiveDate) -> Result<ReconciliationReport> {
    // 1. Fetch all transactions for the day from our DB
    let our_transactions = db::get_transactions_by_date_and_provider(
        provider.clone(),
        date,
    ).await?;
    
    // 2. Fetch settlement report from provider
    let provider_report = match provider.as_str() {
        "mtn" => mtn_api::get_settlement_report(date).await?,
        "orange" => orange_api::get_settlement_report(date).await?,
        "airtel" => airtel_api::get_settlement_report(date).await?,
        _ => return Err(Error::UnknownProvider),
    };
    
    let mut report = ReconciliationReport {
        date,
        provider,
        total_our_count: our_transactions.len(),
        total_provider_count: provider_report.transactions.len(),
        matched_count: 0,
        mismatched_count: 0,
        missing_in_our_system: vec![],
        missing_in_provider: vec![],
        amount_discrepancies: vec![],
    };
    
    // 3. Match transactions by reference ID
    let mut provider_map: HashMap<String, &ProviderTransaction> = provider_report
        .transactions
        .iter()
        .map(|t| (t.reference_id.clone(), t))
        .collect();
    
    for our_tx in &our_transactions {
        if let Some(provider_tx) = provider_map.remove(&our_tx.provider_reference) {
            report.matched_count += 1;
            
            // Check amount match
            if our_tx.amount != provider_tx.amount {
                report.amount_discrepancies.push(AmountDiscrepancy {
                    reference_id: our_tx.provider_reference.clone(),
                    our_amount: our_tx.amount,
                    provider_amount: provider_tx.amount,
                    difference: our_tx.amount - provider_tx.amount,
                });
            }
        } else {
            report.missing_in_provider.push(our_tx.reference_id.clone());
        }
    }
    
    // Remaining items are missing in our system
    for (_, provider_tx) in provider_map {
        report.missing_in_our_system.push(provider_tx.reference_id.clone());
    }
    
    report.mismatched_count = 
        report.missing_in_our_system.len() + 
        report.missing_in_provider.len() +
        report.amount_discrepancies.len();
    
    // 4. Generate alerts for discrepancies
    if report.mismatched_count > 0 {
        alerts::send_reconciliation_alert(&report).await;
    }
    
    // 5. Store report
    db::save_reconciliation_report(&report).await?;
    
    Ok(report)
}
```

---

## 6. Configuration Management

### Provider Configuration (Environment Variables)

```bash
# MTN
MTN_API_KEY=your_mtn_api_key
MTN_API_SECRET=your_mtn_api_secret
MTN_SUBSCRIPTION_KEY=your_subscription_key
MTN_ENVIRONMENT=sandbox # or production
MTN_WEBHOOK_SECRET=your_webhook_secret

# Orange
ORANGE_CLIENT_ID=your_orange_client_id
ORANGE_CLIENT_SECRET=your_orange_client_secret
ORANGE_MERCHANT_KEY=your_merchant_key
ORANGE_WEBHOOK_SECRET=your_webhook_secret

# Airtel
AIRTEL_APP_ID=your_airtel_app_id
AIRTEL_SECRET=your_airtel_secret
AIRTEL_COUNTRY=UG
AIRTEL_WEBHOOK_SECRET=your_webhook_secret

# General
WEBHOOK_BASE_URL=https://api.example.com/api/v1/webhooks
RECONCILIATION_TIME=02:00 # UTC
```

---

## 7. Next Steps for Payments Team

1. Register developer accounts with MTN, Orange, Airtel
2. Obtain sandbox credentials and test API connectivity
3. Implement authentication token management with caching
4. Build unified payment interface abstraction layer
5. Implement webhook endpoints with signature verification
6. Create retry logic with exponential backoff
7. Set up daily reconciliation jobs
8. Build admin dashboard for transaction monitoring
9. Test end-to-end flows in sandbox environment
10. Prepare documentation for production onboarding
