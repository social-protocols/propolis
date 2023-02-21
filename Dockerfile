FROM rust:1.67 as builder
WORKDIR /propolis
COPY . .
RUN SQLX_OFFLINE=true cargo install --path . --features embed_migrations

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/propolis /usr/local/bin/propolis
CMD ["propolis"]
