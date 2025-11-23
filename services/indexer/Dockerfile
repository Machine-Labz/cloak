# Dockerfile for Cloak Indexer Service
FROM rust:1.90-bullseye AS build

# Install build dependencies including lld (faster linker) and git
RUN apt-get update && apt-get install -y \
    ca-certificates \
    pkg-config \
    libssl-dev \
    libclang-dev \
    clang \
    lld \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Configure git to handle SSL and use CLI for fetching
RUN git config --global http.sslVerify true && \
    git config --global http.postBuffer 524288000

# Install SP1 toolchain ONLY to build the ELF binary (needed at compile time for include_elf! macro)
# For TEE usage, we don't need SP1 at runtime - only the ELF binary is embedded
RUN curl -L https://sp1.succinct.xyz | bash -s || true && \
    bash -c '\
        export PATH="/root/.cargo/bin:/usr/local/cargo/bin:$PATH" && \
        if [ -f "/root/.cargo/env" ]; then source "/root/.cargo/env"; fi && \
        if [ -f "/root/.zshenv" ]; then source "/root/.zshenv"; fi && \
        export PATH="/root/.cargo/bin:/usr/local/cargo/bin:$PATH" && \
        if ! command -v cargo-prove >/dev/null 2>&1; then \
            cargo install cargo-prove --locked 2>&1 | head -20 || true; \
        fi && \
        if command -v cargo-prove >/dev/null 2>&1; then \
            cargo prove install-toolchain 2>&1 | head -20 || echo "Toolchain install attempted"; \
        fi'

COPY . /src/cloak
WORKDIR /src/cloak

# Use lld linker for faster linking
ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld"
# Ensure SP1 tools are in PATH during build
ENV PATH="/root/.cargo/bin:/usr/local/cargo/bin:$PATH"
# Configure cargo to use git CLI for fetching (helps with SSL issues)
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

# Build with locked dependencies (use all available CPUs)
# Note: The ELF is embedded at compile time via include_elf! macro
# For TEE usage, we only need the ELF binary (not the proving toolchain at runtime)
# If pre-built ELF exists, it will be used; otherwise build-guest feature will build it
# Update git dependencies first to ensure lock file is current, then build
RUN mkdir /out && \
    cargo update -p sp1-solana && \
    cargo build --release -p indexer --locked -j $(nproc) && \
    cp target/release/indexer /out/

# Runtime stage
FROM debian:bullseye-slim

# Default environment variables
ENV INDEXER_HOST=0.0.0.0
ENV INDEXER_PORT=3001

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=build /out/ /bin/

WORKDIR /workspace

# Expose port
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3001/health || exit 1

# Run the indexer
CMD ["indexer"]
