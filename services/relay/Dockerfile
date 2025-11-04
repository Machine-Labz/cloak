# Multi-stage Dockerfile for Cloak Relay Service
# Optimized build without SP1 toolchain installation (uses pre-built ELF)

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

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all source code (we'll use Cargo's incremental compilation for caching)
COPY packages ./packages/
COPY programs ./programs/
COPY services ./services/
COPY tooling ./tooling/

# Build with BuildKit cache mounts for maximum performance
# Cargo's incremental compilation will cache unchanged dependencies
ENV CARGO_TARGET_DIR=/tmp/target
ENV SQLX_OFFLINE=true
ENV DATABASE_URL=postgresql://cloak:password@postgres:5432/cloak
# Clean any stale build artifacts first
RUN rm -rf /tmp/target/release/build/sp1-* || true
# Build the workspace dependencies first, using pre-built ELF (no SP1 toolchain needed)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/tmp/target \
    cargo build --release --workspace --exclude relay -j 16 && \
    cargo build --release --bin relay -p relay -j 16 && \
    cp /tmp/target/release/relay /app/relay

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
COPY --from=builder --chown=relay:relay /app/relay /app/relay

# Switch to non-root user
USER relay

# Expose the application ports
EXPOSE 3002 9090

# Health check - check every 5 minutes (much less noisy)
HEALTHCHECK --interval=5m --timeout=10s --start-period=40s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3002/health || exit 1

# Run the application
CMD ["/app/relay"]
