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

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    openssl \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

ENV RUST_LOG=info

EXPOSE 8080

COPY --from=builder /app/target/release/emma-backend .

RUN useradd -m appuser && \
    chown -R appuser:appuser /app
USER appuser

HEALTHCHECK --interval=30s --timeout=3s \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["/app/emma-backend"]

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    curl \
    iproute2 \
    ca-certificates && \
    curl -fsSL https://tailscale.com/install.sh | sh && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/emma-backend .

CMD ["sh", "-c", "tailscaled --state=/var/lib/tailscale/tailscaled.state & \
     sleep 5 && \
     tailscale up --authkey=${TAILSCALE_AUTHKEY} && \
     /app/emma-backend"]

# docker build --platform linux/amd64 -t orangedot/emma-backend:v1 .

# docker run -it --rm \
#   --privileged \
#   --cap-add=NET_ADMIN \
#   -v /dev/net/tun:/dev/net/tun \
#   -e TAILSCALE_AUTHKEY=tskey-auth-kK6S4MagTj11CNTRL-fD9AqG7uwve5FZCWruxoveRymzFitrmc \
#   -e DATABASE_URL=postgres://postgres:WfkropE5AonK6grsHndmcVAgrNswE8Tt@100.100.140.27:5432 \
#   orangedot/emma-backend:v1