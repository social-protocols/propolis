# FROM messense/rust-musl-cross:x86_64-musl as builder
FROM rust as builder
WORKDIR /propolis
COPY . .
RUN apt-get update
RUN apt-get -y install openssl libssl-dev libsqlite3-dev cmake
RUN cargo fetch --locked
RUN SQLX_OFFLINE=true cargo install --locked --path . --features embed_migrations,with_predictions

FROM alpine:3.17
COPY --from=builder /root/.cargo/bin/propolis /usr/local/bin/propolis
RUN apk add sqlite
CMD ["propolis"]
