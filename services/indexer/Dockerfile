# Multi-stage Dockerfile for Cloak Indexer Service (Rust)

# Builder stage
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    && rm -rf /var/cache/apk/*

# Set up the working directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application in release mode
ENV CARGO_TARGET_DIR=/tmp/target
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release --bin indexer

# Runtime stage
FROM alpine:latest AS runtime

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    && rm -rf /var/cache/apk/*

# Create non-root user
RUN addgroup -g 1001 -S indexer && \
    adduser -S indexer -u 1001 -G indexer

# Create necessary directories
RUN mkdir -p /app/logs /app/artifacts && \
    chown -R indexer:indexer /app

# Copy the binary from builder stage
COPY --from=builder /tmp/target/release/indexer /app/indexer

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Set permissions
RUN chmod +x /app/indexer && \
    chown indexer:indexer /app/indexer

# Switch to non-root user
USER indexer
WORKDIR /app

# Expose port
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3001/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Start the application
CMD ["./indexer"]
