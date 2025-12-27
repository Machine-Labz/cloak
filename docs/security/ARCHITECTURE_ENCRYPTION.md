# Architecture Migration: Client-Side Encryption & Artifact-Based Proof Generation

## Overview

This document explains two major architectural improvements made to the Cloak system:

1. **Deposit Flow**: Migration from server-side encryption to **client-side encryption** for note data
2. **Withdraw Flow**: Migration from backend proof generation to **artifact-based proof generation** where private inputs never pass through the backend

Both changes significantly improve privacy and security by ensuring that sensitive data never leaves the client in plaintext form.

## Architecture Comparison

### Before: Server-Side Encryption (Deprecated)

**Flow:**
```
┌─────────┐         ┌──────────┐         ┌─────────┐
│ Client  │         │ Backend  │         │ Indexer │
└────┬────┘         └────┬─────┘         └────┬────┘
     │                   │                    │
     │ 1. Note data      │                    │
     │    (plaintext)    │                    │
     ├──────────────────>│                    │
     │                   │                    │
     │                   │ 2. Encrypt         │
     │                   │    note data       │
     │                   │                    │
     │                   │ 3. Encrypted       │
     │                   │    output         │
     │                   ├───────────────────>│
     │                   │                    │
     │                   │ 4. Store           │
     │                   │    encrypted      │
     │                   │    (indexer can   │
     │                   │     decrypt)      │
```

**Characteristics:**
- ❌ Note data (`amount`, `r`, `sk_spend`, `commitment`) sent in **plaintext** from client to backend
- ❌ Backend/Indexer can **read and decrypt** all note data
- ❌ Privacy risk: Backend operators can see all deposit details
- ❌ Security risk: If backend is compromised, all note data is exposed
- ✅ Simpler implementation (no client-side crypto required)

**Data Flow:**
1. Client generates note: `{ amount, r, sk_spend, commitment }`
2. Client sends **plaintext note data** to backend API (`/api/deposit/finalize`)
3. Backend encrypts note data using recipient's public key
4. Backend sends encrypted output to indexer
5. Indexer stores encrypted output (but can decrypt it if needed)

**Code Example (Old):**
```typescript
// Frontend - OLD WAY
const noteData = {
  amount: note.amount,
  r: note.r,
  sk_spend: note.sk_spend,
  commitment: note.commitment,
};

// ❌ Sending plaintext to backend
const response = await fetch('/api/deposit/finalize', {
  method: 'POST',
  body: JSON.stringify({
    tx_signature: signature,
    commitment: note.commitment,
    note_data: noteData, // ❌ Plaintext!
  }),
});
```

```rust
// Backend - OLD WAY
// Backend received plaintext and encrypted it
let encrypted_output = encrypt_note_data(&request.note_data, &recipient_pvk);
// Backend could see all note data before encryption
```

---

### After: Client-Side Encryption (Current)

**Flow:**
```
┌─────────┐         ┌──────────┐         ┌─────────┐
│ Client  │         │ Backend  │         │ Indexer │
└────┬────┘         └────┬─────┘         └────┬────┘
     │                   │                    │
     │ 1. Generate note  │                    │
     │    secrets        │                    │
     │                   │                    │
     │ 2. Encrypt note   │                    │
     │    data locally   │                    │
     │    (X25519 ECDH)  │                    │
     │                   │                    │
     │ 3. Encrypted      │                    │
     │    output (base64)│                    │
     │    (opaque blob)  │                    │
     ├──────────────────>│                    │
     │                   │                    │
     │                   │ 4. Forward         │
     │                   │    encrypted       │
     │                   │    (can't read)    │
     │                   ├───────────────────>│
     │                   │                    │
     │                   │ 5. Validate        │
     │                   │    structure only  │
     │                   │    (can't decrypt) │
     │                   │                    │
     │                   │ 6. Store           │
     │                   │    encrypted      │
     │                   │    (opaque)        │
```

**Characteristics:**
- ✅ Note data encrypted **on the client** before transmission
- ✅ Backend/Indexer **cannot read** note data (only validates structure)
- ✅ Privacy: Backend operators cannot see deposit details
- ✅ Security: Even if backend is compromised, note data remains encrypted
- ✅ Uses strong encryption: **X25519 ECDH + XSalsa20-Poly1305**
- ⚠️ Requires client-side crypto libraries (nacl/tweetnacl)

**Data Flow:**
1. Client generates note: `{ amount, r, sk_spend, commitment }`
2. Client encrypts note data locally using recipient's public view key
3. Client sends **encrypted output (base64)** to backend API
4. Backend forwards encrypted output to indexer (cannot decrypt)
5. Indexer validates structure (`ephemeral_pk`, `ciphertext`, `nonce`) and stores
6. Only the recipient (with private view key) can decrypt later

**Code Example (New):**
```typescript
// Frontend - NEW WAY
const noteData = {
  amount: note.amount,
  r: note.r,
  sk_spend: note.sk_spend,
  commitment: note.commitment,
};

// ✅ Encrypt on client before sending
const publicViewKey = getPublicViewKey();
const pvkBytes = Buffer.from(publicViewKey, "hex");
const encryptedNote = encryptNoteForRecipient(noteData, pvkBytes);
const encryptedOutput = btoa(JSON.stringify(encryptedNote));

// ✅ Only encrypted data sent to backend
const response = await fetch('/api/deposit/finalize', {
  method: 'POST',
  body: JSON.stringify({
    tx_signature: signature,
    commitment: note.commitment,
    encrypted_output: encryptedOutput, // ✅ Opaque base64 blob
  }),
});
```

```rust
// Backend - NEW WAY
// Backend receives encrypted_output and just forwards it
// Cannot decrypt - only validates structure
if let Ok(decoded) = base64::decode(&request.encrypted_output) {
    if let Ok(json) = serde_json::from_str::<Value>(&String::from_utf8(decoded)?) {
        // ✅ Only validates structure, cannot read content
        let has_ephemeral_pk = json.get("ephemeral_pk").is_some();
        let has_ciphertext = json.get("ciphertext").is_some();
        let has_nonce = json.get("nonce").is_some();
        // ✅ Backend cannot decrypt - data remains private
    }
}
```

---

## Encryption Details

### Encryption Scheme

**Algorithm:** X25519 ECDH + XSalsa20-Poly1305 (via NaCl/TweetNaCl)

**Components:**
1. **Ephemeral Key Pair:** Generated fresh for each note encryption
   - `ephemeral_sk`: Random 32-byte secret key
   - `ephemeral_pk`: Derived public key (sent with encrypted data)

2. **Shared Secret:** Computed via ECDH
   ```
   shared_secret = ECDH(recipient_pvk, ephemeral_sk)
   ```

3. **Encryption:** XSalsa20-Poly1305 authenticated encryption
   ```
   ciphertext = XSalsa20-Poly1305(plaintext, nonce, shared_secret)
   ```

4. **Output Format:**
   ```json
   {
     "ephemeral_pk": "hex_string (64 chars)",
     "ciphertext": "hex_string (variable length)",
     "nonce": "hex_string (48 chars)"
   }
   ```
   Then base64-encoded for transmission.

### Decryption (Recipient Side)

Only the recipient with the private view key can decrypt:

```typescript
// Recipient decrypts using their private view key
const viewKey = getViewKey(); // Private view key
const decrypted = tryDecryptNote(encryptedNote, viewKey);

if (decrypted) {
  // ✅ Successfully decrypted - this note belongs to us
  const { amount, r, sk_spend, commitment } = decrypted;
} else {
  // ❌ Decryption failed - this note belongs to someone else
}
```

---

## Security Improvements

### Privacy Benefits

1. **Backend Privacy:** Backend operators cannot see deposit amounts, spending keys, or other sensitive data
2. **Database Privacy:** Even if database is compromised, note data remains encrypted
3. **Network Privacy:** Note data never travels in plaintext over the network
4. **Compliance:** Better alignment with privacy regulations (data encrypted at rest and in transit)

### Security Benefits

1. **Reduced Attack Surface:** Backend compromise doesn't expose note data
2. **Defense in Depth:** Multiple layers of encryption (client-side + network TLS)
3. **Forward Secrecy:** Each note uses a fresh ephemeral key
4. **Authenticated Encryption:** XSalsa20-Poly1305 provides integrity verification

### Threat Model Changes

**Before:**
- ❌ Backend compromise → All note data exposed
- ❌ Database breach → All note data readable
- ❌ Network interception → Note data visible (if TLS fails)

**After:**
- ✅ Backend compromise → Note data remains encrypted
- ✅ Database breach → Note data remains encrypted
- ✅ Network interception → Only encrypted blobs visible

---

## Implementation Details

### Frontend Changes

**File:** `services/web/app/transaction/page.tsx`

**Key Changes:**
1. Added client-side encryption before sending to backend
2. Uses `encryptNoteForRecipient()` from `services/web/lib/keys.ts`
3. Sends only `encrypted_output` (base64) instead of plaintext note data

**Code Location:**
```typescript
// Line ~1026
const encryptedNote = encryptNoteForRecipient(noteData, pvkBytes);
const encryptedOutput = btoa(JSON.stringify(encryptedNote));
```

### Backend Changes

**File:** `services/web/app/api/deposit/finalize/route.ts`

**Key Changes:**
1. Receives `encrypted_output` instead of plaintext note data
2. Forwards encrypted output to indexer without modification
3. No decryption or inspection of note data

**Code Location:**
```typescript
// Receives encrypted_output and forwards to indexer
const depositPayload = {
  leaf_commit: commitment,
  encrypted_output: encrypted_output, // Opaque base64
  tx_signature: tx_signature,
  slot: slot,
};
```

### Indexer Changes

**File:** `services/indexer/src/server/final_handlers.rs`

**Key Changes:**
1. Receives `encrypted_output` as opaque string
2. Validates structure (checks for `ephemeral_pk`, `ciphertext`, `nonce`)
3. Stores encrypted output without decryption
4. Cannot read note data content

**Code Location:**
```rust
// Line ~110-151
// Validates encrypted output structure
if let Ok(decoded) = base64::decode(&request.encrypted_output) {
    if let Ok(json) = serde_json::from_str::<Value>(&String::from_utf8(decoded)?) {
        // Only validates structure, cannot decrypt
        let has_ephemeral_pk = json.get("ephemeral_pk").is_some();
        let has_ciphertext = json.get("ciphertext").is_some();
        let has_nonce = json.get("nonce").is_some();
    }
}
```

---

## Migration Checklist

### Deposit Flow (Client-Side Encryption)
- [x] Implement client-side encryption in frontend
- [x] Update backend to accept `encrypted_output` instead of plaintext
- [x] Update indexer to validate encrypted output structure
- [x] Add logging to verify encryption is working
- [x] Test end-to-end deposit flow with client-side encryption
- [x] Verify backend/indexer cannot decrypt note data

### Withdraw Flow (Artifact-Based Proof Generation)
- [x] Implement artifact creation endpoint (`POST /api/v1/tee/artifact`)
- [x] Implement stdin upload endpoint (`POST /api/v1/tee/artifact/:id/upload`)
- [x] Implement proof request endpoint (`POST /api/v1/tee/request-proof`)
- [x] Implement proof status endpoint (`GET /api/v1/tee/proof-status`)
- [x] Update frontend to use artifact-based flow
- [x] Remove deprecated `/api/v1/prove` endpoint
- [x] Test end-to-end withdraw flow with artifacts
- [x] Verify backend never receives private inputs in plaintext
- [x] Update documentation

---

## Testing

### Verify Client-Side Encryption (Deposit)

**Frontend Logs:**
```javascript
🔐 [DEPOSIT DEBUG] Encrypting note data: { ... }
🔐 [DEPOSIT DEBUG] Note encrypted: { hasEphemeralPk: true, ... }
🔐 [DEPOSIT DEBUG] Encrypted output prepared: { encryptedOutputLength: 920, ... }
```

**Backend Logs:**
```typescript
[Deposit Finalize] 🔐 Encrypted output received
[Deposit Finalize] 🔐 Encrypted output structure: { hasEphemeralPk: true, ... }
```

**Indexer Logs:**
```rust
✅ Encrypted output format verified - contains encryption fields
encrypted_output_format = "base64_json"
has_ephemeral_pk = true
has_ciphertext = true
has_nonce = true
```

### Verify Artifact-Based Proof Generation (Withdraw)

**Frontend Logs:**
```javascript
[ArtifactProver] Step 1: Creating artifact...
[ArtifactProver] Artifact created: uuid-here
[ArtifactProver] Upload URL (raw): /api/v1/tee/artifact/.../upload
[ArtifactProver] Step 2: Uploading stdin to TEE...
[ArtifactProver] Stdin uploaded successfully
[ArtifactProver] Step 3: Requesting proof generation...
[ArtifactProver] Proof request created: uuid-here
[ArtifactProver] Step 4: Polling for proof status...
[ArtifactProver] Proof generation completed
```

**Indexer Logs:**
```rust
📦 Creating artifact for program_id: None
✅ Artifact created artifact_id=... upload_url=...
📤 Receiving stdin upload artifact_id=...
✅ Stdin uploaded successfully
🔐 Requesting proof generation artifact_id=...
✅ Proof request created, processing in background request_id=...
✅ Proof generation completed request_id=... generation_time_ms=...
```

### Verify Backend Cannot Access Private Data

**Deposit Flow:**
- Backend should **not** be able to decrypt the encrypted output because:
  1. Backend doesn't have the recipient's private view key
  2. Only the recipient (client) can decrypt using their private view key

**Withdraw Flow:**
- Backend should **not** receive private inputs because:
  1. Private inputs are uploaded directly to TEE (bypassing backend)
  2. Backend only receives `artifact_id` and `public_inputs`
  3. Backend cannot access stdin data stored in TEE

---

## Backward Compatibility

**Breaking Change:** This is a **breaking change** for any clients that were sending plaintext note data. All clients must be updated to use client-side encryption.

**Migration Path:**
1. Deploy backend/indexer with support for `encrypted_output`
2. Update all clients to use client-side encryption
3. Remove support for plaintext note data (if it existed)

---

## API Endpoints

### Deposit Flow (Client-Side Encryption)

- `POST /api/v1/deposit` - Register deposit with encrypted note data
  - Receives: `{ leaf_commit, encrypted_output, tx_signature, slot }`
  - Returns: `{ success, leaf_index, root, merkle_proof }`

### Withdraw Flow (Artifact-Based Proof Generation)

- `POST /api/v1/tee/artifact` - Create artifact and get upload URL
  - Receives: `{ program_id? }`
  - Returns: `{ artifact_id, upload_url, expires_at }`

- `POST /api/v1/tee/artifact/:artifact_id/upload` - Upload stdin directly to TEE
  - Receives: `{ private: {...}, public: {...}, outputs: [...] }` (JSON)
  - Returns: `{ success, artifact_id }`
  - **Note:** This endpoint is called directly by the frontend, not through Next.js API route

- `POST /api/v1/tee/request-proof` - Request proof generation
  - Receives: `{ artifact_id, program_id?, public_inputs }`
  - Returns: `{ request_id, status: "pending" }`

- `GET /api/v1/tee/proof-status` - Get proof generation status
  - Query: `?request_id=...`
  - Returns: `{ request_id, status, proof?, public_inputs?, generation_time_ms?, error? }`

### Deprecated Endpoints

- ~~`POST /api/v1/prove`~~ - **REMOVED** - Use artifact-based flow instead

## Related Documentation

- **[Deposit Workflow](./workflows/deposit.md)** - Complete deposit flow guide
- **[Withdraw Workflow](./workflows/withdraw.md)** - Complete withdraw flow guide
- **[View/Spend Keys](./overview/view-spend-keys.md)** - Key management and encryption
- **[Indexer API](./api/indexer.md)** - Indexer endpoints and data structures
- **[System Architecture](./overview/system-architecture.md)** - Overall system design
- **[Architecture Comparison](../ARCHITECTURE_COMPARISON.md)** - Comparison with suggested SP1 flow

---

---

## Part 2: Artifact-Based Proof Generation (Withdraw Flow)

### Overview

The withdraw flow has been migrated from a backend-centric proof generation model to an **artifact-based model** where private inputs are uploaded directly to the TEE (Trusted Execution Environment), never passing through the backend in plaintext.

### Before: Backend Proof Generation (Deprecated)

**Flow:**
```
┌─────────┐         ┌──────────┐         ┌─────────┐
│ Client  │         │ Backend  │         │   TEE   │
└────┬────┘         └────┬─────┘         └────┬────┘
     │                   │                    │
     │ 1. Private inputs │                    │
     │    (plaintext)    │                    │
     │    + Public       │                    │
     ├──────────────────>│                    │
     │                   │                    │
     │                   │ 2. Build stdin     │
     │                   │    from inputs     │
     │                   │                    │
     │                   │ 3. Request proof   │
     │                   ├───────────────────>│
     │                   │                    │
     │                   │ 4. Generate proof  │
     │                   │◀───────────────────│
     │                   │                    │
     │ 5. Return proof   │                    │
     │◀──────────────────│                    │
```

**Characteristics:**
- ❌ Private inputs (`amount`, `r`, `sk_spend`, `merkle_path`) sent in **plaintext** from client to backend
- ❌ Backend can **read all private inputs** before sending to TEE
- ❌ Privacy risk: Backend operators can see all withdraw details
- ❌ Security risk: If backend is compromised, all private inputs are exposed
- ✅ Simpler implementation (single endpoint)

**Code Example (Old):**
```typescript
// Frontend - OLD WAY
const response = await fetch('/api/v1/prove', {
  method: 'POST',
  body: JSON.stringify({
    private_inputs: JSON.stringify({
      amount: note.amount,
      r: note.r,
      sk_spend: note.sk_spend,
      leaf_index: leafIndex,
      merkle_path: { ... }
    }), // ❌ Plaintext private data
    public_inputs: JSON.stringify({ ... }),
    outputs: JSON.stringify([...])
  })
});
```

```rust
// Backend - OLD WAY
// Backend received plaintext private inputs
pub async fn generate_proof(
    Json(request): Json<ProveRequest>,
) -> Response {
    // ❌ Backend can see private_inputs in plaintext
    let private_inputs = request.private_inputs; // Plaintext!
    // ... build stdin and call TEE
}
```

---

### After: Artifact-Based Proof Generation (Current)

**Flow:**
```
┌─────────┐         ┌──────────┐         ┌─────────┐
│ Client  │         │ Backend  │         │   TEE   │
└────┬────┘         └────┬─────┘         └────┬────┘
     │                   │                    │
     │ 1. Request        │                    │
     │    artifact       │                    │
     ├──────────────────>│                    │
     │                   │                    │
     │ 2. Create artifact│                    │
     │    + upload URL   │                    │
     │◀──────────────────│                    │
     │                   │                    │
     │ 3. Upload private │                    │
     │    inputs directly│                    │
     │    (to TEE)       │                    │
     │───────────────────────────────────────>│
     │                   │                    │
     │ 4. Request proof  │                    │
     │    (artifact_id) │                    │
     ├──────────────────>│                    │
     │                   │                    │
     │                   │ 5. Request proof   │
     │                   │    using artifact  │
     │                   ├───────────────────>│
     │                   │                    │
     │                   │ 6. Generate proof  │
     │                   │◀───────────────────│
     │                   │                    │
     │ 7. Poll status    │                    │
     │    until ready    │                    │
     ├──────────────────>│                    │
     │                   │                    │
     │ 8. Return proof   │                    │
     │◀──────────────────│                    │
```

**Characteristics:**
- ✅ Private inputs uploaded **directly to TEE** (never pass through backend)
- ✅ Backend **cannot read** private inputs (only orchestrates via artifact_id)
- ✅ Privacy: Backend operators cannot see withdraw details
- ✅ Security: Even if backend is compromised, private inputs remain in TEE
- ✅ Uses SP1's artifact system for secure stdin uploads
- ⚠️ More complex implementation (multiple endpoints, polling)

**Code Example (New):**
```typescript
// Frontend - NEW WAY
// Step 1: Create artifact
const artifactRes = await fetch('/api/tee/artifact', {
  method: 'POST',
  body: JSON.stringify({ program_id: null })
});
const { artifact_id, upload_url } = await artifactRes.json();

// Step 2: Upload private inputs directly to TEE
const stdinPayload = JSON.stringify({
  private: { amount, r, sk_spend, leaf_index, merkle_path },
  public: { root, nf, outputs_hash, amount },
  outputs: [...]
});
await fetch(upload_url, { // ✅ Direct to TEE
  method: 'POST',
  body: stdinPayload
});

// Step 3: Request proof (backend only gets artifact_id)
const proofRes = await fetch('/api/tee/request-proof', {
  method: 'POST',
  body: JSON.stringify({
    artifact_id,
    public_inputs: JSON.stringify({ ... })
  })
});
const { request_id } = await proofRes.json();

// Step 4: Poll for proof status
const status = await fetch(`/api/tee/proof-status?request_id=${request_id}`);
```

```rust
// Backend - NEW WAY
// Backend never sees private inputs
pub async fn create_artifact(...) -> Response {
    // Returns artifact_id and upload_url
    // Private inputs never come here
}

pub async fn request_proof(
    Json(request): Json<RequestProofRequest>,
) -> Response {
    // Only receives artifact_id, not private inputs
    let artifact_id = request.artifact_id; // ✅ No private data
    // Backend requests proof from TEE using artifact_id
    // TEE already has private inputs from direct upload
}
```

---

## Summary

### Deposit Flow Migration

| Aspect | Before | After |
|--------|--------|-------|
| **Encryption Location** | Server | Client |
| **Backend Can Read** | ✅ Yes | ❌ No |
| **Privacy** | ⚠️ Backend sees all data | ✅ Backend sees nothing |
| **Security** | ⚠️ Backend compromise = data leak | ✅ Backend compromise = no data leak |
| **Encryption** | Server-side (unknown scheme) | Client-side (X25519 + XSalsa20-Poly1305) |

**Result:** Note data is now encrypted end-to-end, with only the recipient able to decrypt using their private view key.

### Withdraw Flow Migration

| Aspect | Before | After |
|--------|--------|-------|
| **Private Inputs Path** | Client → Backend → TEE | Client → TEE (direct) |
| **Backend Sees Private Data** | ✅ Yes (plaintext) | ❌ No |
| **Privacy** | ⚠️ Backend sees all inputs | ✅ Backend sees nothing |
| **Security** | ⚠️ Backend compromise = input leak | ✅ Backend compromise = no input leak |
| **Architecture** | Single endpoint | Artifact-based (3 endpoints) |
| **TEE Integration** | Backend builds stdin | Client uploads directly |

**Result:** Private inputs for proof generation are now uploaded directly to the TEE, with the backend only orchestrating the proof request using artifact IDs. This provides true privacy-preserving withdrawals where backend operators cannot see withdraw details.

### Combined Benefits

1. **End-to-End Privacy**: Both deposits and withdrawals protect user data from backend operators
2. **Defense in Depth**: Multiple layers of protection (client-side encryption + direct TEE upload)
3. **Compliance**: Better alignment with privacy regulations
4. **Trust Model**: Reduced trust requirements for backend operators
5. **Security Posture**: Backend compromise doesn't expose sensitive user data
