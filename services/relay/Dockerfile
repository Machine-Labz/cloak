# Dockerfile for Cloak Relay Service
FROM rust:1.90-bullseye AS build

# Install build dependencies including lld (faster linker)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    pkg-config \
    libssl-dev \
    libclang-dev \
    clang \
    lld \
    && rm -rf /var/lib/apt/lists/*

COPY . /src/cloak
WORKDIR /src/cloak

# Use lld linker for faster linking
ENV RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Build with locked dependencies (use all available CPUs)
RUN mkdir /out && \
    cargo build --release -p relay --locked -j $(nproc) && \
    cp target/release/relay /out/

# Runtime stage
FROM debian:bullseye-slim

# Default environment variables
ENV RELAY_HOST=0.0.0.0
ENV RELAY_PORT=3002
ENV METRICS_PORT=9090

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=build /out/ /bin/

WORKDIR /workspace

# Expose ports
EXPOSE 3002 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3002/health || exit 1

# Run the relay
CMD ["relay"]
