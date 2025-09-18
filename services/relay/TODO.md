# Relay Service Implementation TODO

## Phase 1: Project Setup and Core Structure
- [ ] Initialize project with Cargo (Rust)
- [ ] Set up project structure:
  - [ ] `src/`
    - [ ] `main.rs` - Entry point and server setup
    - [ ] `api/` - API handlers and routes
    - [ ] `models/` - Data structures and validation
    - [ ] `queue/` - Job queue implementation
    - [ ] `solana/` - Solana client and transaction building
    - [ ] `config/` - Configuration management
    - [ ] `db/` - Database layer for job persistence
    - [ ] `metrics/` - Observability and monitoring

## Phase 2: Core Functionality
- [ ] Implement API Layer (`/withdraw` endpoint)
  - [ ] Request validation
  - [ ] Job creation and queuing
  - [ ] Response handling
- [ ] Implement Job Processing
  - [ ] Queue worker implementation
  - [ ] Job state management
  - [ ] Retry logic with backoff
- [ ] Implement Solana Integration
  - [ ] RPC client setup
  - [ ] Transaction building and signing
  - [ ] Transaction submission and confirmation

## Phase 3: Validation Layer
- [ ] Proof validation
  - [ ] Proof format verification
  - [ ] Public inputs validation
  - [ ] Merkle root verification
- [ ] Business logic validation
  - [ ] Outputs hash verification
  - [ ] Conservation check (sum(outputs) + fee == amount)
  - [ ] Nullifier check for double-spend prevention

## Phase 4: Reliability and Observability
- [ ] Implement persistence
  - [ ] Job state storage
  - [ ] Nullifier tracking
- [ ] Add metrics and logging
  - [ ] Request/response logging
  - [ ] Performance metrics
  - [ ] Error tracking
- [ ] Add health checks
  - [ ] Readiness probe
  - [ ] Liveness probe

## Phase 5: Testing
- [ ] Unit tests
- [ ] Integration tests
- [ ] Load testing
- [ ] E2E test with local validator

## Phase 6: Documentation and Deployment
- [ ] API documentation (OpenAPI/Swagger)
- [ ] Deployment configuration
- [ ] Monitoring setup
- [ ] Runbook/operational guide

## Dependencies
- [ ] axum/actix-web for HTTP server
- [ ] tokio for async runtime
- [ ] solana-client for Solana RPC
- [ ] redis/sqlx for job queue and persistence
- [ ] prometheus for metrics
- [ ] tracing for structured logging
