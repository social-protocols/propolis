# FROM messense/rust-musl-cross:x86_64-musl as builder
FROM rust as builder
WORKDIR /propolis
COPY . .
RUN apt-get update
RUN apt-get -y install pkg-config cmake
RUN apt-get -y install openssl libssl-dev libsqlite3-dev sqlite3 gcc-multilib g++-multilib
RUN cmake --find-package -DNAME=SQLite3 -DCOMPILER_ID=GNU -DLANGUAGE=C -DMODE=EXIST
RUN pkg-config --cflags sqlite3
RUN SQLX_OFFLINE=true cargo install --locked --path . --features embed_migrations,with_predictions

FROM alpine:3.17
COPY --from=builder /root/.cargo/bin/propolis /usr/local/bin/propolis
RUN apk add sqlite
CMD ["propolis"]
