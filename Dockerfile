# Build stage
FROM rust:latest AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates gosu && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/crooner /usr/local/bin/crooner

# Create directory for backups
RUN mkdir -p /backups

# Copy default config (will be overridden by volume mount)
COPY config.toml /app/config.toml

# Create non-root user and docker group
RUN groupadd -g 999 docker && \
    useradd -m -u 1000 -G docker crooner && \
    chown -R crooner:crooner /app /backups

# Copy and setup entrypoint
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["crooner"]
