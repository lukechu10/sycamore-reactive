use sycamore::prelude::*;

#[component]
fn MyComponent<G: Html>(ctx: ScopeRef, props: RcSignal<i32>) -> View<G> {
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
fn App<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    let state = create_rc_signal(0);

    let increment = {
        let state = state.clone();
        move |_| state.set(*state.get() + 1)
    };

    view! {
        div {
            "Component demo"

            MyComponent(state.clone())
            MyComponent(state)

            button(on:click=increment) { "+" }
        }
    }
}

fn main() {
    sycamore::render(|ctx| view! { App() });
}
