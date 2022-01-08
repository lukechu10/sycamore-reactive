use sycamore_reactive::*;

fn main() {
    create_scope_immediate(|ctx| {
        let mut outer = None;
        let disposer = ctx.create_child_scope(|ctx| {
            let signal = ctx.create_signal(0);
            outer = Some(signal);
            //           ^^^^^^
        });
        disposer();
        dbg!(outer.unwrap().get());
    });
}
