# propolis

```
sqlx database create
sqlx migrate run

cargo run
```
open browser https://localhost:8000

## Benchmarking

Start release web server:

```
cargo run --release
```

Then benchmark with your preferred tool.

Using [wrk](https://github.com/wg/wrk):

```
wrk -t8 -c100 -d20s --latency http://localhost:8000
```

Using Apache Bench:

```
ab -n 1000000 -c 100 -t 20 http://localhost:8000/
```
