use sycamore::prelude::*;

#[component]
fn App<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    let items = ctx.create_signal(vec![
        view! { li { "Hello!" } },
        view! { li { "I am an item in a fragment" } },
    ]);

    let add_item = |_| {
        items.set(
            items
                .get()
                .as_ref()
                .clone()
                .into_iter()
                .chain(Some(view! { li { "New item" } }))
                .collect(),
        );
    };

    view! {
        div {
            button(on:click=add_item) { "Add item" }
            ul(class="items") {
                (View::new_fragment((*items.get()).clone()))
            }
        }
    }
}

fn main() {
    sycamore::render(|ctx| view! { App() });
}
