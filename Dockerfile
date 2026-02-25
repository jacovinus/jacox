# --- Builder Stage ---
FROM rust:latest as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main to pre-build dependencies and cache them
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/jacox*

# Copy source and static assets
COPY src ./src
COPY static ./static

# Build the real binary
RUN cargo build --release

# --- Runner Stage ---
FROM debian:bookworm-slim

# Install runtime dependencies (OpenSSL is required by reqwest)
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/jacox /usr/local/bin/jacox

# Copy default config (can be overridden by volume)
COPY config.yaml ./config.yaml
COPY static ./static

# Set default host to 0.0.0.0 to be accessible from outside the container
ENV JACOX_SERVER_HOST=0.0.0.0

EXPOSE 8080

ENTRYPOINT ["jacox"]
CMD ["serve"]
