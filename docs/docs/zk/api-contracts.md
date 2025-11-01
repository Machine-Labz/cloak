# ZK API Contracts & Specifications

This document defines the API contracts for Cloak's zero-knowledge privacy layer, including indexer endpoints, relay endpoints, data formats, and error handling.

## Indexer API

### Merkle Tree Endpoints

**Get Current Root:**
```http
GET /api/v1/merkle/root
```

**Response:**
```json
{
  "root": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
  "next_index": 42,
  "height": 32,
  "updated_at": "2025-01-15T10:30:00Z"
}
```

**Get Merkle Proof:**
```http
GET /api/v1/merkle/proof/{leaf_index}
```

**Response:**
```json
{
  "path_elements": [
    "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
    "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
  ],
  "path_indices": [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
  "leaf": "commitment_hash_at_leaf_index",
  "root": "current_tree_root"
}
```

**Get Leaf Range:**
```http
GET /api/v1/merkle/leaves?start={start}&limit={limit}
```

**Response:**
```json
{
  "leaves": [
    {
      "index": 0,
      "commitment": "commitment_hash_0"
    }
  ],
  "has_more": true,
  "total_count": 1000
}
```

### Artifact Endpoints

**Get Withdraw Artifacts:**
```http
GET /api/v1/artifacts/withdraw/{version}
```

**Response:**
```json
{
  "guestElfUrl": "ipfs://QmHash...",
  "vk": "base64_encoded_verification_key",
  "sha256": {
    "elf": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
    "vk": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"
  },
  "sp1Version": "0.8.0",
  "createdAt": "2025-01-15T10:30:00Z",
  "metadata": {
    "constraints": 6,
    "publicInputs": 104,
    "proofSize": 260
  }
}
```

## Relay API

### Withdraw Endpoints

**Submit Withdraw:**
```http
POST /api/v1/withdraw
```

**Request:**
```json
{
  "outputs": [
    {
      "address": "11111111111111111111111111111112",
      "amount": "500000"
    },
    {
      "address": "22222222222222222222222222222223",
      "amount": "300000"
    }
  ],
  "policy": {
    "fee_bps": 60
  },
  "public_inputs": {
    "root": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
    "nf": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
    "amount": "1000000",
    "fee_bps": 60,
    "outputs_hash": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
  },
  "proof_bytes": "base64_encoded_groth16_proof"
}
```

**Response:**
```json
{
  "request_id": "uuid",
  "txid": "transaction_signature",
  "root_used": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
  "nf": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
  "receipt_asset": "receipt_asset_id"
}
```

**Get Withdraw Status:**
```http
GET /api/v1/withdraw/{request_id}
```

**Response:**
```json
{
  "state": "queued|executing|settled|failed",
  "txid": "transaction_signature",
  "root_used": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
  "nf": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
  "receipt_asset": "receipt_asset_id",
  "error": null
}
```

## Data Formats

### Public Inputs Format

**104-byte Public Inputs:**
```
Offset  Size    Field           Description
0       32      root            Merkle tree root
32      32      nullifier       Spending nullifier
64      32      outputs_hash    Hash of output recipients
96      8       amount          Total amount being spent
```

### Proof Format

**260-byte Groth16 Proof:**
```
Offset  Size    Field           Description
0       64      A               G1 point (compressed)
64      64      B               G2 point (compressed)
128     64      C               G1 point (compressed)
192     68      Padding         Reserved for future use
```

## Error Handling

### Error Response Format

**Standard Error Response:**
```json
{
  "error": {
    "code": "INVALID_PROOF",
    "message": "Proof verification failed",
    "details": {
      "constraint": "merkle_inclusion",
      "expected": "a1b2c3d4...",
      "actual": "fedcba09..."
    }
  }
}
```

### Error Codes

**Indexer Errors:**
- `INVALID_LEAF_INDEX` - Leaf index out of range
- `MISSING_SIBLING` - Merkle path sibling not found
- `INVALID_PATH_LENGTH` - Merkle path length mismatch
- `ROOT_NOT_FOUND` - Root not found in ring buffer

**Relay Errors:**
- `INVALID_PROOF` - Proof verification failed
- `INVALID_ROOT` - Root not found in ring buffer
- `OUTPUTS_MISMATCH` - Outputs hash mismatch
- `DOUBLE_SPEND` - Nullifier already used
- `AMOUNT_CONSERVATION` - Amount conservation failed

## Rate Limiting

### Rate Limits

**Indexer:**
- 100 requests per minute per IP
- 1000 requests per minute per API key

**Relay:**
- 10 withdraw requests per minute per IP
- 100 withdraw requests per minute per API key

### Rate Limit Headers

**Response Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
```

## Authentication

### API Key Authentication

**Header:**
```
Authorization: Bearer {api_key}
```

**API Key Format:**
```
cloak_{environment}_{random_32_chars}
```

## WebSocket Subscriptions

### Real-time Updates

**Subscribe to Root Updates:**
```javascript
const ws = new WebSocket('ws://localhost:3001/ws/merkle/root');
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('New root:', data.root);
};
```

**Subscribe to Withdraw Status:**
```javascript
const ws = new WebSocket('ws://localhost:3002/ws/withdraw/{request_id}');
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Status update:', data.state);
};
```

## Client SDK Examples

### TypeScript Client

**Indexer Client:**
```typescript
export class IndexerClient {
  constructor(private baseUrl: string) {}
  
  async getRoot(): Promise<MerkleRootResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/merkle/root`);
    return response.json();
  }
  
  async getProof(leafIndex: number): Promise<MerkleProofResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/merkle/proof/${leafIndex}`);
    return response.json();
  }
}
```

**Relay Client:**
```typescript
export class RelayClient {
  constructor(private baseUrl: string) {}
  
  async submitWithdraw(request: WithdrawRequest): Promise<WithdrawResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/withdraw`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request)
    });
    return response.json();
  }
  
  async getWithdrawStatus(requestId: string): Promise<WithdrawStatusResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/withdraw/${requestId}`);
    return response.json();
  }
}
```

## Testing & Validation

### API Testing

**Contract Tests:**
```rust
#[tokio::test]
async fn test_indexer_api_contracts() {
    let client = IndexerClient::new("http://localhost:3001");
    
    // Test root endpoint
    let root = client.get_root().await.unwrap();
    assert_eq!(root.root.len(), 64); // 32 bytes = 64 hex chars
    
    // Test proof endpoint
    let proof = client.get_proof(0).await.unwrap();
    assert_eq!(proof.path_elements.len(), 32);
    assert_eq!(proof.path_indices.len(), 32);
}
```

**Integration Tests:**
```rust
#[tokio::test]
async fn test_relay_api_contracts() {
    let client = RelayClient::new("http://localhost:3002");
    
    // Test withdraw submission
    let request = WithdrawRequest {
        outputs: vec![Output { address: [0x01; 32], amount: 500000 }],
        policy: Policy { fee_bps: 60 },
        public_inputs: PublicInputs { /* ... */ },
        proof_bytes: "base64_proof".to_string(),
    };
    
    let response = client.submit_withdraw(request).await.unwrap();
    assert!(response.request_id.len() > 0);
}
```

This API contract specification ensures consistent integration between Cloak's zero-knowledge privacy layer and client applications.
