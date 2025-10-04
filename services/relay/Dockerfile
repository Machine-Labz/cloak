# Multi-stage Dockerfile for Cloak Relay Service (Rust)

# Builder stage
FROM rust:1.82-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    && rm -rf /var/cache/apk/*

# Set up the working directory
WORKDIR /app

# Copy workspace root files
COPY Cargo.toml Cargo.lock ./

# Copy packages directory
COPY packages ./packages/

# Copy programs directory
COPY programs ./programs/

# Copy services directory
COPY services ./services/

# Build the application in release mode
ENV CARGO_TARGET_DIR=/tmp/target
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release -p relay

# Runtime stage
FROM alpine:latest AS runtime

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    && rm -rf /var/cache/apk/*

# Create non-root user
RUN addgroup -g 1001 -S relay && \
    adduser -S relay -u 1001 -G relay

# Create necessary directories
RUN mkdir -p /app/logs && \
    chown -R relay:relay /app

# Copy the binary from builder stage
COPY --from=builder /tmp/target/release/relay /app/relay

# Copy migrations
COPY --from=builder /app/services/relay/migrations /app/migrations

# Set permissions
RUN chmod +x /app/relay && \
    chown relay:relay /app/relay

# Switch to non-root user
USER relay
WORKDIR /app

# Expose port
EXPOSE 3002

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3002/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Start the application
CMD ["./relay"]
