reset-db:
	sqlx database drop
	sqlx database create
	sqlx migrate run

start:
	cargo run

develop:
	cargo watch -cx run