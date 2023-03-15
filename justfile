# Commands in here can be run using `just`, see https://just.systems/man/en/ for syntax etc.

# Run all commands using bash by default
set shell := ["bash", "-c"]

# List available recipes in the order in which they appear in this file
_default:
    @just --list --unsorted

# Delete the database file
drop-db:
	sqlx database drop

# Create and migrate database
create-db:
	sqlx database create
	sqlx migrate run

# Delete, recreate and migrate database
reset-db:
	sqlx database drop
	sqlx database create
	sqlx migrate run

migrate:
	sqlx migrate run

# Create sqlx-data.json file for sqlx offline mode
prepare-sqlx-offline-mode:
	cargo sqlx prepare

# Run server
start:
	cargo run

# Continuously build and run the project, watching for file changes
develop:
	cargo watch -cx run

fix:
  cargo fix --allow-dirty --allow-staged
  cargo fmt
  cargo sqlx prepare
  sqlite3 -init /dev/null data/data.sqlite '.schema' > schema.sql

install-fix-hook:
	echo "just fix > /dev/null 2>&1; git add sqlx-data.json > /dev/null" > .git/hooks/pre-commit
	chmod +x .git/hooks/pre-commit

# Run wrk HTTP benchmark against server running on localhost
benchmark:
	wrk -t8 -c100 -d20s --latency http://localhost:8000

# Run Apache bench HTTP benchmark against server running on localhost
benchmark-ab:
  ab -n 1000000 -c 100 -t 20 http://localhost:8000/

# delete local database, download production database
download-prod-db:
  rm -f data/data.sqlite
  rm -f data/data.sqlite-shm
  rm -f data/data.sqlite-wal
  flyctl ssh sftp get data/data.sqlite
  flyctl ssh sftp get data/data.sqlite-shm
  flyctl ssh sftp get data/data.sqlite-wal
