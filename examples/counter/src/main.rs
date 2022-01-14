use sycamore::prelude::*;

#[component]
fn App<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    let state = ctx.create_signal(0i32);
    let increment = |_| state.set(*state.get() + 1);
    let decrement = |_| state.set(*state.get() - 1);
    view! {
        div {
            p { "Value: " (state.get()) }
            button(on:click=increment) { "+" }
            button(on:click=decrement) { "-" }
        }
    }
}

fn main() {
    sycamore::render(|ctx| {
        view! {
            App()
        }
    });
}
