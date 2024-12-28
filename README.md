# kiwi
Experimental archetype based ECS

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

### Comparison results:
```bash
add_remove_component/apecs
                        time:   [1.9862 ms 1.9922 ms 1.9983 ms]
add_remove_component/bevy
                        time:   [863.65 µs 867.24 µs 870.27 µs]
add_remove_component/hecs
                        time:   [500.40 µs 501.11 µs 501.98 µs]
add_remove_component/kiwi
                        time:   [8.4957 ms 8.5127 ms 8.5310 ms]
add_remove_component/legion
                        time:   [1.9071 ms 1.9101 ms 1.9137 ms]
add_remove_component/planck_ecs
                        time:   [45.124 µs 45.187 µs 45.255 µs]
add_remove_component/shipyard
                        time:   [61.100 µs 61.358 µs 61.603 µs]
add_remove_component/specs
                        time:   [79.773 µs 79.949 µs 80.176 µs]

simple_iter/apecs       time:   [9.3105 µs 9.3212 µs 9.3343 µs]
simple_iter/bevy        time:   [11.329 µs 11.335 µs 11.341 µs]
simple_iter/hecs        time:   [6.8621 µs 6.8650 µs 6.8681 µs]
simple_iter/kiwi        time:   [39.774 µs 39.969 µs 40.178 µs]
simple_iter/legion      time:   [5.4838 µs 5.4904 µs 5.4970 µs]
simple_iter/planck_ecs  time:   [33.550 µs 33.566 µs 33.582 µs]
simple_iter/shipyard    time:   [16.206 µs 16.291 µs 16.370 µs]
simple_iter/specs       time:   [27.780 µs 27.793 µs 27.807 µs]

simple_insert/apecs     time:   [306.03 µs 306.21 µs 306.40 µs]
simple_insert/bevy      time:   [324.79 µs 326.03 µs 327.03 µs]
simple_insert/hecs      time:   [321.79 µs 321.99 µs 322.21 µs]
simple_insert/kiwi      time:   [18.616 ms 18.625 ms 18.636 ms]
simple_insert/legion    time:   [138.92 µs 138.99 µs 139.06 µs]
simple_insert/planck_ecs
                        time:   [255.58 µs 255.82 µs 256.06 µs]
simple_insert/shipyard  time:   [276.69 µs 277.07 µs 277.52 µs]
simple_insert/specs     time:   [979.55 µs 979.99 µs 980.48 µs]

frag_iter/apecs         time:   [391.26 ns 392.19 ns 393.11 ns]
frag_iter/bevy          time:   [1.3175 µs 1.3219 µs 1.3262 µs]
frag_iter/hecs          time:   [680.15 ns 693.64 ns 707.77 ns]
frag_iter/kiwi          time:   [2.6917 µs 2.7058 µs 2.7241 µs]
frag_iter/legion        time:   [227.33 ns 228.37 ns 229.95 ns]
frag_iter/planck_ecs    time:   [240.07 ns 240.26 ns 240.48 ns]
frag_iter/shipyard      time:   [45.535 ns 45.572 ns 45.606 ns]
frag_iter/specs         time:   [1.2201 µs 1.2238 µs 1.2276 µs]

schedule/apecs          time:   [76.053 µs 76.396 µs 76.756 µs]
schedule/bevy           time:   [64.412 µs 65.063 µs 65.657 µs]
schedule/kiwi           time:   [230.86 µs 231.08 µs 231.31 µs]
schedule/legion         time:   [53.391 µs 54.925 µs 56.442 µs]
schedule/planck_ecs     time:   [229.30 µs 230.91 µs 232.63 µs]
schedule/shipyard       time:   [80.612 µs 81.078 µs 81.554 µs]
schedule/specs          time:   [151.48 µs 152.17 µs 152.86 µs]

heavy_compute/apecs     time:   [310.14 µs 321.85 µs 333.56 µs]
heavy_compute/bevy      time:   [317.87 µs 319.76 µs 321.80 µs]
heavy_compute/hecs      time:   [364.92 µs 365.95 µs 367.10 µs]
heavy_compute/kiwi      time:   [334.02 µs 336.53 µs 339.00 µs]
heavy_compute/legion    time:   [290.82 µs 300.34 µs 310.03 µs]
heavy_compute/shipyard  time:   [291.29 µs 298.49 µs 306.13 µs]
heavy_compute/specs     time:   [415.14 µs 417.13 µs 419.19 µs]
```
