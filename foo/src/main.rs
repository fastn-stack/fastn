fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(outer_main())
}

async fn outer_main() {
    let mut i = P { x: 10, y: 20 };

    yo(&mut i).await;
    println!("Hello, world: {}", i.x);
}

struct P {
    x: i32,
    y: i32,
}

async fn yo(i: &mut P) {
    i.x = 20;
    println!("yo: {}", i.x);
}
