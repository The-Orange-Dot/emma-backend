FROM rust:1.87 as builder

RUN apt-get update \
    && apt-get install --no-install-recommends -y \
    && rustup target add x86_64-unknown-linux-musl

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/emma-backend ./

RUN addgroup -S appgroup && adduser -S appuser -G appgroup
USER appuser

CMD ["/app/emma-backend"]