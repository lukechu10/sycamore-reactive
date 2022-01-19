use sycamore::prelude::*;

#[component]
fn MyComponent<'a, G: Html>(ctx: ScopeRef<'a>, props: &'a Signal<i32>) -> View<G> {
    view! {
        div(class="my-component") {
            "My component"
            p {
                "Value: "
                (props.get())
            }
        }
    }
}

#[component]
fn App<'a, G: Html>(ctx: ScopeRef<'a>, _: ()) -> View<G> {
    let state = ctx.create_signal(0);

    let increment = |_| state.set(*state.get() + 1);

    view! {
        div {
            "Component demo"

            MyComponent(state)
            MyComponent(state)

            button(on:click=increment) { "+" }
        }
    }
}

fn main() {
    sycamore::render(|ctx| view! { App() });
}
