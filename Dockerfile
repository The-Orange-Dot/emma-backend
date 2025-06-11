FROM rust:1.87 AS builder

RUN apt-get update && \
    apt-get install --no-install-recommends -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY . .

RUN cargo build --release

# --- Runtime Stage ---
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y curl && \
    rm -rf /var/lib/apt/lists/*

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    openssl \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

ENV RUST_LOG=info
ENV SERVER_URL=http://100.79.35.30:11434/api/generate
ENV POSTGRES_URL=100.100.140.27:5432
ENV LLM_MODEL=gemma3:12b
ENV POOL_CLEANUP_INTERVAL_SECS=300   
ENV POOL_IDLE_TIMEOUT_SECS=1800       
ENV DEMO_ACCOUNT_ID=fa1fe6d9-beae-45be-80da-aad89b147ea0
ENV APP_ENV=production
ENV DATABASE_URL=postgres://postgres:WfkropE5AonK6grsHndmcVAgrNswE8Tt@100.100.140.27:5432

EXPOSE 8080

COPY --from=builder /app/target/release/emma-backend /app/emma-backend

COPY cert.pem /app/cert.pem
COPY key.pem /app/key.pem

RUN useradd -m appuser && \
    chown -R appuser:appuser /app
USER appuser

HEALTHCHECK --interval=30s --timeout=3s --start-period=60s \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["/app/emma-backend"]