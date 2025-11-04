# Multi-stage Dockerfile for Cloak Indexer Service
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
# Clean any stale build artifacts first
RUN rm -rf /tmp/target/release/build/sp1-* || true
# Build the workspace dependencies first, using pre-built ELF (no SP1 toolchain needed)
# This ensures all dependencies are compiled correctly before building the indexer
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/tmp/target \
    cargo build --release --workspace --exclude indexer -j 16 && \
    cargo build --release --bin indexer -p indexer -j 16 && \
    cp /tmp/target/release/indexer /app/indexer

# Runtime stage
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies including Docker
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    wget \
    curl \
    gnupg \
    lsb-release \
    && rm -rf /var/lib/apt/lists/*

# Install Docker
RUN curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null \
    && apt-get update \
    && apt-get install -y docker-ce-cli \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -g 1001 indexer && \
    useradd -r -u 1001 -g indexer indexer

# Create necessary directories
RUN mkdir -p /app/logs /app/artifacts && \
    chown -R indexer:indexer /app

# Set working directory
WORKDIR /app

# Copy the built binary from builder stage
COPY --from=builder --chown=indexer:indexer /app/indexer /app/indexer

# Switch to non-root user
USER indexer

# Expose the application port
EXPOSE 3001

# Health check - check every 5 minutes (much less noisy)
HEALTHCHECK --interval=5m --timeout=10s --start-period=40s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3001/health || exit 1

# Run the application
CMD ["/app/indexer"]
