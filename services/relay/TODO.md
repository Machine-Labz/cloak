# Relay Service Implementation TODO

## Phase 1: Project Setup and Core Structure
- [x] Initialize project with Cargo (Rust)
- [x] Set up project structure:
  - [x] `src/`
    - [x] `main.rs` - Entry point and server setup
    - [x] `api/` - API handlers and routes
    - [x] `models/` - Data structures and validation
    - [x] `queue/` - Job queue implementation
    - [x] `solana/` - Solana client and transaction building
    - [x] `config/` - Configuration management
    - [x] `db/` - Database layer for job persistence
    - [x] `metrics/` - Observability and monitoring

## Phase 2: Core Functionality
- [x] Implement API Layer (`/withdraw` endpoint)
  - [x] Request validation
  - [x] Job creation and queuing
  - [x] Response handling
- [ ] Implement Job Processing
  - [ ] Queue worker implementation
  - [x] Job state management
  - [x] Retry logic with backoff
- [ ] Implement Solana Integration
  - [x] RPC client setup
  - [ ] Transaction building and signing
  - [x] Transaction submission and confirmation

## Phase 3: Validation Layer
- [ ] Proof validation
  - [ ] Proof format verification 
  - [x] Public inputs validation
  - [ ] Merkle root verification
- [x] Business logic validation
  - [x] Outputs hash verification
  - [x] Conservation check (sum(outputs) + fee == amount)
  - [x] Nullifier check for double-spend prevention

## Phase 4: Reliability and Observability
- [x] Implement persistence
  - [x] Job state storage
  - [x] Nullifier tracking
- [x] Add metrics and logging
  - [x] Request/response logging
  - [x] Performance metrics
  - [x] Error tracking
- [x] Add health checks
  - [x] Readiness probe
  - [x] Liveness probe

## Phase 5: Testing
- [x] Unit tests
- [x] Integration tests
- [ ] Load testing
- [ ] E2E test with local validator

## Phase 6: Documentation and Deployment
- [x] API documentation (OpenAPI/Swagger)
- [x] Deployment configuration
- [x] Monitoring setup
- [x] Runbook/operational guide

## Dependencies
- [x] axum/actix-web for HTTP server
- [x] tokio for async runtime
- [x] solana-client for Solana RPC
- [x] redis/sqlx for job queue and persistence
- [x] prometheus for metrics
- [x] tracing for structured logging