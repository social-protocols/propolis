FROM rust:1.70-alpine3.18 as builder

WORKDIR /propolis
COPY . .
RUN apk add sqlite-dev cmake openssl-dev musl-dev make build-base
RUN cargo fetch --locked
RUN SQLX_OFFLINE=true cargo install --locked --path . --features embed_migrations,with_predictions

FROM alpine:3.18
COPY --from=builder /root/.cargo/bin/propolis /usr/local/bin/propolis
RUN apk add sqlite
CMD ["propolis"]
