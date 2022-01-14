use sycamore::prelude::*;

#[component]
fn App<G: Html>(_: ScopeRef, _: ()) -> View<G> {
    view! {
        p {
            "Hello World!"
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
