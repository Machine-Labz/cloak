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

# Install SP1
ENV SP1_DIR=/root/.sp1
ENV PATH="/root/.sp1/bin:${PATH}"
ENV SP1_VERSION=v5.2.2

RUN mkdir -p ${SP1_DIR}/bin

# Download and install cargo-prove
RUN ARCHITECTURE="$(uname -m)" && \
    if [ "${ARCHITECTURE}" = "x86_64" ]; then ARCHITECTURE="amd64"; fi && \
    if [ "${ARCHITECTURE}" = "aarch64" ]; then ARCHITECTURE="arm64"; fi && \
    echo "Downloading cargo-prove ${SP1_VERSION} for linux ${ARCHITECTURE}" && \
    curl -L "https://github.com/succinctlabs/sp1/releases/download/${SP1_VERSION}/cargo_prove_${SP1_VERSION}_linux_${ARCHITECTURE}.tar.gz" | \
    tar -xzC ${SP1_DIR}/bin

# Install the succinct toolchain
RUN cargo-prove prove install-toolchain

# Verify toolchains
RUN rustup toolchain list

# Set up the working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY packages ./packages/
COPY programs ./programs/
COPY services ./services/
COPY tooling ./tooling/

# Build the relay with SQLX offline mode
# Use a unique target dir to avoid SP1 build script race conditions
ENV CARGO_TARGET_DIR=/tmp/target
# Clean any stale build artifacts that might cause "File exists" errors
RUN rm -rf /tmp/target/release/build/sp1-* || true
# Build with a single job to avoid parallel build race conditions in SP1
ENV SQLX_OFFLINE=true
ENV DATABASE_URL=postgresql://cloak:password@postgres:5432/cloak

RUN cargo build --release -p relay -j 4

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

# Switch to non-root user
USER relay

# Expose the application ports
EXPOSE 3002 9090

# Health check - check every 5 minutes (much less noisy)
HEALTHCHECK --interval=5m --timeout=10s --start-period=40s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3002/health || exit 1

# Run the application
CMD ["/app/relay"]
