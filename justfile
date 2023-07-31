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

# watch migrations and show diff of schema.sql
schema-diff:
  scripts/schema-diff

update-schema:
  scripts/sorted_schema > schema.sql

seed:
  URL=http://localhost:8000 scripts/seed

# Create ./.sqlx folder for sqlx offline mode
prepare-sqlx-offline-mode:
	cargo sqlx prepare --workspace -- --all-targets --all-features

# Run server
start:
	cargo run

# Continuously build and run the project, watching for file changes
develop:
  process-compose -f process-compose-dev.yaml --tui=false up

test:
  cargo watch -cx 'test --workspace --all-targets --all-features'

fix:
  echo "Make sure no other compilers are running at the same time (e.g. just develop)"
  sqlx migrate run
  just prepare-sqlx-offline-mode
  just update-schema
  cargo fix --allow-dirty --allow-staged --workspace --all-targets --all-features
  cargo clippy --fix --allow-dirty --allow-staged --workspace --all-targets --all-features
  cargo fmt

install-fix-hook:
	echo "just fix > /dev/null 2>&1; git add .sqlx schema.sql > /dev/null" > .git/hooks/pre-commit
	chmod +x .git/hooks/pre-commit

# Run wrk HTTP benchmark against server running on localhost
benchmark:
	wrk -t8 -c100 -d20s --latency http://localhost:8000

# Run Apache bench HTTP benchmark against server running on localhost
benchmark-ab:
  ab -n 1000000 -c 100 -t 20 http://localhost:8000/

# delete local database, download production database
download-prod-db:
  rm -f "$DATABASE_FILE"
  rm -f "$DATABASE_FILE"-shm
  rm -f "$DATABASE_FILE"-wal
  flyctl ssh console -C "sqlite3 /data/data.sqlite '.backup /data/backup.sqlite'"
  flyctl ssh sftp get data/backup.sqlite "$DATABASE_FILE" || true
