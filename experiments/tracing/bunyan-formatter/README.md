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

## Console Output

```shell
cargo run
```

```shell
{"v":0,"name":"tracing_demo","msg":"Orphan event without a parent span","level":30,"hostname":"192.168.1.2","pid":89415,"time":"2022-12-28T05:37:32.913883Z","target":"bunyan_formatter","line":27,"file":"src/main.rs"}
{"v":0,"name":"tracing_demo","msg":"[A_UNIT_OF_WORK - START]","level":30,"hostname":"192.168.1.2","pid":89415,"time":"2022-12-28T05:37:32.914113Z","target":"bunyan_formatter","line":7,"file":"src/main.rs","first_parameter":2}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - START]","level":30,"hostname":"192.168.1.2","pid":89415,"time":"2022-12-28T05:37:32.914216Z","target":"bunyan_formatter","line":15,"file":"src/main.rs","sub_parameter":0,"first_parameter":2}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - EVENT] Events have the full context of their parent span!","level":30,"hostname":"192.168.1.2","pid":89415,"time":"2022-12-28T05:37:32.914315Z","target":"bunyan_formatter","line":17,"file":"src/main.rs","sub_parameter":0,"first_parameter":2}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - END]","level":30,"hostname":"192.168.1.2","pid":89415,"time":"2022-12-28T05:37:32.914425Z","target":"bunyan_formatter","line":15,"file":"src/main.rs","elapsed_milliseconds":0,"sub_parameter":0,"first_parameter":2}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - START]","level":30,"hostname":"192.168.1.2","pid":89415,"time":"2022-12-28T05:37:32.914529Z","target":"bunyan_formatter","line":15,"file":"src/main.rs","sub_parameter":1,"first_parameter":2}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - EVENT] Events have the full context of their parent span!","level":30,"hostname":"192.168.1.2","pid":89415,"time":"2022-12-28T05:37:32.914622Z","target":"bunyan_formatter","line":17,"file":"src/main.rs","sub_parameter":1,"first_parameter":2}
{"v":0,"name":"tracing_demo","msg":"[A_SUB_UNIT_OF_WORK - END]","level":30,"hostname":"192.168.1.2","pid":89415,"time":"2022-12-28T05:37:32.914709Z","target":"bunyan_formatter","line":15,"file":"src/main.rs","elapsed_milliseconds":0,"sub_parameter":1,"first_parameter":2}
{"v":0,"name":"tracing_demo","msg":"[A_UNIT_OF_WORK - EVENT] Tracing is quite cool!","level":30,"hostname":"192.168.1.2","pid":89415,"time":"2022-12-28T05:37:32.914811Z","target":"bunyan_formatter","line":12,"file":"src/main.rs","excited":"true","first_parameter":2}
{"v":0,"name":"tracing_demo","msg":"[A_UNIT_OF_WORK - END]","level":30,"hostname":"192.168.1.2","pid":89415,"time":"2022-12-28T05:37:32.9149Z","target":"bunyan_formatter","line":7,"file":"src/main.rs","elapsed_milliseconds":0,"first_parameter":2}
```