# Multi-stage Dockerfile for Cloak Relay Service
# Using Debian for better compatibility

# Builder stage
FROM rust:1.90-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Set up the working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY packages ./packages/
COPY programs ./programs/
COPY services ./services/
COPY tooling ./tooling/

# Build the relay with SQLX offline mode
ENV CARGO_TARGET_DIR=/tmp/target
ENV SQLX_OFFLINE=true
ENV DATABASE_URL=postgresql://cloak:password@postgres:5432/cloak_relay
RUN cargo build --release -p relay

# Runtime stage
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -g 1001 relay && \
    useradd -r -u 1001 -g relay relay

# Create necessary directories
RUN mkdir -p /app/logs && \
    chown -R relay:relay /app

# Set working directory
WORKDIR /app

# Copy the built binary from builder stage
COPY --from=builder --chown=relay:relay /tmp/target/release/relay /app/relay

# Copy migrations
COPY --chown=relay:relay services/relay/migrations /app/migrations

# Switch to non-root user
USER relay

# Expose the application ports
EXPOSE 3002 9090

# Health check - check every 5 minutes (much less noisy)
HEALTHCHECK --interval=5m --timeout=10s --start-period=40s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3002/health || exit 1

# Run the application
CMD ["/app/relay"]
