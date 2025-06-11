# --- Builder Stage ---
# Use a specific, stable Rust version to ensure reproducible builds.
# 1.87 is a very recent version, ensure it's suitable for your project.
FROM rust:1.87 AS builder

# Install necessary system dependencies for building Rust applications,
# especially those interacting with SSL (like many web apps).
# --no-install-recommends keeps the image smaller.
RUN apt-get update && \
    apt-get install --no-install-recommends -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only Cargo.toml and Cargo.lock first to leverage Docker's build cache.
# If these files don't change, subsequent builds can skip re-downloading dependencies.
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs and run a build to cache dependencies.
# This makes subsequent builds faster if only source code changes.
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the rest of your application's source code.
COPY . .

# Build your Rust application in release mode.
# This will produce an optimized binary.
RUN cargo build --release

# --- Runtime Stage ---
# Use a slim Debian image for the final runtime, which is smaller than the full Rust image.
FROM debian:bookworm-slim

# Install curl for Coolify's health check. This is crucial for the warning you received.
# Clean up apt lists immediately to keep the image size down.
RUN apt-get update && \
    apt-get install -y curl && \
    rm -rf /var/lib/apt/lists/*

# Install openssl and ca-certificates for secure communication (e.g., HTTPS, database connections).
# This is often needed by Rust applications that make outgoing network requests.
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    openssl \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Define environment variables for your application.
# These will be available inside the running container.
# Consider using Coolify's secret management for sensitive values like database URLs
# rather than hardcoding them in the Dockerfile if they change frequently or are sensitive.
ENV RUST_LOG=info
ENV SERVER_URL=http://100.79.35.30:11434/api/generate
ENV POSTGRES_URL=100.100.140.27:5432
ENV LLM_MODEL=gemma3:12b
ENV POOL_CLEANUP_INTERVAL_SECS=300   
ENV POOL_IDLE_TIMEOUT_SECS=1800       
ENV DEMO_ACCOUNT_ID=fa1fe6d9-beae-45be-80da-aad89b147ea0
ENV APP_ENV=production
ENV DATABASE_URL=postgres://postgres:WfkropE5AonK6grsHndmcVAgrNswE8Tt@100.100.140.27:5432

# Expose the port your Rust application listens on.
# This tells Docker that the container intends to listen on this port.
EXPOSE 8080

# Copy the compiled Rust binary from the builder stage into the runtime image.
# Renaming it to `emma-backend` at the destination for clarity.
COPY --from=builder /app/target/release/emma-backend /app/emma-backend

# Copy TLS certificates if your application handles HTTPS directly.
# Ensure these files are present in the same directory as your Dockerfile.
COPY cert.pem /app/cert.pem
COPY key.pem /app/key.pem

# Create a non-root user for security best practices.
# Running as root inside a container is generally discouraged.
RUN useradd -m appuser && \
    chown -R appuser:appuser /app
USER appuser

# Define a health check for Coolify.
# This command runs periodically (every 30s) and fails if it doesn't get a response
# within 3 seconds, or if `curl` returns a non-zero exit code (e.g., 4xx/5xx HTTP status).
# This is excellent for Coolify to determine if your app is truly healthy.
HEALTHCHECK --interval=30s --timeout=3s \
  CMD curl -f http://localhost:8080/health || exit 1

# The command to execute when the container starts.
# This will run your compiled Rust application.
CMD ["/app/emma-backend"]