# Universal Business Ledger 2.0 - Architecture

## The Vision: Trust as Infrastructure

This system separates **meaning** from **proof**. The Mind (TypeScript) handles semantics; the Body (Rust) handles cryptography. Together, they create **trustworthy systems**.

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                      UNIVERSAL BUSINESS LEDGER 2.0                            ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║   Mind (TS)  ───▶  TDLN  ───▶  Body (Rust)  ───▶  Ledger  ───▶  Proof       ║
║      │              │              │               │              │           ║
║      │              │              │               │              │           ║
║      ▼              ▼              ▼               ▼              ▼           ║
║   MEANING      TRANSLATION     PHYSICS         HISTORY        TRUTH          ║
║   (Local)     (Deterministic)  (Universal)    (Immutable)   (Verifiable)     ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

## The Two Worlds

### The Body (Rust) — "Doesn't think, only obeys laws"

```
crates/
├── ubl-atom/        # Canonical JSON → Deterministic Bytes
│   └── src/lib.rs   # SPEC-UBL-ATOM v1.0
│
├── ubl-link/        # The Commit Envelope
│   └── src/lib.rs   # SPEC-UBL-LINK v1.0
│
├── ubl-kernel/      # Pure Cryptography
│   └── src/lib.rs   # BLAKE3 + Ed25519
│
├── ubl-ledger/      # Append-Only History
│   └── src/lib.rs   # SPEC-UBL-LEDGER v1.0
│
├── ubl-membrane/    # Where Laws Are Enforced
│   └── src/lib.rs   # SPEC-UBL-MEMBRANE v1.0
│
└── ubl-server/      # HTTP Interface
    └── src/main.rs  # Exposes Body to Mind
```

### The Mind (TypeScript) — "Thinks, but has no physical authority"

```
packages/
└── ubl-cortex/      # The Orchestrator
    └── src/
        └── index.ts # Intent → Atom → Link → Commit
```

## Data Flow: From Intent to Proof

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  STEP 1: INTENT (Mind)                                                     │
│  ═══════════════════════                                                    │
│                                                                             │
│  const intent = {                                                           │
│    type: "payment",                                                         │
│    payload: {                                                               │
│      to: "bob",                                                             │
│      amount: 100,                                                           │
│      reason: "Consulting services"   // ← Semantic, local meaning          │
│    }                                                                        │
│  };                                                                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  STEP 2: CANONICALIZE (Mind → ubl-atom)                                    │
│  ════════════════════════════════════════                                   │
│                                                                             │
│  // Sort keys, remove whitespace, deterministic                            │
│  const canonical = '{"amount":100,"reason":"Consulting services","to":"bob"}';
│                                                                             │
│  // Same input ALWAYS produces same output                                 │
│  // This is the ubl-atom                                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  STEP 3: HASH (Mind → ubl-kernel)                                          │
│  ═════════════════════════════════                                          │
│                                                                             │
│  atom_hash = BLAKE3("ubl:atom\n" + canonical)                              │
│            = "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069"
│                                                                             │
│  // The hash IS the identity of this intent                                │
│  // Anyone with the same intent gets the same hash                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  STEP 4: GET STATE (Mind ← Body)                                           │
│  ═════════════════════════════════                                          │
│                                                                             │
│  GET /state → {                                                             │
│    container_id: "wallet_alice",                                            │
│    sequence: 41,                                                            │
│    last_hash: "abc123...",                                                  │
│    physical_balance: 1000                                                   │
│  }                                                                          │
│                                                                             │
│  // Mind now knows where it is in the causal chain                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  STEP 5: BUILD LINK (Mind → ubl-link)                                      │
│  ══════════════════════════════════════                                     │
│                                                                             │
│  LinkCommit {                                                               │
│    version: 1,                                                              │
│    container_id: "wallet_alice",                                            │
│    expected_sequence: 42,           // Next in chain                        │
│    previous_hash: "abc123...",      // Links to last                        │
│    atom_hash: "7f83b165...",        // What we're committing                │
│    intent_class: Conservation,      // Physical class                       │
│    physics_delta: -100,             // Change in value                      │
│    author_pubkey: "ed25519...",     // Who is committing                    │
│    signature: "..."                 // Proof of authorship                  │
│  }                                                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  STEP 6: VALIDATE (Body → ubl-membrane)                                    │
│  ════════════════════════════════════════                                   │
│                                                                             │
│  The Membrane checks (in < 1ms):                                           │
│                                                                             │
│  V1 ✓ Version = 1                                                          │
│  V2 ✓ Container matches                                                    │
│  V3 ✓ Signature verifies                                                   │
│  V4 ✓ Previous hash matches (causal chain intact)                          │
│  V5 ✓ Sequence is next (no gaps)                                           │
│  V6 ✓ Atom hash is valid format                                            │
│  V7 ✓ Physics holds (balance 1000 - 100 = 900 ≥ 0)                        │
│                                                                             │
│  Decision: ACCEPT                                                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  STEP 7: APPEND (Body → ubl-ledger)                                        │
│  ════════════════════════════════════                                       │
│                                                                             │
│  LedgerEntry {                                                              │
│    sequence: 42,                                                            │
│    entry_hash: BLAKE3("ubl:link\n" + signing_bytes),                       │
│    link: LinkCommit { ... },                                                │
│    timestamp: 1735142400                                                    │
│  }                                                                          │
│                                                                             │
│  // Appended to the chain. IMMUTABLE. FOREVER.                             │
│                                                                             │
│  [E₀]──[E₁]──[E₂]──···──[E₄₁]──[E₄₂] ← NEW                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  STEP 8: RECEIPT (Body → Mind)                                             │
│  ══════════════════════════════                                             │
│                                                                             │
│  {                                                                          │
│    "status": "ACCEPTED",                                                    │
│    "receipt": {                                                             │
│      "entry_hash": "8a3f2b1c...",                                           │
│      "sequence": 42,                                                        │
│      "timestamp": 1735142400,                                               │
│      "container_id": "wallet_alice"                                         │
│    }                                                                        │
│  }                                                                          │
│                                                                             │
│  // This receipt is PROOF that the commit was accepted                     │
│  // The Mind can store this alongside the original intent                  │
│  // Together: meaning + proof = trustworthy record                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Module Details

### ubl-atom — Canonical Serialization

**Purpose:** Transform arbitrary JSON into deterministic bytes.

**Guarantee:** Semantically equal JSONs produce identical bytes.

```rust
// Two ways to express the same thing
let a = json!({"z": 1, "a": 2});
let b = json!({"a": 2, "z": 1});

// Same canonical output
assert_eq!(canonicalize(&a), canonicalize(&b));
// Both produce: {"a":2,"z":1}
```

**Rules:**
- Keys sorted lexicographically (recursive)
- No whitespace
- Arrays preserve order
- Non-finite numbers rejected (NaN, Infinity)

---

### ubl-link — The Commit Envelope

**Purpose:** Define the structure that crosses the Mind/Body boundary.

**The LinkCommit:**

| Field | Type | Purpose |
|-------|------|---------|
| `version` | u8 | Protocol version (must be 1) |
| `container_id` | String | Target container |
| `expected_sequence` | u64 | Causal ordering |
| `previous_hash` | String | Chain link |
| `atom_hash` | String | Content identity |
| `intent_class` | Enum | Physical classification |
| `physics_delta` | i64 | Value change |
| `author_pubkey` | String | Who is committing |
| `signature` | String | Proof of authorship |

---

### ubl-kernel — Pure Cryptography

**Purpose:** Hash and sign. Nothing else.

**Functions:**

```rust
// Hash an atom with domain separation
fn hash_atom(bytes: &[u8]) -> String;
// → BLAKE3("ubl:atom\n" + bytes)

// Hash a link for chaining
fn hash_link(bytes: &[u8]) -> String;
// → BLAKE3("ubl:link\n" + bytes)

// Sign with Ed25519
fn sign(key: &SigningKey, msg: &[u8]) -> String;

// Verify signature
fn verify(pubkey: &str, msg: &[u8], sig: &str) -> Result<()>;
```

**Properties:**
- Deterministic (same input → same output)
- Domain-separated (atom hash ≠ link hash for same data)
- Semantically blind (doesn't know what it's hashing)

---

### ubl-ledger — Append-Only History

**Purpose:** Store the immutable chain of commits.

**Structure:**

```
Container: wallet_alice
├── Entry 0 (Genesis)
│   ├── sequence: 0
│   ├── entry_hash: hash₀
│   ├── previous_hash: 0000...0000
│   └── link: {...}
│
├── Entry 1
│   ├── sequence: 1
│   ├── entry_hash: hash₁
│   ├── previous_hash: hash₀  ← Points to previous
│   └── link: {...}
│
├── Entry 2
│   ├── sequence: 2
│   ├── entry_hash: hash₂
│   ├── previous_hash: hash₁
│   └── link: {...}
│
└── ... (append only, no updates, no deletes)
```

**Invariants:**
- Sequence has no gaps (0, 1, 2, 3, ...)
- Each entry's previous_hash matches prior entry's hash
- State is derivable: `S = rehydrate(H)`

**Merkle Root:**
```
          root
         /    \
      h₀₁      h₂₃
      / \      / \
    h₀   h₁  h₂   h₃
```
Daily anchor for external verification.

---

### ubl-membrane — Law Enforcement

**Purpose:** Validate commits before they enter the ledger.

**Validation Rules:**

| Code | Check | Error |
|------|-------|-------|
| V1 | Version = 1 | InvalidVersion |
| V2 | Container matches | ContainerMismatch |
| V3 | Signature verifies | SignatureInvalid |
| V4 | Previous hash matches | RealityDrift |
| V5 | Sequence is next | SequenceMismatch |
| V6 | Atom hash valid | InvalidAtomHash |
| V7 | Physics holds | ConservationViolation |

**Physics Enforcement:**

```
Observation:   Δ must = 0
Conservation:  balance + Δ must ≥ 0
Entropy:       any Δ allowed (with authority)
Evolution:     rule changes (with authority)
```

**Performance Target:** < 1ms per validation

---

### ubl-server — HTTP Interface

**Purpose:** Expose the Body to the Mind.

**Endpoints:**

```
GET  /health     → { status: "healthy", version: "2.0.0" }
GET  /state      → { container_id, sequence, last_hash, physical_balance }
POST /commit     → { status: "ACCEPTED"|"REJECTED", receipt|error }
POST /validate   → { valid: true|false, error?, code? }
```

**Environment:**
```bash
CONTAINER_ID=wallet_alice  # Which container
PORT=3000                  # Which port
```

---

### ubl-cortex — The Orchestrator

**Purpose:** Help the Mind prepare commits.

**Flow:**
1. Define intent (semantic)
2. Canonicalize (deterministic)
3. Hash (identity)
4. Get state (synchronize)
5. Build link (envelope)
6. Submit (cross boundary)
7. Handle response (proof or error)

```typescript
const cortex = new Cortex({
  bodyUrl: 'http://localhost:3000',
  containerId: 'wallet_alice',
  authorPubkey: '...'
});

// High-level API
await cortex.observe(intent);           // Δ = 0
await cortex.conserve(intent, -100);    // Move value
await cortex.entropy(intent, 1000);     // Create value
```

## API Reference

### GET /state

**Response:**
```json
{
  "container_id": "wallet_alice",
  "sequence": 42,
  "last_hash": "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069",
  "physical_balance": 900,
  "merkle_root": "8a3f2b1c..."
}
```

### POST /commit

**Request:**
```json
{
  "version": 1,
  "container_id": "wallet_alice",
  "expected_sequence": 43,
  "previous_hash": "7f83b165...",
  "atom_hash": "abc123...",
  "intent_class": "conservation",
  "physics_delta": -50,
  "author_pubkey": "ed25519_pubkey_hex",
  "signature": "ed25519_signature_hex"
}
```

**Response (Success):**
```json
{
  "status": "ACCEPTED",
  "receipt": {
    "entry_hash": "def456...",
    "sequence": 43,
    "timestamp": 1735142400,
    "container_id": "wallet_alice"
  }
}
```

**Response (Failure):**
```json
{
  "status": "REJECTED",
  "error": "V7: Physics violation - conservation requires balance >= 0, would be -50",
  "code": "V7_CONSERVATION_VIOLATION"
}
```

## Quick Start

### 1. Start the Body

```bash
cd "/Users/voulezvous/UBL 2.0"

# Build
cargo build --release

# Run (in-memory ledger)
cargo run -p ubl-server

# Or with custom container
CONTAINER_ID=my_wallet PORT=3001 cargo run -p ubl-server
```

### 2. Start the Mind

```bash
cd "/Users/voulezvous/UBL 2.0/packages/ubl-cortex"

npm install
npx tsx src/index.ts
```

### 3. Test the Flow

```bash
# Check health
curl http://localhost:3000/health

# Get state
curl http://localhost:3000/state

# Submit a commit (example)
curl -X POST http://localhost:3000/commit \
  -H "Content-Type: application/json" \
  -d '{
    "version": 1,
    "container_id": "default",
    "expected_sequence": 1,
    "previous_hash": "0000000000000000000000000000000000000000000000000000000000000000",
    "atom_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "intent_class": "entropy",
    "physics_delta": 1000,
    "author_pubkey": "mock",
    "signature": "mock"
  }'
```

## Production Considerations

### Persistent Storage

The current implementation uses in-memory storage. For production:

1. **SQLite** (single-node):
   ```rust
   // Add to ubl-ledger
   LedgerEngine::new("sqlite:./ledger.db").await?
   ```

2. **PostgreSQL** (distributed):
   ```sql
   CREATE TABLE ledger_events (
     id SERIAL PRIMARY KEY,
     container_id TEXT NOT NULL,
     sequence BIGINT NOT NULL,
     entry_hash BYTEA NOT NULL,
     previous_hash BYTEA,
     link_data JSONB NOT NULL,
     timestamp BIGINT NOT NULL,
     UNIQUE(container_id, sequence)
   );
   
   -- Prevent modifications
   CREATE TRIGGER prevent_update BEFORE UPDATE ON ledger_events
   BEGIN RAISE(ABORT, 'Updates not allowed'); END;
   ```

### Signature Verification

Current implementation allows `"mock"` signatures for development. Production requires:

```rust
// In ubl-membrane
if link.signature != "mock" {
    verify(&link.author_pubkey, &link.signing_bytes(), &link.signature)?;
}
```

### Multiple Containers

Run multiple containers with different IDs:

```bash
CONTAINER_ID=wallet_alice PORT=3001 cargo run -p ubl-server &
CONTAINER_ID=wallet_bob PORT=3002 cargo run -p ubl-server &
CONTAINER_ID=escrow PORT=3003 cargo run -p ubl-server &
```

### Merkle Root Anchoring

Daily anchoring to external systems:

```bash
# Get today's merkle root
curl http://localhost:3000/state | jq .merkle_root

# Anchor to Git
echo "2024-12-25: $(curl -s http://localhost:3000/state | jq -r .merkle_root)" >> merkle_anchors.txt
git add merkle_anchors.txt && git commit -m "Daily merkle anchor"
```

## Why This Architecture?

### Separation of Concerns

| Concern | Location | Reason |
|---------|----------|--------|
| Meaning | Mind | Subjective, local, evolvable |
| Proof | Body | Objective, universal, immutable |
| Validation | Membrane | Fast, deterministic, trustless |
| Storage | Ledger | Append-only, verifiable, auditable |

### Auditor Independence

An auditor can verify the ledger without:
- Understanding the business logic
- Trusting the application
- Having access to the Mind

They only need the Body's data and the validation rules.

### Cross-Language Trust

The Body is language-agnostic. Any system that can:
1. Compute BLAKE3 hashes
2. Create Ed25519 signatures
3. Send HTTP requests

...can participate in the ledger.

### Offline Verification

Download the ledger. Verify locally:
1. Check all hashes chain correctly
2. Check all sequences are contiguous
3. Check all signatures verify
4. Check physics invariants hold

No network needed. Mathematical certainty.

---

## Complete Architecture Diagram

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                       UNIVERSAL BUSINESS LEDGER 2.0                           ║
║                         Complete Architecture                                  ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║  ┌─────────────────────────────────────────────────────────────────────────┐ ║
║  │                           THE MIND                                       │ ║
║  │                         (TypeScript)                                     │ ║
║  │                                                                          │ ║
║  │   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐             │ ║
║  │   │    Intent    │ → │  Canonical   │ → │    Hash      │             │ ║
║  │   │  (Semantic)  │    │   (Atom)     │    │  (Identity)  │             │ ║
║  │   └──────────────┘    └──────────────┘    └──────────────┘             │ ║
║  │          │                                       │                       │ ║
║  │          └───────────────────┬───────────────────┘                       │ ║
║  │                              ▼                                           │ ║
║  │                     ┌──────────────┐                                     │ ║
║  │                     │  LinkCommit  │                                     │ ║
║  │                     │  (Envelope)  │                                     │ ║
║  │                     └──────────────┘                                     │ ║
║  │                              │                                           │ ║
║  └──────────────────────────────┼───────────────────────────────────────────┘ ║
║                                 │                                             ║
║                          POST /commit                                         ║
║                                 │                                             ║
║  ┌──────────────────────────────┼───────────────────────────────────────────┐ ║
║  │                              ▼                                           │ ║
║  │   ┌──────────────────────────────────────────────────────────────────┐  │ ║
║  │   │                        MEMBRANE                                   │  │ ║
║  │   │                                                                   │  │ ║
║  │   │   V1: Version    V2: Container   V3: Signature   V4: Causality  │  │ ║
║  │   │   V5: Sequence   V6: AtomHash    V7: Physics                     │  │ ║
║  │   │                                                                   │  │ ║
║  │   │                    ┌─────────┐                                    │  │ ║
║  │   │                    │ ACCEPT  │ or REJECT                          │  │ ║
║  │   │                    └─────────┘                                    │  │ ║
║  │   └──────────────────────────────────────────────────────────────────┘  │ ║
║  │                              │                                           │ ║
║  │                              ▼                                           │ ║
║  │   ┌──────────────────────────────────────────────────────────────────┐  │ ║
║  │   │                        LEDGER                                     │  │ ║
║  │   │                                                                   │  │ ║
║  │   │   [E₀]──hash──[E₁]──hash──[E₂]──hash──[E₃]──hash──[E₄]──···     │  │ ║
║  │   │                                                                   │  │ ║
║  │   │   Append-only. Immutable. Verifiable.                            │  │ ║
║  │   └──────────────────────────────────────────────────────────────────┘  │ ║
║  │                              │                                           │ ║
║  │                              ▼                                           │ ║
║  │                        ┌──────────┐                                      │ ║
║  │                        │ Receipt  │ → Proof of inclusion                │ ║
║  │                        └──────────┘                                      │ ║
║  │                                                                          │ ║
║  │                           THE BODY                                       │ ║
║  │                            (Rust)                                        │ ║
║  └──────────────────────────────────────────────────────────────────────────┘ ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

---

*"The Mind dreams. The Body remembers. Together, they create trust."*
