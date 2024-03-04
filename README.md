# kiwi
Archetype based ECS

## Examples

```bash
$ cargo run --release --example simple
```

### Tracing
```bash
$ cargo run --release --example tracing --no-default-features --features trace_release_max_level_trace
```

This will output `trace-*.json` and `tracing.folded`.
- `trace-*.json` can be viewed in chrome based browser at `chrome://tracing/`.
- `tracing.folded` can be converted to svg with `cat tracing.folded | inferno-flamegraph > tracing-flamegraph.svg`

## Benches

```bash
$ cargo bench
```
