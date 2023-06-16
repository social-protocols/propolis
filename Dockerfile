FROM rust:1.70-alpine3.18 as builder

RUN apk add git cmake make g++ musl-dev openssl-dev sqlite-dev

WORKDIR /propolis
COPY ./src ./src
COPY ./lib ./lib
COPY ./migrations ./migrations
COPY ./static ./static
COPY sqlx-data.json rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY ./sqlite-vector ./sqlite-vector
RUN ls -alh .

RUN make -C sqlite-vector
RUN cargo fetch --locked
RUN SQLX_OFFLINE=true cargo install --locked --path . --features embed_migrations,with_predictions

FROM alpine:3.18
COPY --from=builder /usr/local/cargo/bin/propolis /usr/local/bin/propolis
COPY --from=builder /propolis/sqlite-vector/vector0.so /usr/local/bin/sqlite-vector/vector0.so
CMD ["propolis"]
