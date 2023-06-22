# Better support of Docker layer caching in Cargo:
# https://hackmd.io/@kobzol/S17NS71bh#Using-cargo-chef
# https://github.com/LukeMathWalker/cargo-chef#without-the-pre-built-image


# only install cargo-chef, to be reused in other stages
FROM rust:1.70 as chef
# install cargo-chef and trigger a download of the rustup toolchain
RUN cargo install cargo-chef && cargo chef --version
WORKDIR app



# only prepares the build plan
FROM chef as planner
COPY . .
# Prepare a build plan ("recipe")
RUN cargo chef prepare --recipe-path recipe.json



# build the project with a cached dependency layer
FROM chef as builder
# for alpine: RUN apk add git cmake make g++ musl-dev openssl-dev sqlite-dev
RUN apt-get update && apt-get install --yes git cmake make g++ libssl-dev libsqlite3-dev
# build sqlite-vector extension
COPY ./sqlite-vector ./sqlite-vector
RUN make -C sqlite-vector
# Copy the build plan from the previous Docker stage
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this layer is cached as long as `recipe.json` doesn't change.
RUN cargo chef cook --release --recipe-path recipe.json
# Build the full project
COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY ./src ./src
COPY sqlx-data.json ./
COPY ./lib ./lib
COPY ./migrations ./migrations
COPY ./static ./static
RUN SQLX_OFFLINE=true cargo build --locked --release --features embed_migrations



# copy the binary and sqlite-vector extension to a minimal image
FROM debian:bullseye-slim
RUN apt-get update && apt-get install --yes ca-certificates libssl-dev openssl sqlite3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/propolis /usr/local/bin/app
COPY --from=builder /app/sqlite-vector/vector0.so /sqlite-vector/vector0.so
CMD ["app"]
