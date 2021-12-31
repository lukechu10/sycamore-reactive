use sycamore_reactive::*;

fn main() {
    let ctx = Ctx::default();
    let data = ctx.create_signal(123);
    ctx.create_effect(|| {
        println!("Hello World!");
        dbg!(data.get());
    });

    ctx.on_cleanup(|| {
        println!("Outer scope cleanup");
    });

    ctx.create_child_scope(|ctx| {
        let inner = ctx.create_signal("abc");
        ctx.on_cleanup(|| {
            println!("Start inner scope cleanup");
            dbg!(inner.get());
            dbg!(data.get());
            println!("Finish inner scope cleanup");
        })
    });
    drop(ctx);
    dbg!(data.get());
}
