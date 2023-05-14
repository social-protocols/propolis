FROM messense/rust-musl-cross:x86_64-musl as builder
WORKDIR /propolis
COPY . .
RUN SQLX_OFFLINE=true cargo install --locked --path . --features embed_migrations

FROM alpine:3.17
COPY --from=builder /root/.cargo/bin/propolis /usr/local/bin/propolis
RUN apk add sqlite
CMD ["propolis"]
