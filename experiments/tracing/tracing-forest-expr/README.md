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

```shell
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