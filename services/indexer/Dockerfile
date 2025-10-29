# Multi-stage Dockerfile for Cloak Indexer Service
# Using Debian instead of Alpine for better SP1 toolchain compatibility

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

# Pre-download SP1 Groth16 circuit artifacts to avoid 2GB download at runtime
# These are needed for zkProof generation and take several minutes to download
RUN echo "Pre-downloading SP1 Groth16 circuit artifacts for v5.0.0..." && \
    mkdir -p ${SP1_DIR}/circuits/groth16/v5.0.0 && \
    cd ${SP1_DIR}/circuits/groth16/v5.0.0 && \
    curl -L -o groth16_vk.bin "https://sp1-circuits.s3.us-east-2.amazonaws.com/v5.0.0/groth16_vk.bin" && \
    curl -L -o groth16_pk.bin "https://sp1-circuits.s3.us-east-2.amazonaws.com/v5.0.0/groth16_pk.bin" && \
    echo "✓ SP1 Groth16 circuits downloaded successfully"

# Set up the working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY packages ./packages/
COPY programs ./programs/
COPY services ./services/
COPY tooling ./tooling/

# Build the indexer
# SP1 build script will use the succinct toolchain automatically
# Use a unique target dir to avoid SP1 build script race conditions
ENV CARGO_TARGET_DIR=/tmp/target
# Clean any stale build artifacts that might cause "File exists" errors
RUN rm -rf /tmp/target/release/build/sp1-* || true
# Build with a single job to avoid parallel build race conditions in SP1

RUN cargo build --release -p indexer -j 32

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

# Create necessary directories including SP1 cache directory
RUN mkdir -p /app/logs /app/artifacts /home/indexer/.sp1/circuits && \
    chown -R indexer:indexer /app /home/indexer

# Set working directory
WORKDIR /app

# Copy the built binary from builder stage
COPY --from=builder --chown=indexer:indexer /tmp/target/release/indexer /app/indexer

# Switch to non-root user
USER indexer

# Expose the application port
EXPOSE 3001

# Health check - check every 5 minutes (much less noisy)
HEALTHCHECK --interval=5m --timeout=10s --start-period=40s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3001/health || exit 1

# Run the application
CMD ["/app/indexer"]
