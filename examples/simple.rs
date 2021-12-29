use sycamore_reactive::*;

fn main() {
    create_scope(|ctx| {
        let data = ctx.create_signal(123);
        dbg!(data.get());
        data.set(456);
        dbg!(data.get());

        create_scope(|ctx| {
            let inner = ctx.create_signal("abc");
            dbg!(inner.get());
            dbg!(data.get());
        });
    });
}
