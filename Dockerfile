FROM rust:1.93-bookworm AS builder

WORKDIR /app
COPY rust-backend ./rust-backend
COPY public ./public

WORKDIR /app/rust-backend
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/rust-backend/target/release/rvideo-rust-backend /usr/local/bin/rvideo-rust-backend
COPY --from=builder /app/public ./public

EXPOSE 3000

ENV HOST=0.0.0.0
ENV PORT=3000

CMD ["rvideo-rust-backend"]
