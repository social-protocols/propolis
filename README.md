# <img src="logo.svg" width="24" /> Propolis
Enable useful discussions among thousands of people.

Try it: <https://propolis.fly.dev>

## User Flow

```mermaid
flowchart TD
    link[User arrives via Statement Link] --> statement
    link --> create
    statement[View Statement] --> vote
    statement --> depends
    vote[Vote Yes/No]
    vote --> statement
    depends[It Depends] --> create
    create[Add New Statement]
```

## Development

```bash
just reset-db
just develop
```
Open in browser: <https://localhost:8000>

## Benchmarking

Start release web server:

```bash
cargo run --release
```

Then benchmark:

```bash
just benchmark
```
