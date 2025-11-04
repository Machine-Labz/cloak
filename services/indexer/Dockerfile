# Dockerfile for Cloak Indexer Service
FROM rust:1.90-bullseye AS build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    pkg-config \
    libssl-dev \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . /src/cloak
WORKDIR /src/cloak

# Build with locked dependencies for reproducibility
RUN mkdir /out && \
    cargo build --release --bin indexer --locked -j 8 && \
    cp /src/cloak/target/release/indexer /out/

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
