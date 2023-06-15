# <img src="logo.svg" width="24" /> Propolis

*(Early stage project)*

Enable useful discussions among thousands of people.

The idea is to create a discussion interface that allows to re-use the strategies people know from real-world one-to-one discussions. Different mechanics, like *askink a yes-no question*, *clarifying definitions or context* are provided.

Try it: <https://propolis.fly.dev>

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
