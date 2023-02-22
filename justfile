drop-db:
	sqlx database drop

create-db:
	mkdir -p data
	sqlx database create
	sqlx migrate run

reset-db:
	just drop-db
	just create-db

start:
	cargo run

develop:
	cargo watch -cx run