use sycamore::prelude::*;

fn App<G: Html>(ctx: ScopeRef, props: ()) -> View<G> {
    view! {
        p {
            "Hello World!"
        }
    }
}

fn main() {
    sycamore::render(|ctx| {
        App(ctx, ())
    });
}
