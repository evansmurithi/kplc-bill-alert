FROM rust:1.64 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/kplc-bill-alert /usr/local/bin/kplc-bill-alert

CMD ["kplc-bill-alert"]
