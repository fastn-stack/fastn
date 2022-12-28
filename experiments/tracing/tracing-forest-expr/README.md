# tracing_forest


Docs: https://docs.rs/tracing-forest/


## Function `init()`

Initializes a global subscriber with a ForestLayer using the default configuration.

## Code Example

```rust

#[tracing::instrument]
async fn recursive(param: i32) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()>>> {
    Box::pin(async move {
        if param < 0 {
            return;
        }
        tracing::Span::current().in_scope(|| async {
            recursive(param - 1).await;
        });
    })
}

#[tracing::instrument]
async fn some_expensive_operation(id: u32) {
    debug!("starting from `some_expensive_operation`");
    std::thread::sleep(std::time::Duration::from_secs(1));
    recursive(5).await;
    error!("exiting from `some_expensive_operation`");
}

#[tracing::instrument(fields(id))]
async fn conn(id: u32) {
    for i in 0..3 {
        some_expensive_operation(id).await;
        info!(id, "step {}", i);
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Registry::default().with(ForestLayer::default()).init();
    conn(5).await;
}


```

## Console Output

```text
INFO     conn [ 3.01s | 0.01% / 100.00% ]
INFO     ┝━ some_expensive_operation [ 1.00s | 33.34% / 33.34% ]
DEBUG    │  ┝━ 🐛 [debug]: starting from `some_expensive_operation`
INFO     │  ┝━ recursive [ 14.7µs | 0.00% ]
ERROR    │  ┕━ 🚨 [error]: exiting from `some_expensive_operation`
INFO     ┝━ ｉ [info]: step 0 | id: 5
INFO     ┝━ some_expensive_operation [ 1.00s | 33.35% / 33.35% ]
DEBUG    │  ┝━ 🐛 [debug]: starting from `some_expensive_operation`
INFO     │  ┝━ recursive [ 55.5µs | 0.00% ]
ERROR    │  ┕━ 🚨 [error]: exiting from `some_expensive_operation`
INFO     ┝━ ｉ [info]: step 1 | id: 5
INFO     ┝━ some_expensive_operation [ 1.00s | 33.30% / 33.30% ]
DEBUG    │  ┝━ 🐛 [debug]: starting from `some_expensive_operation`
INFO     │  ┝━ recursive [ 40.7µs | 0.00% ]
ERROR    │  ┕━ 🚨 [error]: exiting from `some_expensive_operation`
INFO     ┕━ ｉ [info]: step 2 | id: 5

```

# Current Span not recording the fields

In the below function current span "recursive-span" is not recording the param

```rust
fn recursive(param: i32) {
    std::thread::sleep(std::time::Duration::from_secs(1));
    if param < 0 {
        return;
    }
    // This span is not recording the param is not getting logged
    tracing::info_span!("recursive-span").in_scope(|| {
        tracing::Span::current_span().record("param", param);
        recursive(param - 1);
    });
}

```

# Recursion Without async

## Code Example

```rust

#[tracing::instrument]
fn recursive(param: i32) {
    info!(msg = "inside recursive", param = param);
    std::thread::sleep(std::time::Duration::from_secs(1));
    if param < 0 {
        return;
    }
    recursive(param - 1);
}

#[tracing::instrument]
async fn some_expensive_operation(id: u32) {
    tracing::Span::current().record("id", id);
    debug!("starting from `some_expensive_operation`");
    recursive(5);
    error!("exiting from `some_expensive_operation`");
}

#[tracing::instrument(fields(id))]
async fn conn(id: u32) {
    for i in 0..2 {
        info!(id, "running step {}", i);
        some_expensive_operation(id).await;
        info!(id, "ending step {}", i);
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Registry::default().with(ForestLayer::default()).init();
    conn(5).await;
}

```

## Console Output

```text

INFO     conn [ 14.1s | 0.00% / 100.00% ]
INFO     ┝━ ｉ [info]: running step 0 | id: 5
INFO     ┝━ some_expensive_operation [ 7.04s | 0.00% / 50.04% ]
DEBUG    │  ┝━ 🐛 [debug]: starting from `some_expensive_operation`
INFO     │  ┝━ recursive [ 7.04s | 7.15% / 50.04% ]
INFO     │  │  ┝━ ｉ [info]:  | msg: "inside recursive" | param: 5
INFO     │  │  ┕━ recursive [ 6.03s | 7.15% / 42.89% ]
INFO     │  │     ┝━ ｉ [info]:  | msg: "inside recursive" | param: 4
INFO     │  │     ┕━ recursive [ 5.02s | 7.15% / 35.74% ]
INFO     │  │        ┝━ ｉ [info]:  | msg: "inside recursive" | param: 3
INFO     │  │        ┕━ recursive [ 4.02s | 7.15% / 28.59% ]
INFO     │  │           ┝━ ｉ [info]:  | msg: "inside recursive" | param: 2
INFO     │  │           ┕━ recursive [ 3.01s | 7.15% / 21.44% ]
INFO     │  │              ┝━ ｉ [info]:  | msg: "inside recursive" | param: 1
INFO     │  │              ┕━ recursive [ 2.01s | 7.14% / 14.29% ]
INFO     │  │                 ┝━ ｉ [info]:  | msg: "inside recursive" | param: 0
INFO     │  │                 ┕━ recursive [ 1.01s | 7.15% ]
INFO     │  │                    ┕━ ｉ [info]:  | msg: "inside recursive" | param: -1
ERROR    │  ┕━ 🚨 [error]: exiting from `some_expensive_operation`
INFO     ┝━ ｉ [info]: ending step 0 | id: 5
INFO     ┝━ ｉ [info]: running step 1 | id: 5
INFO     ┝━ some_expensive_operation [ 7.02s | 0.00% / 49.96% ]
DEBUG    │  ┝━ 🐛 [debug]: starting from `some_expensive_operation`
INFO     │  ┝━ recursive [ 7.02s | 7.15% / 49.96% ]
INFO     │  │  ┝━ ｉ [info]:  | msg: "inside recursive" | param: 5
INFO     │  │  ┕━ recursive [ 6.02s | 7.12% / 42.81% ]
INFO     │  │     ┝━ ｉ [info]:  | msg: "inside recursive" | param: 4
INFO     │  │     ┕━ recursive [ 5.02s | 7.12% / 35.68% ]
INFO     │  │        ┝━ ｉ [info]:  | msg: "inside recursive" | param: 3
INFO     │  │        ┕━ recursive [ 4.02s | 7.13% / 28.56% ]
INFO     │  │           ┝━ ｉ [info]:  | msg: "inside recursive" | param: 2
INFO     │  │           ┕━ recursive [ 3.01s | 7.14% / 21.44% ]
INFO     │  │              ┝━ ｉ [info]:  | msg: "inside recursive" | param: 1
INFO     │  │              ┕━ recursive [ 2.01s | 7.15% / 14.30% ]
INFO     │  │                 ┝━ ｉ [info]:  | msg: "inside recursive" | param: 0
INFO     │  │                 ┕━ recursive [ 1.01s | 7.15% ]
INFO     │  │                    ┕━ ｉ [info]:  | msg: "inside recursive" | param: -1
ERROR    │  ┕━ 🚨 [error]: exiting from `some_expensive_operation`
INFO     ┕━ ｉ [info]: ending step 1 | id: 5

```

# With HierarchicalLayer

We can instead use tracing-forest as a drop-in replacement for tracing-tree.

## Code

```rust
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    use tracing_tree::HierarchicalLayer;
    Registry::default()
        .with(HierarchicalLayer::default())
        .init();
    conn(5).await;
}

```

## Console Output

```shell
conn 
  0ms  INFO running step 0, id=5
  some_expensive_operation id=5
    0ms DEBUG starting from `some_expensive_operation`
    recursive param=5
    
    1004ms ERROR exiting from `some_expensive_operation`
  
  1004ms  INFO ending step 0, id=5
  1004ms  INFO running step 1, id=5
  some_expensive_operation id=5
    0ms DEBUG starting from `some_expensive_operation`
    recursive param=5
    
    1003ms ERROR exiting from `some_expensive_operation`
  
  2008ms  INFO ending step 1, id=5
  2008ms  INFO running step 2, id=5
  some_expensive_operation id=5
    0ms DEBUG starting from `some_expensive_operation`
    recursive param=5
    
    1001ms ERROR exiting from `some_expensive_operation`
  
  3009ms  INFO ending step 2, id=5

```

# `tracing_forest::runtime::worker_task`

## Nonblocking log processing with worker_task

> tracing-forest collects trace data into trees, and can sometimes produce large 
> trees that need to be processed. To avoid blocking the main task in these cases,
> a common strategy is to send this data to a worker task for formatting and writing.

> The worker_task function provides this behavior as a first-class feature of this
> crate, and handles the configuration, initialization, and graceful shutdown of a
> subscriber with an associated worker task for formatting and writing.

## Code Example: With async recursion

async recursion still not working

```rust

use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, Registry};

#[tracing::instrument]
async fn recursive(param: i32) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()>>> {
    info!(msg = "inside recursive", param = param);
    std::thread::sleep(std::time::Duration::from_secs(1));
    Box::pin(async move {
        if param < 0 {
            return;
        }
        recursive(param - 1).await;
    })
}

#[tracing::instrument]
async fn some_expensive_operation(id: u32) {
    tracing::Span::current().record("id", id);
    debug!("starting from `some_expensive_operation`");
    recursive(5).await;
    error!("exiting from `some_expensive_operation`");
}

#[tracing::instrument(fields(id))]
async fn conn(id: u32) {
    for i in 0..2 {
        info!(id, "running step {}", i);
        some_expensive_operation(id).await;
        info!(id, "ending step {}", i);
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_forest::worker_task()
        .set_global(true)
        .build_with(|_layer: tracing_forest::ForestLayer<_, _>| {
            Registry::default()
                .with(tracing_forest::ForestLayer::default())
                .with(tracing_forest::util::LevelFilter::INFO)
        })
        .on(async {}) // this statement is needed, without this logs are getting printed
        .await;
    conn(5).await;
}

```

## Console Output

```text
8777dd74-d547-48c1-8228-39bbb170593d 2022-12-28T07:51:32.064880+00:00     INFO     conn [ 2.01s | 0.02% / 100.00% ]
8777dd74-d547-48c1-8228-39bbb170593d 2022-12-28T07:51:32.064990+00:00     INFO     ┝━ ｉ [info]: running step 0 | id: 5
8777dd74-d547-48c1-8228-39bbb170593d 2022-12-28T07:51:32.065027+00:00     INFO     ┝━ some_expensive_operation [ 1.01s | 0.01% / 50.03% ]
8777dd74-d547-48c1-8228-39bbb170593d 2022-12-28T07:51:32.065061+00:00     INFO     │  ┝━ recursive [ 1.01s | 50.02% ]
8777dd74-d547-48c1-8228-39bbb170593d 2022-12-28T07:51:32.065078+00:00     INFO     │  │  ┕━ ｉ [info]:  | msg: "inside recursive" | param: 5
8777dd74-d547-48c1-8228-39bbb170593d 2022-12-28T07:51:33.070344+00:00     ERROR    │  ┕━ 🚨 [error]: exiting from `some_expensive_operation`
8777dd74-d547-48c1-8228-39bbb170593d 2022-12-28T07:51:33.070442+00:00     INFO     ┝━ ｉ [info]: ending step 0 | id: 5
8777dd74-d547-48c1-8228-39bbb170593d 2022-12-28T07:51:33.070463+00:00     INFO     ┝━ ｉ [info]: running step 1 | id: 5
8777dd74-d547-48c1-8228-39bbb170593d 2022-12-28T07:51:33.070529+00:00     INFO     ┝━ some_expensive_operation [ 1.00s | 0.01% / 49.95% ]
8777dd74-d547-48c1-8228-39bbb170593d 2022-12-28T07:51:33.070611+00:00     INFO     │  ┝━ recursive [ 1.00s | 49.94% ]
8777dd74-d547-48c1-8228-39bbb170593d 2022-12-28T07:51:33.070647+00:00     INFO     │  │  ┕━ ｉ [info]:  | msg: "inside recursive" | param: 5
8777dd74-d547-48c1-8228-39bbb170593d 2022-12-28T07:51:34.074174+00:00     ERROR    │  ┕━ 🚨 [error]: exiting from `some_expensive_operation`
8777dd74-d547-48c1-8228-39bbb170593d 2022-12-28T07:51:34.074307+00:00     INFO     ┕━ ｉ [info]: ending step 1 | id: 5
```

## Code Example: With recursion

```rust
#[tracing::instrument]
fn recursive(param: i32) {
    info!(msg = "inside recursive", param = param);
    std::thread::sleep(std::time::Duration::from_secs(1));
    if param < 0 {
        return;
    }
    recursive(param - 1)
}

#[tracing::instrument]
async fn some_expensive_operation(id: u32) {
    tracing::Span::current().record("id", id);
    debug!("starting from `some_expensive_operation`");
    recursive(5);
    error!("exiting from `some_expensive_operation`");
}
```

## Console Output

```text
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:22.102118+00:00     INFO     conn [ 14.1s | 0.00% / 100.00% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:22.102242+00:00     INFO     ┝━ ｉ [info]: running step 0 | id: 5
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:22.102271+00:00     INFO     ┝━ some_expensive_operation [ 7.03s | 0.00% / 50.02% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:22.102306+00:00     INFO     │  ┝━ recursive [ 7.03s | 7.12% / 50.02% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:22.102323+00:00     INFO     │  │  ┝━ ｉ [info]:  | msg: "inside recursive" | param: 5
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:23.103016+00:00     INFO     │  │  ┕━ recursive [ 6.03s | 7.15% / 42.90% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:23.103076+00:00     INFO     │  │     ┝━ ｉ [info]:  | msg: "inside recursive" | param: 4
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:24.108129+00:00     INFO     │  │     ┕━ recursive [ 5.02s | 7.15% / 35.75% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:24.108149+00:00     INFO     │  │        ┝━ ｉ [info]:  | msg: "inside recursive" | param: 3
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:25.113266+00:00     INFO     │  │        ┕━ recursive [ 4.02s | 7.15% / 28.59% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:25.113370+00:00     INFO     │  │           ┝━ ｉ [info]:  | msg: "inside recursive" | param: 2
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:26.118541+00:00     INFO     │  │           ┕━ recursive [ 3.01s | 7.15% / 21.44% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:26.118631+00:00     INFO     │  │              ┝━ ｉ [info]:  | msg: "inside recursive" | param: 1
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:27.123792+00:00     INFO     │  │              ┕━ recursive [ 2.01s | 7.13% / 14.29% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:27.123877+00:00     INFO     │  │                 ┝━ ｉ [info]:  | msg: "inside recursive" | param: 0
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:28.126128+00:00     INFO     │  │                 ┕━ recursive [ 1.01s | 7.15% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:28.126198+00:00     INFO     │  │                    ┕━ ｉ [info]:  | msg: "inside recursive" | param: -1
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:29.131702+00:00     ERROR    │  ┕━ 🚨 [error]: exiting from `some_expensive_operation`
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:29.131821+00:00     INFO     ┝━ ｉ [info]: ending step 0 | id: 5
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:29.131843+00:00     INFO     ┝━ ｉ [info]: running step 1 | id: 5
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:29.131896+00:00     INFO     ┝━ some_expensive_operation [ 7.02s | 0.00% / 49.98% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:29.131968+00:00     INFO     │  ┝━ recursive [ 7.02s | 7.13% / 49.97% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:29.132004+00:00     INFO     │  │  ┝━ ｉ [info]:  | msg: "inside recursive" | param: 5
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:30.133913+00:00     INFO     │  │  ┕━ recursive [ 6.02s | 7.15% / 42.84% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:30.133990+00:00     INFO     │  │     ┝━ ｉ [info]:  | msg: "inside recursive" | param: 4
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:31.139176+00:00     INFO     │  │     ┕━ recursive [ 5.02s | 7.12% / 35.69% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:31.139270+00:00     INFO     │  │        ┝━ ｉ [info]:  | msg: "inside recursive" | param: 3
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:32.140122+00:00     INFO     │  │        ┕━ recursive [ 4.01s | 7.13% / 28.57% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:32.140145+00:00     INFO     │  │           ┝━ ｉ [info]:  | msg: "inside recursive" | param: 2
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:33.141991+00:00     INFO     │  │           ┕━ recursive [ 3.01s | 7.13% / 21.44% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:33.142008+00:00     INFO     │  │              ┝━ ｉ [info]:  | msg: "inside recursive" | param: 1
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:34.144080+00:00     INFO     │  │              ┕━ recursive [ 2.01s | 7.16% / 14.31% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:34.144346+00:00     INFO     │  │                 ┝━ ｉ [info]:  | msg: "inside recursive" | param: 0
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:35.149558+00:00     INFO     │  │                 ┕━ recursive [ 1.01s | 7.15% ]
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:35.149651+00:00     INFO     │  │                    ┕━ ｉ [info]:  | msg: "inside recursive" | param: -1
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:36.154913+00:00     ERROR    │  ┕━ 🚨 [error]: exiting from `some_expensive_operation`
7880e5c5-2118-4772-adba-b823726ba944 2022-12-28T07:56:36.154996+00:00     INFO     ┕━ ｉ [info]: ending step 1 | id: 5

```

## Spans Time Format

More: https://docs.rs/tracing-forest/0.1.5/tracing_forest/printer/struct.Pretty.html

```text
/// Format logs for pretty printing.
///
/// # Interpreting span times
///
/// Spans have the following format:
/// ```txt
/// <NAME> [ <DURATION> | <BODY> / <ROOT> ]
/// ```
/// * `DURATION` represents the total time the span was entered for. If the span
/// was used to instrument a `Future` that sleeps, then that time won't be counted
/// since the `Future` won't be polled during that time, and so the span won't enter.
/// * `BODY` represents the percent time the span is entered relative to the root
/// span, *excluding* time that any child spans are entered.
/// * `ROOT` represents the percent time the span is entered relative to the root
/// span, *including* time that any child spans are entered.
///
/// As a mental model, look at `ROOT` to quickly narrow down which branches are
/// costly, and look at `BASE` to pinpoint exactly which spans are expensive.
///
/// Spans without any child spans would have the same `BASE` and `ROOT`, so the
/// redundency is omitted.
```

# Formatted Output

- We can omit fields from log output based on features to use

`Cargo.toml`

```toml
tracing-forest = { version = "0.1.5", features = [
    "smallvec",
    "tokio",
    "serde",
    "ansi", # This is for printing the ColorLevel in the events
    # "uuid",  # This is for printing uuid for every request
    # "chrono", # This is for printing the time of the event
    # "env-filter",
]}

```

## Console Output

```text
INFO     conn [ 14.0s | 0.00% / 100.00% ]
INFO     ┝━ ｉ [info]: running step 0 | id: 5
INFO     ┝━ some_expensive_operation [ 7.03s | 0.00% / 50.01% ]
INFO     │  ┝━ recursive [ 7.03s | 7.14% / 50.01% ]
INFO     │  │  ┝━ ｉ [info]:  | msg: "inside recursive" | param: 5
INFO     │  │  ┕━ recursive [ 6.02s | 7.16% / 42.87% ]
INFO     │  │     ┝━ ｉ [info]:  | msg: "inside recursive" | param: 4
INFO     │  │     ┕━ recursive [ 5.02s | 7.13% / 35.71% ]
INFO     │  │        ┝━ ｉ [info]:  | msg: "inside recursive" | param: 3
INFO     │  │        ┕━ recursive [ 4.02s | 7.14% / 28.58% ]
INFO     │  │           ┝━ ｉ [info]:  | msg: "inside recursive" | param: 2
INFO     │  │           ┕━ recursive [ 3.01s | 7.13% / 21.44% ]
INFO     │  │              ┝━ ｉ [info]:  | msg: "inside recursive" | param: 1
INFO     │  │              ┕━ recursive [ 2.01s | 7.16% / 14.31% ]
INFO     │  │                 ┝━ ｉ [info]:  | msg: "inside recursive" | param: 0
INFO     │  │                 ┕━ recursive [ 1.01s | 7.16% ]
INFO     │  │                    ┕━ ｉ [info]:  | msg: "inside recursive" | param: -1
ERROR    │  ┕━ 🚨 [error]: exiting from `some_expensive_operation`
...
```

