# QA/Test Agent - Test Strategy & Quality Assurance

## 1. Test Scenarios by Feature

### Authentication & Onboarding

| Test ID | Scenario | Steps | Expected Result | Priority |
|---------|----------|-------|-----------------|----------|
| AUTH001 | Successful registration with valid phone | Enter valid phone → Receive OTP → Enter correct OTP → Set PIN | User account created, logged in | P0 |
| AUTH002 | Registration with invalid phone format | Enter malformed phone → Submit | Error: "Invalid phone number format" | P0 |
| AUTH003 | OTP expiration | Request OTP → Wait 5 min → Enter OTP | Error: "OTP expired, please request new" | P1 |
| AUTH004 | Wrong OTP entry | Enter incorrect OTP 3 times | Account temporarily locked, retry after 15 min | P1 |
| AUTH005 | PIN setup with weak PIN (1234) | Try to set PIN as 1234 | Error: "PIN too simple, choose another" | P2 |
| AUTH006 | Login with correct PIN | Enter registered phone → Enter correct PIN | Logged in successfully | P0 |
| AUTH007 | Login with wrong PIN | Enter wrong PIN 5 times | Account locked, contact support | P1 |
| AUTH008 | Biometric login (fingerprint) | Enable biometric → Lock app → Use fingerprint | Logged in without PIN | P2 |

### P2P Payments

| Test ID | Scenario | Steps | Expected Result | Priority |
|---------|----------|-------|-----------------|----------|
| P2P001 | Send money to existing contact | Select contact → Enter amount → Confirm with PIN | Transaction completed, balance updated | P0 |
| P2P002 | Send money to new phone number | Enter new number → Validate → Enter amount → Confirm | Transaction completed, recipient receives notification | P0 |
| P2P003 | Send money with insufficient balance | Enter amount > balance → Confirm | Error: "Insufficient funds" | P0 |
| P2P004 | Send money with wrong PIN | Enter wrong PIN at confirmation | Transaction cancelled, balance unchanged | P0 |
| P2P005 | Duplicate transaction prevention | Submit same transaction twice rapidly | Second request rejected as duplicate | P0 |
| P2P006 | Send money offline | Disable network → Create transaction → Reconnect | Transaction queued, synced when online | P0 |
| P2P007 | Transaction limit exceeded | Enter amount > daily limit | Error: "Exceeds daily transaction limit" | P1 |
| P2P008 | Send to non-registered user | Enter phone not in system | Transaction succeeds, user prompted to register on first login | P1 |
| P2P009 | Cancel transaction before confirmation | Start flow → Tap back/cancel at confirmation | No transaction created | P2 |
| P2P010 | Receipt generation | Complete transaction → View receipt | Receipt shows all details, shareable via SMS/WhatsApp | P2 |

### Mobile Money Integration

| Test ID | Scenario | Steps | Expected Result | Priority |
|---------|----------|-------|-----------------|----------|
| MM001 | Cash-in via MTN Mobile Money | Select amount → Confirm → Approve on phone | Wallet credited, transaction recorded | P0 |
| MM002 | Cash-in failure (insufficient MoMo balance) | Initiate top-up → Reject on provider side | Transaction failed, proper error shown | P0 |
| MM003 | Cash-in timeout | Initiate top-up → Don't approve within 5 min | Transaction expired, user notified | P1 |
| MM004 | Cash-out to agent | Select cash-out → Choose agent → Confirm | Agent notified, cash dispensed, wallet debited | P0 |
| MM005 | Cash-out with insufficient agent float | Select amount > agent float | Error: "Agent has insufficient cash" | P1 |
| MM006 | Webhook delayed delivery | Complete MoMo transaction → Simulate delayed webhook | System polls and updates status correctly | P1 |
| MM007 | Duplicate webhook handling | Send same webhook twice | Second webhook ignored, no double credit | P0 |
| MM008 | Provider API downtime | Initiate transaction during provider outage | Graceful error, retry suggested | P1 |

### Bill Payments

| Test ID | Scenario | Steps | Expected Result | Priority |
|---------|----------|-------|-----------------|----------|
| BILL001 | Pay electricity bill (UMEME) | Select UMEME → Enter meter number → Validate → Pay | Bill paid, receipt generated | P0 |
| BILL002 | Invalid meter number | Enter non-existent meter → Validate | Error: "Invalid meter number" | P0 |
| BILL003 | Bill payment with insufficient balance | Enter amount > balance | Error: "Insufficient funds" | P0 |
| BILL004 | Save bill account for later | Pay bill → Check "Save account" | Account appears in saved accounts | P2 |
| BILL005 | Pay saved bill quickly | Select saved account → Enter amount → Pay | Payment completed in 2 taps | P2 |
| BILL006 | Bill payment offline | Disable network → Create payment → Reconnect | Queued, synced when online | P1 |
| BILL007 | Partial bill payment | Enter amount < bill total | Payment accepted, remaining balance shown | P2 |

### Offline Sync

| Test ID | Scenario | Steps | Expected Result | Priority |
|---------|----------|-------|-----------------|----------|
| SYNC001 | Transaction queued offline | Create P2P while offline | Shows as pending, syncs when online | P0 |
| SYNC002 | Multiple offline transactions | Create 5 transactions offline → Reconnect | All synced in correct order | P0 |
| SYNC003 | Conflict resolution (balance mismatch) | Spend offline → Receive money online → Sync | Server balance wins, local adjusted | P0 |
| SYNC004 | Offline transaction fails on sync | Queue transaction → Sync fails (e.g., insufficient funds now) | Transaction marked failed, user notified | P0 |
| SYNC005 | Large sync queue (>100 transactions) | Create many offline transactions → Sync | All processed, UI remains responsive | P2 |
| SYNC006 | Background sync | App in background → Network restored | Sync completes, notification sent | P1 |
| SYNC007 | Manual sync trigger | Pull down on home screen → Force sync | Sync initiated immediately | P2 |

---

## 2. Edge Cases

### Financial Edge Cases

| Edge Case | Description | Handling |
|-----------|-------------|----------|
| Race condition on balance | Two transactions simultaneously debiting same wallet | Optimistic locking with version check |
| Decimal precision | Amounts with fractional currency units | Store as integers (smallest unit) |
| Currency conversion | Cross-border payments with different currencies | Use fixed exchange rate at transaction time |
| Negative amounts | Attempt to send negative money | Validation rejects before processing |
| Zero amount transactions | Send 0 UGX | Validation rejects minimum amount (e.g., 100 UGX) |
| Very large amounts | Send 1 billion UGX | Enforce per-transaction and daily limits |
| Rounding errors | Calculate fees with decimals | Round down for customer, round up for fees |

### Network Edge Cases

| Edge Case | Description | Handling |
|-----------|-------------|----------|
| Network drops mid-transaction | Connection lost during API call | Retry with idempotency key |
| Slow network (2G) | High latency, packet loss | Timeout after 30s, queue for retry |
| Intermittent connectivity | Network flipping on/off | Exponential backoff retry |
| DNS resolution failure | Can't resolve API domain | Show offline mode, use cached data |
| SSL certificate expiry | Provider certificate expired | Fail securely, alert ops team |

### Device Edge Cases

| Edge Case | Description | Handling |
|-----------|-------------|----------|
| Low storage space | Device has <100MB free | Clear old cache, warn user |
| Old Android version | Android 5.0 (API 21) | Graceful degradation, disable biometrics |
| Small screen | 480x800 resolution | Responsive layout, scrollable content |
| Low RAM | 512MB-1GB RAM | Limit background processes, lazy load |
| No GPS | Device without location services | Manual agent selection from list |
| Rooted device | Security-compromised device | Warn user, potentially block high-value transactions |

### Time-Based Edge Cases

| Edge Case | Description | Handling |
|-----------|-------------|----------|
| Midnight crossover | Transaction spans two days | Timestamp stored in UTC, display in local time |
| Leap year | February 29th transactions | Standard date handling |
| Daylight saving | Not applicable in most African countries | N/A for MVP markets |
| Session timeout | User inactive for 15 minutes | Require re-authentication |
| Token expiry | JWT token expired during long session | Silent refresh or re-login |

---

## 3. Validation Checklist

### Pre-Release Checklist

#### Functional Testing
- [ ] All P0 test cases pass
- [ ] At least 95% of P1 test cases pass
- [ ] No critical or high severity bugs open
- [ ] Payment flows work end-to-end
- [ ] Offline mode functions correctly
- [ ] All three mobile money providers tested

#### Security Testing
- [ ] PIN is never logged or transmitted in plain text
- [ ] API authentication working (JWT validation)
- [ ] Rate limiting prevents brute force attacks
- [ ] SQL injection tests pass
- [ ] XSS tests pass (for any web views)
- [ ] Sensitive data encrypted at rest
- [ ] Webhook signatures verified

#### Performance Testing
- [ ] API response time <500ms (p95)
- [ ] App cold start <3 seconds on low-end device
- [ ] Screen transitions <300ms
- [ ] Database queries optimized (no N+1)
- [ ] Load testing: 100 concurrent users handled
- [ ] Memory usage <100MB during normal operation

#### Compatibility Testing
- [ ] Tested on Android 5.0, 6.0, 8.0, 10.0, 12.0
- [ ] Tested on screens: 480p, 720p, 1080p
- [ ] Tested on devices: Tecno, Infinix, Samsung (low-end)
- [ ] Tested in portrait and landscape modes
- [ ] Tested with font size: small, medium, large

#### Localization Testing
- [ ] English language complete
- [ ] Swahili translations accurate
- [ ] French translations accurate
- [ ] Date formats correct (DD/MM/YYYY)
- [ ] Currency formatting correct (UGX 50,000)
- [ ] Phone number formatting correct

#### Accessibility Testing
- [ ] TalkBack navigation works
- [ ] Minimum touch target 48x48dp
- [ ] Color contrast meets WCAG AA
- [ ] No information conveyed by color alone

#### Operational Readiness
- [ ] Monitoring dashboards configured
- [ ] Alerts set up for critical metrics
- [ ] Runbooks documented for common issues
- [ ] Backup and restore tested
- [ ] Rollback procedure documented
- [ ] Support team trained

---

## 4. Test Automation Strategy

### Unit Tests (Rust Backend)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_phone_number_uganda() {
        assert!(validate_phone("+256701234567", "UG").is_ok());
        assert!(validate_phone("256701234567", "UG").is_ok());
        assert!(validate_phone("0701234567", "UG").is_ok());
        assert!(validate_phone("+256123456789", "UG").is_err()); // Invalid prefix
    }

    #[test]
    fn test_insufficient_funds() {
        let mut wallet = Wallet { balance: 10000, .. };
        let result = wallet.debit(15000);
        assert!(result.is_err());
        assert_eq!(wallet.balance, 10000); // Balance unchanged
    }

    #[test]
    fn test_idempotent_transaction() {
        let tx1 = create_transaction("offline_123", 5000);
        let tx2 = create_transaction("offline_123", 5000); // Same ID
        
        assert_eq!(tx1.id, tx2.id); // Should return same transaction
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_p2p_payment() {
    // Setup
    let sender = create_test_user().await;
    let recipient = create_test_user().await;
    
    // Fund sender wallet
    fund_wallet(sender.id, 100000).await;
    
    // Execute payment
    let response = app
        .post("/api/v1/payments/p2p")
        .json(&json!({
            "sender_id": sender.id,
            "recipient_phone": recipient.phone,
            "amount": 50000,
            "pin": "1234",
            "offline_tx_id": "test_123"
        }))
        .send()
        .await;
    
    // Assert
    assert_eq!(response.status(), 200);
    let sender_balance = get_wallet_balance(sender.id).await;
    let recipient_balance = get_wallet_balance(recipient.id).await;
    assert_eq!(sender_balance, 50000);
    assert_eq!(recipient_balance, 50000);
}
```

### Mobile UI Tests (Flutter Integration Tests)

```dart
void main() {
  group('P2P Payment Flow', () {
    testWidgets('Successful payment to contact', (tester) async {
      await tester.pumpWidget(MyApp());
      await tester.pumpAndSettle();
      
      // Login
      await tester.enterText(find.byKey(Key('phone_input')), '0701234567');
      await tester.tap(find.text('Continue'));
      await tester.pumpAndSettle();
      
      // Enter PIN
      await enterPin(tester, '1234');
      await tester.pumpAndSettle();
      
      // Navigate to send money
      await tester.tap(find.text('Send'));
      await tester.pumpAndSettle();
      
      // Select contact
      await tester.tap(find.text('Mary'));
      await tester.pumpAndSettle();
      
      // Enter amount
      await tester.enterText(find.byKey(Key('amount_input')), '50000');
      await tester.tap(find.text('Continue'));
      await tester.pumpAndSettle();
      
      // Confirm with PIN
      await enterPin(tester, '1234');
      await tester.pumpAndSettle();
      
      // Verify success
      expect(find.text('Success!'), findsOneWidget);
      expect(find.text('UGX 50,000'), findsOneWidget);
    });
  });
}
```

---

## 5. Next Steps for QA Team

1. Set up test environments (dev, staging, production-mirror)
2. Create test data generator for realistic scenarios
3. Implement automated regression test suite
4. Set up CI integration for test execution
5. Perform security penetration testing
6. Conduct load testing with realistic traffic patterns
7. Organize beta testing program with real users
8. Document known issues and workarounds
9. Create bug triage process with severity definitions
10. Establish quality gates for each release phase
