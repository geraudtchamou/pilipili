# System Architecture Design

## 1. High-Level Architecture Diagram (Text)

```
┌─────────────────────────────────────────────────────────────────┐
│                      CLIENT LAYER                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  Android    │  │  Android    │  │   Agent     │              │
│  │  User App   │  │  Lite App   │  │   App       │              │
│  │  (Flutter)  │  │  (Flutter)  │  │  (Flutter)  │              │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘              │
│         │                │                │                      │
│         └────────────────┼────────────────┘                      │
│                          │                                       │
│                  ┌───────▼───────┐                              │
│                  │  Offline Sync │                              │
│                  │    Manager    │                              │
│                  └───────┬───────┘                              │
└──────────────────────────┼──────────────────────────────────────┘
                           │ HTTPS/WebSocket
┌──────────────────────────▼──────────────────────────────────────┐
│                      API GATEWAY                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Kong / Traefik                                         │    │
│  │  - Rate Limiting  - Authentication  - Load Balancing    │    │
│  └─────────────────────────────────────────────────────────┘    │
└──────────────────────────┬──────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│                   MICROSERVICES LAYER                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │   User   │ │  Wallet  │ │Transaction│ │   Bill   │           │
│  │ Service  │ │ Service  │ │  Service  │ │ Service  │           │
│  │  (Rust)  │ │  (Rust)  │ │  (Rust)   │ │  (Rust)  │           │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘           │
│       │            │            │            │                  │
│  ┌────▼─────┐ ┌────▼─────┐ ┌────▼─────┐ ┌────▼─────┐           │
│  │   Rent   │ │  Agent   │ │  Notify  │ │   Sync   │           │
│  │ Service  │ │ Service  │ │ Service  │ │ Service  │           │
│  │  (Rust)  │ │  (Rust)  │ │  (Rust)  │ │  (Rust)  │           │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘           │
└───────┼────────────┼────────────┼────────────┼──────────────────┘
        │            │            │            │
┌───────▼────────────▼────────────▼────────────▼──────────────────┐
│                    DATA LAYER                                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │ PostgreSQL  │  │    Redis    │  │  Message    │              │
│  │  (Primary)  │  │   (Cache)   │  │   Queue     │              │
│  │  + Replicas │  │  + Pub/Sub  │  │  (RabbitMQ) │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
└─────────────────────────────────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│                 EXTERNAL INTEGRATIONS                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │   MTN    │ │  Orange  │ │  Airtel  │ │  SMS     │           │
│  │  MoMo    │ │  Money   │ │  Money   │ │ Gateway  │           │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

## 2. Services List

| Service | Responsibility | Tech Stack | Port |
|---------|---------------|------------|------|
| **User Service** | Registration, KYC, authentication, profiles | Rust + Actix | 8001 |
| **Wallet Service** | Balance management, ledger, holds | Rust + Actix | 8002 |
| **Transaction Service** | P2P, remittance, state machine, idempotency | Rust + Actix | 8003 |
| **Bill Service** | Biller integration, payment processing | Rust + Actix | 8004 |
| **Rent Service** | Lease management, recurring payments | Rust + Actix | 8005 |
| **Agent Service** | Agent network, cash-in/out, commissions | Rust + Actix | 8006 |
| **Notification Service** | SMS, push, WhatsApp notifications | Rust + Actix | 8007 |
| **Sync Service** | Offline sync coordination, conflict resolution | Rust + Actix | 8008 |

## 3. Data Flow Description

### 3.1 P2P Payment Flow (Online)
```
1. User App → API Gateway → Transaction Service
2. Transaction Service → Wallet Service (debit sender)
3. Transaction Service → Wallet Service (credit receiver)
4. Wallet Service → PostgreSQL (ledger update)
5. Transaction Service → Notification Service (SMS confirmation)
6. Response → User App
```

### 3.2 P2P Payment Flow (Offline)
```
1. User App → Local SQLite (queue transaction)
2. Sync Manager detects connectivity
3. Sync Manager → Sync Service (batch upload)
4. Sync Service → Transaction Service (process queued)
5. Sync Service → Sync Manager (confirmation)
6. Sync Manager → Local SQLite (update status)
```

### 3.3 Mobile Money Cash-In Flow
```
1. User App → API Gateway → Agent Service
2. Agent Service → External Mobile Money API (USSD push)
3. Mobile Money → Webhook → Agent Service
4. Agent Service → Wallet Service (credit user wallet)
5. Agent Service → Notification Service (confirmation)
```

## 4. Sync Architecture

### 4.1 Offline-First Strategy
- **Local Storage**: SQLite on device with encrypted WAL
- **Operation Queue**: All mutations queued locally first
- **Sync Trigger**: Connectivity detection + periodic heartbeat
- **Conflict Resolution**: Last-write-wins with server authority
- **Delta Sync**: Only changed records transferred

### 4.2 Sync Protocol
```json
{
  "sync_request": {
    "device_id": "uuid",
    "last_sync_timestamp": 1699900000,
    "pending_operations": [
      {
        "op_id": "uuid",
        "type": "CREATE|UPDATE|DELETE",
        "entity": "transaction",
        "payload": {},
        "timestamp": 1699900100
      }
    ]
  }
}
```

### 4.3 Conflict Resolution Rules
| Scenario | Resolution |
|----------|------------|
| Balance update conflict | Server wins, notify user |
| Transaction duplicate | Idempotency key prevents double-process |
| Profile update conflict | Last timestamp wins |
| Offline transaction vs insufficient funds | Reject on sync, notify user |

## 5. Key Technical Decisions

### 5.1 Language & Framework
- **Backend**: Rust with Actix-web for performance and memory safety
- **Mobile**: Flutter for cross-platform with Android-first optimization
- **Database**: PostgreSQL 15 with logical replication

### 5.2 Scalability Decisions
- **Horizontal Scaling**: Stateless services behind load balancer
- **Database Sharding**: By country code for multi-country expansion
- **Caching Strategy**: Redis for sessions, rates, frequent reads
- **Message Queue**: RabbitMQ for async operations (notifications, reconciliation)

### 5.3 Reliability Decisions
- **Idempotency**: All payment endpoints require idempotency keys
- **Circuit Breakers**: External API calls protected with fallbacks
- **Retry Logic**: Exponential backoff with max 3 retries
- **Dead Letter Queue**: Failed transactions stored for manual review

### 5.4 Security Decisions
- **Encryption**: TLS 1.3 in transit, AES-256 at rest
- **Authentication**: JWT with short expiry + refresh tokens
- **PIN Protection**: Argon2id hashing, rate-limited attempts
- **Device Binding**: Device fingerprint tied to user session

### 5.5 Low-Bandwidth Optimizations
- **Protocol Buffers**: Binary serialization for mobile↔server
- **Compression**: gzip for all API responses
- **Batching**: Multiple operations in single request
- **Lazy Loading**: Minimal initial payload, fetch-on-demand

## 6. Failure Handling

### 6.1 Service Failure Matrix
| Failure Type | Detection | Recovery | User Impact |
|--------------|-----------|----------|-------------|
| Database connection loss | Health check < 5s | Auto-reconnect, read replica failover | Temporary read-only |
| External payment API down | Circuit breaker open | Queue transactions, retry later | Delayed confirmation |
| Sync service unavailable | Heartbeat timeout | Local queue persistence | Offline mode extended |
| Message queue full | Queue depth monitor | Scale consumers, alert ops | Notification delays |

### 6.2 Disaster Recovery
- **RPO**: < 5 minutes (WAL archiving)
- **RTO**: < 30 minutes (automated failover)
- **Backup Strategy**: Daily full + continuous WAL to S3
- **Multi-AZ**: Primary services across 2 availability zones

## 7. API Structure Overview

### 7.1 RESTful Endpoints Pattern
```
/api/v1/{service}/{resource}
Examples:
  POST   /api/v1/users/register
  GET    /api/v1/wallets/{id}/balance
  POST   /api/v1/transactions/p2p
  GET    /api/v1/bills/categories
```

### 7.2 WebSocket Channels
```
/ws/sync          - Real-time sync updates
/ws/notifications - Push notifications
/ws/transactions  - Transaction status updates
```

---

*Architecture document ready for Backend Engineer Agent to implement APIs and database schemas.*
