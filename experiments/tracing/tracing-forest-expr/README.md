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
tracing-forest collects trace data into trees, and can sometimes produce large 
trees that need to be processed. To avoid blocking the main task in these cases,
a common strategy is to send this data to a worker task for formatting and writing.

The worker_task function provides this behavior as a first-class feature of this
crate, and handles the configuration, initialization, and graceful shutdown of a
subscriber with an associated worker task for formatting and writing.

