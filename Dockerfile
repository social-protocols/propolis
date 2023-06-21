FROM rust:1.70 as builder

# RUN apk add git cmake make g++ musl-dev openssl-dev sqlite-dev
RUN --mount=type=cache,target=/var/cache/apt \
    apt-get update && apt-get install --yes git cmake make g++ libssl-dev libsqlite3-dev

WORKDIR /propolis
COPY ./sqlite-vector ./sqlite-vector
RUN make -C sqlite-vector

COPY ./src ./src
COPY ./lib ./lib
COPY ./migrations ./migrations
COPY ./static ./static
COPY sqlx-data.json rust-toolchain.toml Cargo.toml Cargo.lock ./
RUN ls -alh .

# https://docs.docker.com/build/cache/#use-the-dedicated-run-cache
RUN --mount=type=cache,target=/usr/local/rustup \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=shared,target=./target \
    cargo fetch --locked
RUN --mount=type=cache,target=/usr/local/rustup \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=shared,target=./target \
    SQLX_OFFLINE=true cargo install --locked --path . --features embed_migrations,with_predictions

FROM debian:bullseye-slim
RUN apt-get update && apt-get install --yes ca-certificates libssl-dev openssl sqlite3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/propolis /usr/local/bin/propolis
COPY --from=builder /propolis/sqlite-vector/vector0.so /sqlite-vector/vector0.so
CMD ["propolis"]
