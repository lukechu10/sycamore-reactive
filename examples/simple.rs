use sycamore_reactive::*;

fn main() {
    let disposer = create_scope(|ctx| {
        let data = ctx.create_signal(0);
        ctx.create_effect(|| println!("data value changed. new value = {}", data.get()));
        data.set(1);
        data.set(2);
        data.set(3);
        data.set(4);
    });
    disposer();
}
