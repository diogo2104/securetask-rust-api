FROM rust:1.99-bookworm AS builder
WORKDIR /app

COPY Cargo.toml ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 appuser

COPY --from=builder /app/target/release/securetask-api /usr/local/bin/securetask-api
USER appuser
EXPOSE 8080
ENTRYPOINT ["securetask-api"]
