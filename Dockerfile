# =============================================================================
# UBL Server - Dockerfile for Railway Deployment
# =============================================================================
# Multi-stage build for optimized production image

# Stage 1: Build stage
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy the entire workspace
COPY . .

# Build the release binary
WORKDIR /app/kernel/rust
RUN cargo build --release -p ubl-server

# Stage 2: Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1001 ubl

# Set working directory
WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/kernel/rust/target/release/ubl-server /app/ubl-server

# Copy SQL migrations
COPY --from=builder /app/sql /app/sql

# Set ownership
RUN chown -R ubl:ubl /app

# Switch to non-root user
USER ubl

# Expose port (Railway will set PORT env var)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["/app/ubl-server"]
