# MVP Definition - Fintech App for Sub-Saharan Africa

## Phase 1: MVP Scope (Months 1-3)

### Core Features for MVP

#### 1. User Registration & Onboarding
**Description:** Simple registration flow with phone number verification, basic KYC, and wallet creation.

**User Stories:**
- As a user, I want to register with my phone number so I can start using the app
- As a user, I want to verify my identity with minimal steps so I can quickly access services
- As a user, I want to create a digital wallet automatically upon registration

**Acceptance Criteria:**
- Registration completes in under 3 minutes
- SMS OTP verification works offline (queued for retry)
- Wallet is created with zero balance initially
- Support for low-literacy users (icon-based navigation)

**Priority:** MVP

---

#### 2. P2P Payments (Domestic)
**Description:** Send and receive money between users within the same country.

**User Stories:**
- As a user, I want to send money to another user by phone number
- As a user, I want to receive money instantly into my wallet
- As a user, I want to see my transaction history clearly

**Acceptance Criteria:**
- Transfer completes in under 30 seconds on good connection
- Works offline with sync when connection restored
- Transaction confirmation via SMS
- Maximum transaction limit enforced (configurable per country)

**Priority:** MVP

---

#### 3. Mobile Money Cash-In/Cash-Out
**Description:** Deposit and withdraw funds using MTN, Orange, and Airtel mobile money.

**User Stories:**
- As a user, I want to add money to my wallet from my mobile money account
- As a user, I want to withdraw money from my wallet to my mobile money account
- As a user, I want to see available mobile money providers

**Acceptance Criteria:**
- Integration with at least one provider per country (MTN, Orange, or Airtel)
- Callback handling for asynchronous transactions
- Failed transaction rollback within 5 minutes
- Clear error messages for insufficient funds, network issues

**Priority:** MVP

---

#### 4. Basic Bill Payments
**Description:** Pay electricity bills for one major provider per country.

**User Stories:**
- As a user, I want to pay my electricity bill using my wallet balance
- As a user, I want to enter my meter number and see my bill amount
- As a user, I want to receive confirmation after payment

**Acceptance Criteria:**
- Integration with one electricity provider per launch country
- Meter validation before payment
- Receipt generation (stored locally and synced)
- Offline queue for bill payment requests

**Priority:** MVP

---

#### 5. Transaction History & Balance
**Description:** View wallet balance and all transactions with filtering.

**User Stories:**
- As a user, I want to see my current wallet balance always
- As a user, I want to view all my past transactions
- As a user, I want to filter transactions by type (send, receive, bill, cash-in, cash-out)

**Acceptance Criteria:**
- Balance visible on home screen
- Last 50 transactions stored locally
- Sync with server when online
- Icons for each transaction type (low-literacy friendly)

**Priority:** MVP

---

### Excluded from MVP (Phase 2/3)

| Feature | Phase | Reason |
|---------|-------|--------|
| Cross-border remittance | Phase 2 | Requires additional regulatory compliance |
| Water & Internet bill payments | Phase 2 | Expand after electricity integration proven |
| Rent collection system | Phase 3 | Complex multi-party workflow |
| Full agent network | Phase 2 | Requires agent onboarding system |
| WhatsApp integration | Phase 2 | Secondary channel after core app stable |
| Multi-country support | Phase 2 | Launch single country first |

---

## Success Metrics for MVP

- **User Adoption:** 1,000 active users in first month
- **Transaction Success Rate:** >95% for P2P, >90% for mobile money
- **Offline Functionality:** 80% of features usable without internet
- **App Size:** <15MB download
- **Registration Time:** <3 minutes average
- **Customer Support Tickets:** <5% of users requiring assistance

---

## Technical MVP Constraints

- **Platform:** Android only (API 21+)
- **Backend:** Single region deployment (expand later)
- **Database:** PostgreSQL with read replicas
- **Cache:** Redis for session and rate limiting
- **Sync:** SQLite local storage with conflict resolution
- **Security:** End-to-end encryption for sensitive data
- **Compliance:** Basic KYC, transaction logging for audit

---

## Next Steps

1. **System Architect** - Design overall architecture based on MVP scope
2. **Backend Engineer** - Define APIs and database schema for MVP features
3. **Mobile Engineer** - Plan Flutter app structure and offline sync
4. **Payments Integration** - Begin mobile money API integration planning
5. **UX/UI Designer** - Create wireframes for MVP screens
6. **DevOps** - Set up initial infrastructure
7. **QA** - Prepare test cases for MVP features
