# Mobile App Engineer Agent - Flutter Android App

## 1. Screen List

### Authentication Screens
| Screen ID | Name | Purpose |
|-----------|------|---------|
| SCR001 | SplashScreen | App logo, version check |
| SCR002 | WelcomeScreen | Value prop, get started button |
| SCR003 | PhoneInputScreen | Enter phone number |
| SCR004 | OTPVerificationScreen | Enter SMS OTP code |
| SCR005 | PINSetupScreen | Create 4-digit transaction PIN |
| SCR006 | PINLoginScreen | Enter PIN to access app |

### Main App Screens
| Screen ID | Name | Purpose |
|-----------|------|---------|
| SCR010 | HomeScreen | Dashboard, balance, quick actions |
| SCR011 | SendMoneyScreen | P2P payment flow |
| SCR012 | RecipientSelectScreen | Choose from contacts or enter phone |
| SCR013 | AmountInputScreen | Enter amount with currency |
| SCR014 | ConfirmPaymentScreen | Review and confirm with PIN |
| SCR015 | PaymentResultScreen | Success/failure receipt |

### Wallet Screens
| Screen ID | Name | Purpose |
|-----------|------|---------|
| SCR020 | WalletScreen | View balance, transaction history |
| SCR021 | CashInScreen | Mobile money top-up flow |
| SCR022 | CashOutScreen | Withdraw to agent flow |
| SCR023 | TransactionDetailScreen | Full transaction details |

### Bill Payment Screens
| Screen ID | Name | Purpose |
|-----------|------|---------|
| SCR030 | BillPaymentsScreen | List of bill categories |
| SCR031 | BillProviderSelectScreen | Choose provider (UMEME, etc.) |
| SCR032 | BillAccountInputScreen | Enter account/meter number |
| SCR033 | BillValidateScreen | Show account name for confirmation |
| SCR034 | BillPayScreen | Enter amount and pay |

### Agent Screens
| Screen ID | Name | Purpose |
|-----------|------|---------|
| SCR040 | FindAgentScreen | Map/list of nearby agents |
| SCR041 | AgentDetailScreen | Agent info, directions, contact |

### Settings & Profile
| Screen ID | Name | Purpose |
|-----------|------|---------|
| SCR050 | ProfileScreen | View/edit user info |
| SCR051 | SettingsScreen | Language, notifications, security |
| SCR052 | HelpScreen | FAQ, contact support |
| SCR053 | AboutScreen | App version, terms, privacy |

### Offline States
| Screen ID | Name | Purpose |
|-----------|------|---------|
| SCR090 | OfflineBanner | Shows when no connection |
| SCR091 | PendingTransactionsScreen | View queued offline transactions |
| SCR092 | SyncStatusScreen | Manual sync trigger and status |

---

## 2. Navigation Flow

```
SplashScreen
    ↓
WelcomeScreen (first time) OR HomeScreen (logged in)
    ↓
PhoneInputScreen → OTPVerificationScreen → PINSetupScreen
    ↓
HomeScreen (Main Tab Navigator)
    ├── Tab 1: Home
    │   └── SendMoneyScreen → RecipientSelectScreen → AmountInputScreen 
    │       → ConfirmPaymentScreen → PaymentResultScreen
    │
    ├── Tab 2: Wallet
    │   ├── CashInScreen → ConfirmPaymentScreen → PaymentResultScreen
    │   ├── CashOutScreen → ConfirmPaymentScreen → PaymentResultScreen
    │   └── TransactionDetailScreen
    │
    ├── Tab 3: Bills
    │   ├── BillProviderSelectScreen → BillAccountInputScreen 
    │   │   → BillValidateScreen → BillPayScreen → PaymentResultScreen
    │   └── TransactionDetailScreen
    │
    ├── Tab 4: Agents
    │   └── FindAgentScreen → AgentDetailScreen
    │
    └── Tab 5: Profile
        ├── ProfileScreen
        ├── SettingsScreen
        └── HelpScreen
```

### Navigation Principles
- Deep linking support for payment requests
- Each screen can be accessed offline (cached data)
- Back button preserves form state
- Biometric authentication option for PIN entry

---

## 3. Local Data Model (SQLite with drift)

### User Entity
```dart
@Table(name: 'users')
class User {
  @PrimaryKey()
  final String id;
  final String phone;
  final String? firstName;
  final String? lastName;
  final int kycLevel;
  final DateTime createdAt;
  final DateTime? lastSyncAt;
}
```

### Wallet Entity
```dart
@Table(name: 'wallets')
class Wallet {
  @PrimaryKey()
  final String userId;
  final int balance; // in smallest currency unit
  final String currency;
  final int version;
  final DateTime updatedAt;
}
```

### Transaction Entity
```dart
@Table(name: 'transactions')
class Transaction {
  @PrimaryKey()
  final String id;
  final String? offlineTxId; // Client-generated for sync
  final String userId;
  final String type; // p2p_send, p2p_receive, topup, withdrawal, bill
  final int amount;
  final String currency;
  final String status; // pending, completed, failed, syncing
  final String? recipientId;
  final String? recipientPhone;
  final String? recipientName;
  final String? description;
  final String? providerReference;
  final String? errorCode;
  final String? errorMessage;
  final Map<String, dynamic>? metadata;
  final DateTime createdAt;
  final DateTime? processedAt;
  final int retryCount;
}
```

### Pending Transaction Queue
```dart
@Table(name: 'pending_transactions')
class PendingTransaction {
  @PrimaryKey(autoGenerate: true)
  final int id;
  final String offlineTxId;
  final String type;
  final Map<String, dynamic> payload;
  final int priority; // 1=highest
  final int retryCount;
  final DateTime createdAt;
  final DateTime? lastAttemptAt;
  final String? lastError;
}
```

### Bill Provider Cache
```dart
@Table(name: 'bill_providers')
class BillProvider {
  @PrimaryKey()
  final String id;
  final String name;
  final String category;
  final String countryCode;
  final Map<String, dynamic> config;
  final DateTime cachedAt;
}
```

### Agent Cache
```dart
@Table(name: 'agents')
class Agent {
  @PrimaryKey()
  final String id;
  final String userId;
  final String businessName;
  final double latitude;
  final double longitude;
  final bool isVerified;
  final DateTime cachedAt;
}
```

### App Settings
```dart
@Table(name: 'app_settings')
class AppSettings {
  @PrimaryKey()
  final String key; // 'language', 'last_sync', 'onboarding_complete'
  final String value;
  final DateTime updatedAt;
}
```

---

## 4. Sync Logic

### Sync Manager Architecture

```dart
class SyncManager {
  final Database db;
  final ApiClient api;
  final Connectivity connectivity;
  
  // Called when connectivity changes
  Future<void> onConnectivityChanged(bool isConnected) async {
    if (isConnected) {
      await processPendingQueue();
      await pullServerUpdates();
    }
  }
  
  // Process locally queued transactions
  Future<void> processPendingQueue() async {
    final pending = await db.pendingTransactions.getAllOrderedByPriority();
    
    for (final tx in pending) {
      if (tx.retryCount >= 3) {
        await markAsFailed(tx, 'Max retries exceeded');
        continue;
      }
      
      try {
        final result = await api.post('/sync/push', {
          'transactions': [tx.payload],
        });
        
        if (result['status'] == 'success') {
          // Move to permanent transactions table
          await db.transaction((db) async {
            await db.transactions.insert(Transaction.fromPayload(tx.payload));
            await db.pendingTransactions.delete(tx.id);
          });
        }
      } catch (e) {
        await db.pendingTransactions.incrementRetry(tx.id);
      }
    }
  }
  
  // Pull latest server state
  Future<void> pullServerUpdates() async {
    final lastSync = await db.appSettings.get('last_sync');
    
    final response = await api.get(
      '/sync/pull',
      queryParameters: {'since': lastSync?.value ?? '1970-01-01'},
    );
    
    await db.transaction((db) async {
      // Update wallet balances
      for (final walletData in response['wallets']) {
        await db.wallets.upsert(Wallet.fromJson(walletData));
      }
      
      // Insert new transactions
      for (final txData in response['transactions']) {
        await db.transactions.insert(Transaction.fromJson(txData));
      }
      
      // Update settings
      await db.appSettings.upsert(
        AppSettings(key: 'last_sync', value: response['timestamp']),
      );
    });
  }
  
  // Queue a new transaction for later sync
  Future<String> queueTransaction(Map<String, dynamic> payload) async {
    final offlineTxId = generateOfflineTxId();
    
    await db.pendingTransactions.insert(PendingTransaction(
      offlineTxId: offlineTxId,
      type: payload['type'],
      payload: {
        ...payload,
        'offline_tx_id': offlineTxId,
        'timestamp': DateTime.now().toIso8601String(),
      },
      priority: _calculatePriority(payload['type']),
    ));
    
    // Try immediate sync if online
    if (await connectivity.isConnected) {
      processPendingQueue();
    }
    
    return offlineTxId;
  }
  
  int _calculatePriority(String type) {
    switch (type) {
      case 'p2p':
        return 1; // Highest priority
      case 'bill':
        return 2;
      case 'topup':
        return 2;
      default:
        return 5;
    }
  }
}
```

### Offline-First Patterns

#### 1. Optimistic UI Updates
```dart
Future<void> sendMoney({
  required String recipientPhone,
  required int amount,
  required String pin,
}) async {
  final offlineTxId = await syncManager.queueTransaction({
    'type': 'p2p',
    'recipient_phone': recipientPhone,
    'amount': amount,
    'pin': pin,
  });
  
  // Immediately show pending state in UI
  navigator.push(PaymentResultScreen(
    transactionId: offlineTxId,
    status: 'pending',
    message: 'Transaction queued. Will send when online.',
  ));
  
  // Update local balance optimistically
  final currentBalance = await db.wallets.getBalance();
  await db.wallets.updateBalance(currentBalance - amount);
}
```

#### 2. Conflict Resolution
```dart
Future<void> resolveConflict(Transaction local, Transaction server) async {
  // Server always wins for financial transactions
  if (local.type == 'p2p' || local.type == 'bill') {
    if (server.status == 'failed') {
      // Refund the optimistic balance update
      await db.wallets.updateBalance(
        (await db.wallets.getBalance()) + local.amount,
      );
    }
    // Replace local record with server truth
    await db.transactions.upsert(server);
  }
}
```

#### 3. Background Sync
```dart
@pragma('vm:entry-point')
Future<void> backgroundSyncHeadlessTask(HeadlessTask task) async {
  final syncManager = Injector.instance.get<SyncManager>();
  
  if (task.timeout) {
    return;
  }
  
  await syncManager.processPendingQueue();
  await syncManager.pullServerUpdates();
}

// Register with WorkManager
Workmanager().registerPeriodicTask(
  'com.fintech.sync',
  'background_sync',
  frequency: Duration(minutes: 15),
  constraints: Constraints(
    networkType: NetworkType.connected,
    requiresBatteryNotLow: false,
  ),
);
```

---

## 5. Low-Bandwidth Optimizations

### Data Compression
```dart
class CompressedApiClient extends ApiClient {
  @override
  Future<Response> post(String path, Map<String, dynamic> data) async {
    final compressed = gzip.encode(jsonEncode(data).codeUnits);
    
    return await http.post(
      Uri.parse('$baseUrl$path'),
      headers: {
        'Content-Type': 'application/octet-stream',
        'Content-Encoding': 'gzip',
      },
      body: compressed,
    );
  }
}
```

### Selective Sync
```dart
Future<Map<String, dynamic>> getSyncPayload(DateTime since) async {
  // Only sync essential fields
  final transactions = await db.transactions
    .filter((t) => t.createdAt.isAfter(since))
    .select((t) => {
      t.id,
      t.offlineTxId,
      t.type,
      t.amount,
      t.status,
      t.recipientPhone,
      t.createdAt,
    })
    .get();
  
  return {
    'transactions': transactions,
    'minimal': true, // Server knows to return minimal response
  };
}
```

### Image Handling
```dart
// No images in MVP - use colored icons only
// If images needed later:
CachedNetworkImage(
  imageUrl: url,
  maxWidthDiskCache: 100,
  maxHeightDiskCache: 100,
  memCacheWidth: 50,
  memCacheHeight: 50,
  placeholder: (context, url) => Icon(Icons.person, size: 50),
  errorWidget: (context, url, error) => Icon(Icons.error, size: 50),
);
```

---

## 6. Tech Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Framework | Flutter 3.x | Cross-platform, Android-first |
| State Management | Riverpod | Simple, testable, efficient |
| Local DB | drift (SQLite) | Type-safe, offline-first ready |
| HTTP Client | dio | Interceptors, compression, retry |
| Secure Storage | flutter_secure_storage | PIN, tokens encrypted |
| Biometrics | local_auth | Fingerprint/Face ID support |
| Maps | flutter_map + OSM | Free, works offline with cached tiles |
| Background Tasks | workmanager | Periodic sync on Android |
| Analytics | firebase_analytics (optional) | Usage tracking |
| Crash Reporting | sentry_flutter | Error monitoring |

---

## 7. Next Steps for Mobile Team

1. Initialize Flutter project with Android-first configuration
2. Set up folder structure (features-based architecture)
3. Implement database layer with drift
4. Build authentication flow screens
5. Create sync manager service
6. Implement home screen with balance display
7. Build P2P payment flow
8. Add offline state handling
9. Test on low-end Android devices (1GB RAM)
10. Optimize APK size (target <15MB)
