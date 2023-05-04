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

seed:
  URL=http://localhost:8000 scripts/seed

# Create sqlx-data.json file for sqlx offline mode
prepare-sqlx-offline-mode:
	cargo sqlx prepare --merged

# Run server
start:
	cargo run

# Continuously build and run the project, watching for file changes
develop:
  process-compose -f process-compose-dev.yaml --tui=false up

test:
  cargo watch -cx 'test --workspace --all-targets --all-features'

fix:
  sqlx migrate run
  cargo sqlx prepare --merged
  sqlite3 -init /dev/null data/data.sqlite '.schema' > schema.sql
  cargo fix --allow-dirty --allow-staged --workspace --all-targets --all-features
  cargo clippy --fix --allow-dirty --allow-staged
  cargo fmt

install-fix-hook:
	echo "just fix > /dev/null 2>&1; git add sqlx-data.json schema.sql > /dev/null" > .git/hooks/pre-commit
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
  flyctl ssh sftp get data/data.sqlite || true
  flyctl ssh sftp get data/data.sqlite-shm || true
  flyctl ssh sftp get data/data.sqlite-wal || true
