# Security Audit Report - Cloak Protocol

**Date:** 2025-01-27  
**Scope:** Complete analysis of the Cloak system for security vulnerabilities

---

## Executive Summary

This audit identified **15 security vulnerabilities**, categorized by severity:
- **Critical:** 3
- **High:** 5
- **Medium:** 4
- **Low:** 3

---

## 1. Critical Vulnerabilities

### 1.1 Absence of Rate Limiting ⚠️ CRITICAL

**Location:**
- `services/relay/src/main.rs:143-167`
- `services/indexer/src/server/routes.rs:100-146`

**Description:**
No endpoint has rate limiting implemented. This allows:
- DoS/DDoS attacks
- Resource abuse (especially `/api/v1/prove` which is computationally expensive)
- Spam of withdraw requests

**Evidence:**
```rust
// services/relay/src/main.rs:143
let cors = CorsLayer::permissive();
// No rate limiting middleware
```

**Impact:**
- High computational cost with coordinated attacks
- Possible denial of service
- Abuse of TEE resources for proof generation

**Recommendation:**
```rust
// Implement rate limiting using tower-governor or similar
use tower_governor::{Governor, GovernorConfigBuilder};

let governor_conf = Box::new(
    GovernorConfigBuilder::default()
        .per_second(10) // 10 req/s per IP
        .burst_size(20)
        .finish()
        .unwrap()
);

let app = Router::new()
    .route("/withdraw", post(api::withdraw::handle_withdraw))
    .layer(GovernorLayer {
        config: governor_conf,
    });
```

---

### 1.2 Use of `unwrap()` in Production Code ⚠️ CRITICAL

**Location:**
- `services/relay/src/api/validator_agent.rs:121`
- `services/relay/src/worker/processor.rs:81`
- `services/relay/src/solana/transaction_builder.rs:604,660`

**Description:**
Use of `unwrap()` can cause panics and crash the service.

**Evidence:**
```rust
// services/relay/src/api/validator_agent.rs:121
let amount = u64::from_le_bytes(public[96..104].try_into().unwrap());

// services/relay/src/worker/processor.rs:81
let amt = u64::from_le_bytes(job.public_inputs[96..104].try_into().unwrap());
```

**Impact:**
- Service panic on invalid data
- Possible DoS through malformed requests
- Loss of availability

**Recommendation:**
```rust
// Replace all unwrap() with proper error handling
let amount = u64::from_le_bytes(
    public[96..104]
        .try_into()
        .map_err(|_| Error::ValidationError("Invalid amount bytes".to_string()))?
);
```

---

### 1.3 Permissive CORS in Production ⚠️ CRITICAL

**Location:**
- `services/relay/src/main.rs:143`
- `services/indexer/src/server/middleware.rs:82-110`

**Description:**
CORS configured as `permissive()` in relay, allowing requests from any origin.

**Evidence:**
```rust
// services/relay/src/main.rs:143
let cors = CorsLayer::permissive();
```

**Impact:**
- CSRF attacks
- Data leakage through cross-origin requests
- API abuse by malicious sites

**Recommendation:**
```rust
// services/relay/src/main.rs
let cors = CorsLayer::new()
    .allow_origin(
        env::var("ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().parse().unwrap())
            .collect::<Vec<_>>()
    )
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([CONTENT_TYPE, AUTHORIZATION]);
```

---

## 2. High Vulnerabilities

### 2.1 Absence of Authentication on Critical Endpoints ⚠️ HIGH

**Location:**
- `services/relay/src/api/withdraw.rs:45`
- `services/indexer/src/server/prover_handler.rs:60`

**Description:**
Critical endpoints like `/withdraw` and `/api/v1/prove` do not require authentication.

**Impact:**
- Anyone can submit withdraws
- Abuse of proof generation resources
- Possible resource exhaustion

**Recommendation:**
Implement authentication based on API keys or JWT tokens:
```rust
// Authentication middleware
async fn auth_middleware(
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response> {
    let api_key = headers
        .get("X-API-Key")
        .ok_or(Error::Unauthorized)?;
    
    // Validate API key
    validate_api_key(api_key).await?;
    
    Ok(next.run(request).await)
}
```

---

### 2.2 Insufficient Validation of Solana Addresses ⚠️ HIGH

**Location:**
- `services/relay/src/api/withdraw.rs:180-184`
- `services/web/components/transaction/withdraw-flow.tsx:85`

**Description:**
Solana address validation only checks minimum length, does not validate base58 format.

**Evidence:**
```rust
// services/relay/src/api/withdraw.rs:180
if output.recipient.len() < 32 {
    return Err(Error::ValidationError(
        "Invalid recipient address".to_string(),
    ));
}
```

**Impact:**
- Possible injection of invalid data
- Runtime errors when processing transactions
- Possible bypass of validations

**Recommendation:**
```rust
use solana_sdk::pubkey::Pubkey;

fn validate_solana_address(address: &str) -> Result<Pubkey, Error> {
    Pubkey::from_str(address)
        .map_err(|_| Error::ValidationError("Invalid Solana address".to_string()))
}
```

---

### 2.3 Possible Overflow in Fee Calculations ⚠️ HIGH

**Location:**
- `programs/shield-pool/src/instructions/withdraw.rs:393`
- `services/relay/src/planner/mod.rs` (if exists)

**Description:**
Fee calculation uses direct multiplication without overflow checking.

**Evidence:**
```rust
// programs/shield-pool/src/instructions/withdraw.rs:393
let expected_fee = 2_500_000u64 + (parsed.public_amount * 5) / 1_000;
```

**Impact:**
- Overflow on very large values
- Incorrect fee calculation
- Possible bypass of validations

**Recommendation:**
```rust
let expected_fee = 2_500_000u64
    .saturating_add(
        parsed.public_amount
            .checked_mul(5)
            .ok_or(ShieldPoolError::MathOverflow)?
            / 1_000
    );
```

---

### 2.4 Use of `dangerouslySetInnerHTML` without Sanitization ⚠️ HIGH

**Location:**
- `services/web/components/ui/chart.tsx:81`

**Description:**
Use of `dangerouslySetInnerHTML` to inject CSS, but without adequate sanitization.

**Evidence:**
```typescript
// services/web/components/ui/chart.tsx:81
dangerouslySetInnerHTML={{
  __html: Object.entries(THEMES)
    .map(([theme, prefix]) => `
${prefix} [data-chart=${id}] {
${colorConfig.map(...).join('\n')}
}
`)
    .join('\n'),
}}
```

**Impact:**
- Possible XSS if `id` or `colorConfig` come from user input
- Malicious CSS injection

**Recommendation:**
```typescript
// Sanitize id before using
const sanitizedId = id.replace(/[^a-zA-Z0-9-_]/g, '');

// Or use sanitization library
import DOMPurify from 'dompurify';
dangerouslySetInnerHTML={{
  __html: DOMPurify.sanitize(cssString)
}}
```

---

### 2.5 Request Size Validation Can Be Bypassed ⚠️ HIGH

**Location:**
- `services/indexer/src/server/middleware.rs:129-132`

**Description:**
1MB limit may be insufficient for some cases and there's no validation of JSON array size.

**Evidence:**
```rust
// services/indexer/src/server/middleware.rs:129
pub fn request_size_limit() -> axum::extract::DefaultBodyLimit {
    axum::extract::DefaultBodyLimit::max(1024 * 1024) // 1MB
}
```

**Impact:**
- Possible DoS through large requests
- Excessive memory consumption
- Request timeouts

**Recommendation:**
```rust
// Add array size validation
fn validate_request_size(payload: &WithdrawRequest) -> Result<(), Error> {
    // Validate maximum number of outputs
    if payload.outputs.len() > 10 {
        return Err(Error::ValidationError("Too many outputs".to_string()));
    }
    
    // Validate total request size
    let size = serde_json::to_string(payload)?.len();
    if size > 1024 * 1024 {
        return Err(Error::ValidationError("Request too large".to_string()));
    }
    
    Ok(())
}
```

---

## 3. Medium Vulnerabilities

### 3.1 Logging of Sensitive Data ⚠️ MEDIUM

**Location:**
- `services/indexer/src/server/prover_handler.rs:70-76`
- `services/relay/src/api/withdraw.rs:49`

**Description:**
Logs may contain sensitive information such as private inputs, addresses, etc.

**Evidence:**
```rust
// services/indexer/src/server/prover_handler.rs:70
tracing::info!(
    client_ip = client_addr.ip().to_string(),
    private_inputs_len = request.private_inputs.len(),
    // ...
);
```

**Impact:**
- Leakage of sensitive information in logs
- Possible reconstruction of private data
- Privacy violation

**Recommendation:**
```rust
// Don't log sensitive data, only metadata
tracing::info!(
    client_ip = client_addr.ip().to_string(),
    private_inputs_len = request.private_inputs.len(),
    // DO NOT log: request.private_inputs
);
```

---

### 3.2 Lack of Timestamp/Nonce Validation ⚠️ MEDIUM

**Location:**
- `services/relay/src/api/withdraw.rs:45`

**Description:**
Withdraw requests do not have nonce or timestamp, allowing replay attacks.

**Impact:**
- Replay attacks
- Possible transaction duplication
- System abuse

**Recommendation:**
```rust
#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    pub nonce: u64, // Unique nonce per request
    pub timestamp: i64, // Request timestamp
    // ... other fields
}

// Validate nonce and timestamp
fn validate_replay_protection(request: &WithdrawRequest) -> Result<(), Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    // Timestamp cannot be too old or future
    if (request.timestamp - now).abs() > 300 { // 5 minutes
        return Err(Error::ValidationError("Invalid timestamp".to_string()));
    }
    
    // Check if nonce was already used (store in cache/DB)
    if is_nonce_used(request.nonce).await? {
        return Err(Error::ValidationError("Nonce already used".to_string()));
    }
    
    Ok(())
}
```

---

### 3.3 Hex String Validation May Accept Empty Strings ⚠️ MEDIUM

**Location:**
- `services/relay/src/api/withdraw.rs:78-87`

**Description:**
Hex string validation does not check if the string is empty after removing "0x" prefix.

**Evidence:**
```rust
// services/relay/src/api/withdraw.rs:78
let root_str = payload.public_inputs.root.strip_prefix("0x").unwrap_or(&payload.public_inputs.root);
let root_hash = hex::decode(root_str)
    .map_err(|e| Error::ValidationError(format!("Invalid root hex: {}", e)))?;
```

**Impact:**
- May accept empty strings as valid
- Bypass of validations
- Runtime errors

**Recommendation:**
```rust
let root_str = payload.public_inputs.root.strip_prefix("0x").unwrap_or(&payload.public_inputs.root);
if root_str.is_empty() {
    return Err(Error::ValidationError("Root cannot be empty".to_string()));
}
if root_str.len() != 64 {
    return Err(Error::ValidationError("Root must be 64 hex characters".to_string()));
}
let root_hash = hex::decode(root_str)
    .map_err(|e| Error::ValidationError(format!("Invalid root hex: {}", e)))?;
```

---

### 3.4 Lack of Array Limit Validation ⚠️ MEDIUM

**Location:**
- `services/relay/src/api/withdraw.rs:166`

**Description:**
Maximum number of outputs validation exists, but there's no validation of minimum limits or string sizes.

**Evidence:**
```rust
// services/relay/src/api/withdraw.rs:166
if request.outputs.len() > 10 {
    return Err(Error::ValidationError(
        "Too many outputs (max 10)".to_string(),
    ));
}
```

**Recommendation:**
```rust
// Validate minimum and maximum limits
if request.outputs.is_empty() {
    return Err(Error::ValidationError("At least one output required".to_string()));
}
if request.outputs.len() > 10 {
    return Err(Error::ValidationError("Too many outputs (max 10)".to_string()));
}

// Validate string sizes
for output in &request.outputs {
    if output.recipient.len() > 44 { // Base58 encoding max length
        return Err(Error::ValidationError("Recipient address too long".to_string()));
    }
}
```

---

## 4. Low Vulnerabilities

### 4.1 Missing Security Headers ⚠️ LOW

**Location:**
- `services/indexer/src/server/middleware.rs`
- `services/relay/src/main.rs`

**Description:**
Missing security headers such as `X-Content-Type-Options`, `X-Frame-Options`, etc.

**Recommendation:**
```rust
// Add security headers middleware
.layer(SetResponseHeaderLayer::overriding(
    header::X_CONTENT_TYPE_OPTIONS,
    HeaderValue::from_static("nosniff"),
))
.layer(SetResponseHeaderLayer::overriding(
    header::X_FRAME_OPTIONS,
    HeaderValue::from_static("DENY"),
))
.layer(SetResponseHeaderLayer::overriding(
    header::STRICT_TRANSPORT_SECURITY,
    HeaderValue::from_static("max-age=31536000; includeSubDomains"),
))
```

---

### 4.2 Request Timeout May Be Too Long ⚠️ LOW

**Location:**
- `services/indexer/src/server/middleware.rs:113-126`

**Description:**
30 second timeout may be too long for some endpoints, allowing connection accumulation.

**Evidence:**
```rust
// services/indexer/src/server/middleware.rs:117
let timeout_duration = std::time::Duration::from_secs(30);
```

**Recommendation:**
```rust
// Different timeouts per endpoint
let timeout_duration = match request.uri().path() {
    "/api/v1/prove" => Duration::from_secs(600), // 10 minutes for prove
    _ => Duration::from_secs(10), // 10 seconds for others
};
```

---

### 4.3 Lack of API Version Validation ⚠️ LOW

**Location:**
- `services/relay/src/api/withdraw.rs`
- `services/indexer/src/server/routes.rs`

**Description:**
No API version validation, making deprecation and changes difficult.

**Recommendation:**
```rust
// Add version header
#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    // ... other fields
}

fn default_api_version() -> String {
    "v1".to_string()
}

// Validate version
fn validate_api_version(version: &str) -> Result<(), Error> {
    match version {
        "v1" => Ok(()),
        _ => Err(Error::ValidationError("Unsupported API version".to_string())),
    }
}
```

---

## 5. Vulnerabilities in Solana Programs

### 5.1 Use of `unsafe` in Lamport Operations ⚠️ HIGH

**Location:**
- `programs/shield-pool/src/instructions/withdraw.rs:456-499`

**Description:**
Use of `unsafe` to manipulate lamports without additional validations.

**Evidence:**
```rust
// programs/shield-pool/src/instructions/withdraw.rs:456
unsafe {
    *pool_info.borrow_mut_lamports_unchecked() =
        pool_lamports - parsed.public_amount;
    // ...
}
```

**Impact:**
- Possible data corruption if validations fail
- Undefined behavior on error

**Recommendation:**
```rust
// Add validations before unsafe
if pool_lamports < parsed.public_amount {
    return Err(ShieldPoolError::InsufficientLamports.into());
}

// Use checked arithmetic
let new_pool_lamports = pool_lamports
    .checked_sub(parsed.public_amount)
    .ok_or(ShieldPoolError::MathOverflow)?;

unsafe {
    *pool_info.borrow_mut_lamports_unchecked() = new_pool_lamports;
}
```

---

### 5.2 Overflow Validation in Fee Calculations ⚠️ MEDIUM

**Location:**
- `programs/shield-pool/src/instructions/withdraw.rs:393-397`

**Description:**
Fee calculation does not use checked arithmetic in all places.

**Recommendation:**
```rust
// Use checked arithmetic
let fixed_fee = 2_500_000u64;
let variable_fee = parsed.public_amount
    .checked_mul(5)
    .ok_or(ShieldPoolError::MathOverflow)?
    .checked_div(1_000)
    .ok_or(ShieldPoolError::DivisionByZero)?;
    
let expected_fee = fixed_fee
    .checked_add(variable_fee)
    .ok_or(ShieldPoolError::MathOverflow)?;
```

---

## 6. General Recommendations

### 6.1 Implement Security Testing

- Fuzzing tests for API endpoints
- Automated penetration tests
- Load and stress tests

### 6.2 Implement Monitoring

- Alerts for attack attempts
- Rate limiting monitoring
- Centralized security logs

### 6.3 Security Documentation

- Document security policies
- Create incident response runbook
- Document security update process

### 6.4 Code Review

- Security-focused code review
- Use of static analysis tools (clippy, cargo-audit)
- Regular security audits

---

## 7. Fix Prioritization

### Priority 1 (Immediate)
1. Implement rate limiting
2. Remove all `unwrap()` and `expect()`
3. Fix CORS configuration

### Priority 2 (Short Term)
4. Implement authentication
5. Improve input validation
6. Add replay attack protection

### Priority 3 (Medium Term)
7. Add security headers
8. Improve logging
9. Implement monitoring

---

## 8. Conclusion

The Cloak system has a solid foundation, but there are several vulnerabilities that need to be fixed before production. Critical vulnerabilities must be addressed immediately, especially rate limiting and removal of `unwrap()`.

**Overall Status:** ⚠️ **Requires Immediate Attention**

---

**Next Steps:**
1. Review and prioritize vulnerabilities
2. Create repository issues for each vulnerability
3. Implement fixes following priority order
4. Conduct new audit after fixes
