use sycamore_reactive::*;

fn main() {
    let mut outer = None;
    let scope = create_scope(|ctx| {
        outer = Some(ctx);
        let data = ctx.create_signal(123);
        ctx.create_effect(|| {
            println!("Hello World!");
            ctx.create_effect(|| {
                dbg!(data.get());
            });
            dbg!(data.get());
        });

        ctx.on_cleanup(|| {
            println!("Outer scope cleanup");
        });

        create_scope(|ctx| {
            let inner = ctx.create_signal("abc");
            ctx.on_cleanup(|| {
                println!("Start inner scope cleanup");
                dbg!(inner.get());
                dbg!(data.get());
                println!("Finish inner scope cleanup");
            })
        });
    });
    drop(scope);
    // let c = outer.unwrap();
    // c.create_effect(|| {});
}
