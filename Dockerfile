FROM rust:1.70-alpine3.18 as builder

WORKDIR /propolis
RUN apk add sqlite-dev cmake openssl-dev musl-dev make build-base

COPY ./src ./src
COPY ./lib ./lib
COPY ./migrations ./migrations
COPY ./static ./static
COPY sqlx-data.json rust-toolchain.toml Cargo.toml Cargo.lock ./
RUN ls -alh .

RUN cargo fetch --locked
RUN SQLX_OFFLINE=true cargo install --locked --path . --features embed_migrations,with_predictions

FROM alpine:3.18
COPY --from=builder /usr/local/cargo/bin/propolis /usr/local/bin/propolis
CMD ["propolis"]
