# --- Builder Stage ---
FROM rust:1.87-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev

WORKDIR /app

COPY . .

RUN cargo build --release --locked

# --- Runtime Stage ---
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/emma-backend /app/emma-backend
RUN chmod +x /app/emma-backend

EXPOSE 4000
CMD ["./emma-backend"]