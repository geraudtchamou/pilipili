-- Migration: 001_initial_schema.sql
-- Creates the core database schema for the fintech backend

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "postgis";

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    phone VARCHAR(20) UNIQUE NOT NULL,
    phone_verified BOOLEAN DEFAULT FALSE,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    pin_hash VARCHAR(255),
    kyc_level INTEGER DEFAULT 0,
    country_code VARCHAR(5) DEFAULT 'UG',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_sync_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE
);

-- Wallets table
CREATE TABLE wallets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    balance BIGINT DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'UGX',
    status VARCHAR(20) DEFAULT 'active',
    version INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Transactions table
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    offline_tx_id VARCHAR(100) UNIQUE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL,
    amount BIGINT NOT NULL,
    currency VARCHAR(3) DEFAULT 'UGX',
    status VARCHAR(30) DEFAULT 'pending',
    recipient_id UUID REFERENCES users(id),
    recipient_phone VARCHAR(20),
    description TEXT,
    provider_reference VARCHAR(100),
    error_code VARCHAR(20),
    error_message TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    synced_at TIMESTAMPTZ,
    version INTEGER DEFAULT 0
);

-- OTP requests table
CREATE TABLE otp_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    phone VARCHAR(20) NOT NULL,
    otp_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    consumed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Agents table
CREATE TABLE agents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    business_name VARCHAR(200),
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    float_balance BIGINT DEFAULT 0,
    commission_rate DECIMAL(5, 4) DEFAULT 0.01,
    is_verified BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Bill providers table
CREATE TABLE bill_providers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    category VARCHAR(50),
    country_code VARCHAR(5) DEFAULT 'UG',
    api_endpoint VARCHAR(500),
    is_active BOOLEAN DEFAULT TRUE,
    config JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Sync queue table
CREATE TABLE sync_queue (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    operation_type VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL,
    priority INTEGER DEFAULT 0,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    error_message TEXT
);

-- Indexes for performance
CREATE INDEX idx_users_phone ON users(phone);
CREATE INDEX idx_wallets_user_id ON wallets(user_id);
CREATE INDEX idx_transactions_user_id ON transactions(user_id);
CREATE INDEX idx_transactions_type ON transactions(type);
CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_transactions_created_at ON transactions(created_at DESC);
CREATE INDEX idx_transactions_offline_tx_id ON transactions(offline_tx_id);
CREATE INDEX idx_agents_location ON agents USING GIST(ll_to_earth(latitude, longitude));
CREATE INDEX idx_sync_queue_user_id ON sync_queue(user_id);
CREATE INDEX idx_sync_queue_status ON sync_queue(status);
CREATE INDEX idx_otp_requests_phone ON otp_requests(phone);
CREATE INDEX idx_otp_requests_expires_at ON otp_requests(expires_at);

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_wallets_updated_at BEFORE UPDATE ON wallets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default bill providers
INSERT INTO bill_providers (name, category, api_endpoint) VALUES
    ('UMEME', 'electricity', 'https://api.umeme.co.ug/validate'),
    ('NWSC', 'water', 'https://api.nwsc.co.ug/validate'),
    ('MTN', 'mobile_money', 'https://api.mtn.co.ug/bill'),
    ('AIRTEL', 'mobile_money', 'https://api.airtel.co.ug/bill');
