<div align="center" style="border-bottom: none">
        <a href="https://propolis.fly.dev">
            <img src="logo.svg" width="80" />
        </a>
</div>


# Propolis
Enable useful discussions among thousands of people.

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
