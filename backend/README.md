# Fintech Backend Service

A high-performance Rust backend for mobile money and P2P payments, built with Actix-web.

## Features

- **User Authentication**: Phone-based registration with OTP verification and PIN security
- **Wallet Management**: Balance tracking, top-ups, and withdrawals
- **P2P Payments**: Instant peer-to-peer transfers with offline sync support
- **Bill Payments**: Integration with bill providers (electricity, water, etc.)
- **Agent Network**: Cash-in/cash-out through agent locations
- **Offline-First**: Sync queue for unreliable network conditions

## Tech Stack

- **Framework**: Actix-web 4.x
- **Database**: PostgreSQL with SQLx
- **Cache**: Redis
- **Security**: Argon2id for PIN hashing, JWT for authentication
- **Serialization**: Serde/serde_json

## Getting Started

### Prerequisites

- Rust 1.70+
- PostgreSQL 15+
- Redis 7+

### Installation

```bash
# Clone the repository
git clone <repo-url>
cd backend

# Copy environment file
cp .env.example .env

# Edit .env with your configuration

# Build and run
cargo build
cargo run
```

### Database Setup

```bash
# Create database
createdb fintech_db

# Run migrations
sqlx migrate run
```

## API Endpoints

### Authentication
- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/verify-otp` - Verify OTP code
- `POST /api/v1/auth/login` - Login with phone + PIN

### Wallets
- `GET /api/v1/wallets` - Get wallet balance
- `POST /api/v1/wallets/topup` - Request mobile money top-up
- `POST /api/v1/wallets/withdraw` - Request cash-out

### Payments
- `POST /api/v1/payments/p2p` - Send money to another user
- `GET /api/v1/payments/p2p/{id}` - Get payment status

### Bills
- `GET /api/v1/bills/providers` - List bill providers
- `POST /api/v1/bills/pay` - Pay a bill

### Agents
- `GET /api/v1/agents/nearby` - Find nearby agents
- `POST /api/v1/agents/cash-in` - Record cash-in
- `POST /api/v1/agents/cash-out` - Record cash-out

### Sync
- `POST /api/v1/sync/push` - Push pending offline transactions
- `GET /api/v1/sync/pull` - Pull latest server state

## Project Structure

```
backend/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library exports
│   ├── config.rs        # Configuration management
│   ├── db.rs            # Database connection pool
│   ├── handlers/        # HTTP request handlers
│   ├── services/        # Business logic
│   ├── models/          # Data models
│   ├── middleware/      # Custom middleware
│   └── utils/           # Utilities (error handling, security)
├── migrations/          # SQL migrations
├── tests/               # Integration tests
└── Cargo.toml
```

## Security Considerations

- All sensitive data encrypted at rest (AES-256)
- PIN hashes using Argon2id
- JWT tokens with short expiry (24 hours)
- Rate limiting on authentication endpoints
- Idempotency keys prevent duplicate transactions

## License

MIT
