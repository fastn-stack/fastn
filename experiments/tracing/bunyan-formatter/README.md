# tracing-bunyan-formatter

tracing-bunyan-formatter provides two Layers implementation to be used on top 
of a tracing Subscriber:

bunyan: https://github.com/trentm/node-bunyan

- JsonStorageLayer, to attach contextual information to spans for ease of 
  consumption by downstream Layers, via JsonStorage and Span's extensions;
- BunyanFormattingLayer`, which emits a bunyan-compatible formatted record 
  upon entering a span, existing a span and event creation.


The main benefit of this
- giving structured logs
- event and span status
- hostname
- pid
- time
- event line
- args
- elapsed_milliseconds

## Console Output

```rust
#[instrument]
pub fn a_unit_of_work(first_parameter: u64) {
    for i in 0..2 {
        a_sub_unit_of_work(i);
    }
    info!(excited = "true", "Tracing is quite cool!");
}

#[instrument]
pub fn a_sub_unit_of_work(sub_parameter: u64) {
    info!("Events have the full context of their parent span!");
    info!("going to sleep");
    std::thread::sleep(std::time::Duration::from_secs(1));
    info!("function waked up");
}

fn main() {
    let formatting_layer = BunyanFormattingLayer::new("tracing_demo".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(JsonStorageLayer)
        .with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    info!("Orphan event without a parent span");
    a_unit_of_work(2);
}

```

```shell
cargo run
```

```shell
{"v":0,"name":"tracing_demo","msg":"Orphan event without a parent span","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:45.666846Z","target":"bunyan_formatter","line":30,"file":"src/main.rs"}
{"v":0,"name":"tracing_demo","msg":"[A_UNIT_OF_WORK - START]","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:45.666996Z","target":"bunyan_formatter","line":7,"file":"src/main.rs","first_parameter":2}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - START]","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:45.667064Z","target":"bunyan_formatter","line":15,"file":"src/main.rs","first_parameter":2,"sub_parameter":0}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - EVENT] Events have the full context of their parent span!","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:45.667125Z","target":"bunyan_formatter","line":17,"file":"src/main.rs","first_parameter":2,"sub_parameter":0}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - EVENT] going to sleep","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:45.667191Z","target":"bunyan_formatter","line":18,"file":"src/main.rs","first_parameter":2,"sub_parameter":0}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - EVENT] function waked up","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:46.670041Z","target":"bunyan_formatter","line":20,"file":"src/main.rs","first_parameter":2,"sub_parameter":0}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - END]","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:46.67024Z","target":"bunyan_formatter","line":15,"file":"src/main.rs","elapsed_milliseconds":1003,"first_parameter":2,"sub_parameter":0}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - START]","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:46.670439Z","target":"bunyan_formatter","line":15,"file":"src/main.rs","first_parameter":2,"sub_parameter":1}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - EVENT] Events have the full context of their parent span!","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:46.670647Z","target":"bunyan_formatter","line":17,"file":"src/main.rs","first_parameter":2,"sub_parameter":1}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - EVENT] going to sleep","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:46.671039Z","target":"bunyan_formatter","line":18,"file":"src/main.rs","first_parameter":2,"sub_parameter":1}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - EVENT] function waked up","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:47.671758Z","target":"bunyan_formatter","line":20,"file":"src/main.rs","first_parameter":2,"sub_parameter":1}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - END]","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:47.672046Z","target":"bunyan_formatter","line":15,"file":"src/main.rs","elapsed_milliseconds":1001,"first_parameter":2,"sub_parameter":1}
{"v":0,"name":"tracing_demo","msg":"[A_UNIT_OF_WORK - EVENT] Tracing is quite cool!","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:47.672307Z","target":"bunyan_formatter","line":12,"file":"src/main.rs","excited":"true","first_parameter":2}
{"v":0,"name":"tracing_demo","msg":"[A_UNIT_OF_WORK - END]","level":30,"hostname":"192.168.1.2","pid":90700,"time":"2022-12-28T05:47:47.672496Z","target":"bunyan_formatter","line":7,"file":"src/main.rs","first_parameter":2,"elapsed_milliseconds":2005}
```