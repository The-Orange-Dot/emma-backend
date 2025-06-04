FROM rust:1.87 AS builder

RUN apt-get update \
    && apt-get install --no-install-recommends -y pkg-config libssl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends openssl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m appuser
USER appuser

COPY --from=builder /app/target/release/emma-backend /app/

HEALTHCHECK --interval=30s --timeout=3s \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["/app/emma-backend"]

# docker build --platform linux/amd64 -t orangedot/emma-backend:v1 .
