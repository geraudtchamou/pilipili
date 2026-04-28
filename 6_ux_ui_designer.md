# UX/UI Designer Agent - Low-Literacy User Design

## 1. Design Principles for Low-Tech Users

### Core Principles

| Principle | Description | Implementation |
|-----------|-------------|----------------|
| **Minimal Text** | Reduce reading requirements | Use icons, colors, numbers |
| **Consistent Layout** | Same pattern on every screen | Header → Content → Action button (bottom) |
| **Large Touch Targets** | Easy to tap accurately | Minimum 48x48dp buttons |
| **Clear Feedback** | Immediate response to actions | Colors, sounds, vibrations |
| **Forgiving** | Easy to undo mistakes | Confirmation screens, cancel options |
| **Progressive Disclosure** | Show only what's needed | One task per screen |
| **Universal Symbols** | Recognizable across cultures | Money bag, phone, home icons |

### Color System

```
Primary Actions:    Green (#2E7D32) - Send, Pay, Confirm
Secondary Actions:  Blue (#1976D2) - View Details, Learn More
Destructive:        Red (#D32F2F) - Cancel, Delete
Success:            Green (#4CAF50) - Completed transactions
Warning:            Amber (#FFA000) - Pending, low balance
Error:              Red (#F44336) - Failed transactions

Background:         White (#FFFFFF) / Light Gray (#F5F5F5)
Text Primary:       Black (#212121)
Text Secondary:     Gray (#757575)
```

### Typography

```
Headings:   24sp, Bold, Roboto
Body:       16sp, Regular, Roboto
Buttons:    16sp, Medium, Roboto (all caps for primary)
Numbers:    32sp, Bold (for amounts)
Small:      14sp, Regular (for hints)
```

---

## 2. Screen Descriptions

### SCR001 - Welcome Screen
```
┌─────────────────────────────┐
│                             │
│         [Logo]              │
│      PayFlow                │
│                             │
│   ┌───────────────────┐     │
│   │  📱 Send Money    │     │
│   │  💰 Check Balance │     │
│   │  📄 Pay Bills     │     │
│   └───────────────────┘     │
│                             │
│   Safe • Fast • Easy        │
│                             │
│     [ Get Started ]         │
│                             │
└─────────────────────────────┘
```
- Large app logo at top
- Three key features with icons
- Single green CTA button
- No text input required

### SCR003 - Phone Input Screen
```
┌─────────────────────────────┐
│  ← Back                     │
│                             │
│   Enter your phone number   │
│                             │
│   ┌─────────────────────┐   │
│   │ 🇺🇬 +256            │   │
│   │ 7__ ___ ___         │   │
│   └─────────────────────┘   │
│                             │
│   [1] [2] [3]               │
│   [4] [5] [6]               │
│   [7] [8] [9]               │
│   [*] [0] [#] [⌫]           │
│                             │
│     [ Continue ]            │
│                             │
└─────────────────────────────┘
```
- Custom numeric keypad (large buttons)
- Country flag and code pre-filled
- Auto-formatting as user types
- Backspace button clearly visible

### SCR010 - Home Screen (Main Dashboard)
```
┌─────────────────────────────┐
│  ☰              🔔  👤     │
│                             │
│   Good morning, John        │
│                             │
│   ┌─────────────────────┐   │
│   │  Balance            │   │
│   │  UGX 250,000        │   │
│   │  👁️ Show/Hide       │   │
│   └─────────────────────┘   │
│                             │
│   Quick Actions             │
│   ┌─────┐ ┌─────┐ ┌─────┐   │
│   │ ➤   │ │ 📄  │ │ 🏧  │   │
│   │Send │ │Bills│ │Cash │   │
│   └─────┘ └─────┘ └─────┘   │
│                             │
│   Recent Activity           │
│   ┌─────────────────────┐   │
│   │ ➤ Mary    -50,000   │   │
│   │   Today 3:30 PM     │   │
│   ├─────────────────────┤   │
│   │ 📄 UMEME   -30,000  │   │
│   │   Yesterday         │   │
│   └─────────────────────┘   │
│                             │
│   [View All Transactions]   │
└─────────────────────────────┘
```
- Balance prominently displayed
- Hide/show balance toggle
- 3 large quick action buttons
- Last 2 transactions visible
- Bottom navigation bar (not shown)

### SCR011 - Send Money Flow

#### Step 1: Choose Recipient
```
┌─────────────────────────────┐
│  ← Send Money               │
│                             │
│   Who to send?              │
│                             │
│   ┌─────────────────────┐   │
│   │ 🔍 Search name...   │   │
│   └─────────────────────┘   │
│                             │
│   Favorites                 │
│   ┌─────┐ ┌─────┐ ┌─────┐   │
│   │ 👩  │ │ 👨  │ │ ➕  │   │
│   │Mom  │ │Dad  │ │New  │   │
│   └─────┘ └─────┘ └─────┘   │
│                             │
│   Recent                    │
│   ○ Mary    +256 701...     │
│   ○ John    +256 772...     │
│   ○ Shop    +256 758...     │
│                             │
│   [ Enter Number ]          │
└─────────────────────────────┘
```

#### Step 2: Enter Amount
```
┌─────────────────────────────┐
│  ← Send to Mary             │
│                             │
│   How much?                 │
│                             │
│   UGX                       │
│   ┌─────────────────────┐   │
│   │      50,000         │   │
│   └─────────────────────┘   │
│                             │
│   Quick Amounts             │
│   [10K] [20K] [50K] [100K]  │
│                             │
│   Available: UGX 250,000    │
│                             │
│   [1] [2] [3]               │
│   [4] [5] [6]               │
│   [7] [8] [9]               │
│   [.] [0] [⌫]               │
│                             │
│     [ Continue ]            │
└─────────────────────────────┘
```

#### Step 3: Confirm
```
┌─────────────────────────────┐
│  ← Confirm                  │
│                             │
│   ✓ Review details          │
│                             │
│   ┌─────────────────────┐   │
│   │  To: Mary Namaganda │   │
│   │  📱 +256 701 234567 │   │
│   │                     │   │
│   │  Amount: UGX 50,000 │   │
│   │  Fee: UGX 0         │   │
│   │  ─────────────────  │   │
│   │  Total: UGX 50,000  │   │
│   └─────────────────────┘   │
│                             │
│   Enter your PIN            │
│   ○ ○ ○ ○                   │
│                             │
│   [1] [2] [3]               │
│   [4] [5] [6]               │
│   [7] [8] [9]               │
│       [0] [⌫]               │
│                             │
│     [ Confirm Payment ]     │
└─────────────────────────────┘
```

#### Step 4: Result
```
┌─────────────────────────────┐
│                             │
│         ✓                   │
│      Success!               │
│                             │
│   Sent to Mary              │
│   UGX 50,000                │
│                             │
│   Transaction ID:           │
│   TXN123456789              │
│                             │
│   Balance: UGX 200,000      │
│                             │
│   ┌─────────┐ ┌─────────┐   │
│   │ 📤 Share│ │ 🖨️Receipt│   │
│   └─────────┘ └─────────┘   │
│                             │
│     [ Done ]                │
│                             │
└─────────────────────────────┘
```

---

## 3. UX Flows

### Flow 1: First-Time User Onboarding
```
[Download App]
      ↓
[Splash Screen] (2 seconds)
      ↓
[Welcome Screen] → Tap "Get Started"
      ↓
[Phone Input] → Enter number → Tap "Continue"
      ↓
[OTP Verification] → Auto-read or enter → Verify
      ↓
[PIN Setup] → Enter 4 digits → Confirm PIN
      ↓
[Permissions] → Request SMS (optional)
      ↓
[Home Screen] → Tutorial overlay (skipable)
      ↓
[Ready to use]
```

### Flow 2: Send Money (Online)
```
[Home] → Tap "Send"
      ↓
[Choose Recipient] → Select from list or enter number
      ↓
[Enter Amount] → Type amount or select quick amount
      ↓
[Confirm] → Review details → Enter PIN
      ↓
[Processing] → Show spinner (max 5 seconds)
      ↓
[Success/Failure] → Show result with receipt
      ↓
[Home] or [Share Receipt]
```

### Flow 3: Send Money (Offline)
```
[Home] → Tap "Send" (offline banner visible)
      ↓
[Choose Recipient] → From cached contacts only
      ↓
[Enter Amount] → Type amount
      ↓
[Confirm] → Review → Enter PIN
      ↓
[Queued] → "Transaction saved. Will send when online."
      ↓
[Home] → Transaction shows as "Pending"
      ↓
[Auto-sync when online] → Push to server
      ↓
[Notification] → SMS/push when completed
```

### Flow 4: Pay Bill
```
[Home] → Tap "Bills"
      ↓
[Select Category] → Electricity, Water, Internet
      ↓
[Select Provider] → UMEME, NWSC, etc.
      ↓
[Enter Account] → Meter/account number
      ↓
[Validate] → Show account name for confirmation
      ↓
[Enter Amount] → Or suggested amount
      ↓
[Confirm] → Review → Enter PIN
      ↓
[Success] → Receipt with reference number
```

---

## 4. Iconography Guide

### Primary Icons (Filled style, 24x24dp)

| Icon | Meaning | Usage |
|------|---------|-------|
| ➤ / ↗ | Send | P2P payments |
| 📄 / 📋 | Bills | Bill payments |
| 🏧 / 💵 | Cash | Cash in/out |
| 👤 | Profile | User account |
| 🔔 | Notifications | Alerts |
| ⚙️ | Settings | Configuration |
| 🔍 | Search | Find contacts |
| ➕ | Add | New recipient |
| ✓ | Success | Completed |
| ✗ | Error/Cancel | Failed/Close |
| 👁️ | View/Hide | Toggle visibility |
| 📱 | Phone | Contact method |
| 🏠 | Home | Main screen |
| 💰 | Money/Wallet | Balance |
| 📊 | History/Stats | Transactions |
| ❓ | Help | Support |
| 🌐 | Language | Locale switch |
| 📶 | Connection | Network status |

### Status Indicators

| Visual | Meaning |
|--------|---------|
| 🟢 Green circle | Online, connected |
| 🟡 Yellow circle | Pending, syncing |
| 🔴 Red circle | Offline, error |
| ✓ Green check | Success |
| ⏳ Hourglass | Processing |
| 🔒 Lock | Secure, PIN required |

---

## 5. Accessibility Considerations

### For Low Vision Users
- Minimum font size: 16sp body, 24sp headings
- High contrast ratios (WCAG AA minimum)
- Option to increase text size in settings
- VoiceOver/TalkBack labels on all elements

### For Low Literacy Users
- Consistent icon + label pairing
- Color-coded actions (green = go, red = stop)
- Minimal form fields
- Progressive disclosure of complexity

### For Motor Impairments
- Large touch targets (48x48dp minimum)
- Generous spacing between interactive elements
- No time-limited interactions
- Voice input support where possible

---

## 6. Localization Strategy

### Supported Languages (MVP)
1. English (default)
2. Swahili
3. French (for Francophone countries)

### Implementation
```dart
// Example localization keys
{
  "home.balance": "Balance",
  "home.balance.sw": "Salio",
  "home.balance.fr": "Solde",
  
  "send.button": "Send",
  "send.button.sw": "Tuma",
  "send.button.fr": "Envoyer",
  
  "success.message": "Payment successful!",
  "success.message.sw": "Umalipo umefanikiwa!",
  "success.message.fr": "Paiement réussi!",
}
```

### Cultural Considerations
- Right-to-left support not needed (no Arabic in MVP)
- Date format: DD/MM/YYYY
- Currency format: UGX 50,000 (symbol before, commas for thousands)
- Phone format: +256 701 234 567 (grouped for readability)

---

## 7. Next Steps for Design Team

1. Create high-fidelity mockups in Figma
2. Build interactive prototype for user testing
3. Conduct usability testing with target users
4. Iterate based on feedback
5. Create design system component library
6. Export assets for mobile team
7. Define animation specifications
8. Prepare handoff documentation
