# Build stage
FROM rust:1.82-slim AS builder
WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /build/target/release/control-plane /app/control-plane

ENV PORT=8080
ENV DATABASE_URL=sqlite:///data/control-plane.db?mode=rwc
EXPOSE 8080

CMD ["/app/control-plane"]
