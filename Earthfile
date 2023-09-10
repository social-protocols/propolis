VERSION 0.7
FROM nixpkgs/nix-flakes:nixos-23.05
WORKDIR /app

nix-shell-rust:
  COPY flake.nix flake.lock rust-toolchain.toml /flake
  # warm up nix cache by running "echo"
  RUN nix develop "/flake#buildRust" --command echo

prepare-cache:
    FROM +nix-shell-rust
    COPY --dir src Cargo.lock Cargo.toml .
    RUN cargo chef prepare
    SAVE ARTIFACT recipe.json

# Using cutoff-optimization to ensure cache hit (see examples/cutoff-optimization)
build-cache:
    FROM +install-chef
    COPY +prepare-cache/recipe.json ./
    CACHE target
    RUN cargo chef cook --release

build:
    FROM +build-cache
    COPY --dir src Cargo.lock Cargo.toml .
    RUN cargo build --release --bin example-rust
    SAVE ARTIFACT target/release/example-rust example-rust

docker:
    FROM debian:buster-slim
    COPY +build/example-rust example-rust
    EXPOSE 9091
    ENTRYPOINT ["./example-rust"]
    SAVE IMAGE --push earthly/examples:rust
