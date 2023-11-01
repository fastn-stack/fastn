fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(outer_main())
}

async fn outer_main() {
    let mut i = P { x: 10 };

    yo(&mut i).await;

    let mut q = Q { p: &mut i };
    bo(&mut q).await;

    println!("Hello, world: {}", q.p.x);
}

struct P {
    x: i32,
}

struct Q<'a> {
    p: &'a mut P,
}

async fn yo(i: &mut P) {
    i.x = 20;
    println!("yo: {}", i.x);
}

async fn bo<'m, 'o: 'm>(i: &'m mut Q<'o>) {
    i.p.x = 30;
    println!("bo: {}", i.p.x);
}
