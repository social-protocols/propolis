FROM rust:1.70 as builder

# RUN apk add git cmake make g++ musl-dev openssl-dev sqlite-dev
RUN apt update
RUN apt install --yes git cmake make g++ libssl-dev libsqlite3-dev

WORKDIR /propolis
COPY ./src ./src
COPY ./lib ./lib
COPY ./migrations ./migrations
COPY ./static ./static
COPY sqlx-data.json rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY ./sqlite-vector ./sqlite-vector
RUN ls -alh .

RUN make -C sqlite-vector
RUN --mount=type=cache,target=./.cargo \
    --mount=type=cache,sharing=private,target=./target \
    cargo fetch --locked
RUN --mount=type=cache,target=./.cargo \
    --mount=type=cache,sharing=private,target=./target \
    SQLX_OFFLINE=true cargo install --locked --path . --features embed_migrations,with_predictions

FROM debian:bullseye-slim
RUN apt-get update && apt-get install --yes tini libssl-dev openssl sqlite3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/propolis /usr/local/bin/propolis
COPY --from=builder /propolis/sqlite-vector/vector0.so /sqlite-vector/vector0.so
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["propolis"]
