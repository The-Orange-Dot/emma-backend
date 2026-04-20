# --- Builder Stage ---
FROM rust:1.87-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

RUN rm -rf src target/release/emma-backend*

COPY . .

RUN cargo build --release

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

RUN ls -lh /app/emma-backend

EXPOSE 4000
CMD ["./emma-backend"]