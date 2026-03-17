# TEA Server Dockerfile
#
# Multi-stage build for minimal production image.
# Build: docker build -t tea-server .
# Run:   docker run -p 8734:8734 tea-server

# ─────────────────────────────────────────────────────────────────────────────
# Build stage
# ─────────────────────────────────────────────────────────────────────────────
FROM rust:1.75-bookworm AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libprotobuf-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Create dummy src to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source
COPY src ./src
COPY migrations ./migrations
COPY tests ./tests

# Build the actual binary
RUN cargo build --release --locked

# ─────────────────────────────────────────────────────────────────────────────
# Runtime stage - minimal image
# ─────────────────────────────────────────────────────────────────────────────
FROM gcr.io/distroless/cc-debian12:latest AS runtime

# Copy the binary
COPY --from=builder /build/target/release/tea-server /tea-server

# Copy migrations for startup
COPY --from=builder /build/migrations /migrations

# Non-root user (distroless has nobody)
USER nobody

# Expose default port
EXPOSE 8734

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/tea-server", "health"] || exit 1

# Default environment
ENV TEA_PORT=8734
ENV TEA_SERVER_URL=http://localhost:8734

# Entry point
ENTRYPOINT ["/tea-server"]
