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
    apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    curl \
    libc6 \
    libgcc-s1 \
    zlib1g \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/emma-backend /app/emma-backend
RUN chmod +x /app/emma-backend

RUN useradd -m appuser && \
    chown -R appuser:appuser /app
USER appuser

EXPOSE 4000
ENV RUST_LOG=info
ENV PYTHONUNBUFFERED=1 
ENV HOST=0.0.0.0
ENV PORT=4000

CMD ["./emma-backend"]