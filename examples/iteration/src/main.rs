use sycamore::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cat {
    id: &'static str,
    name: &'static str,
}

#[component]
fn App<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    let items = ctx.create_signal(vec![
        Cat {
            id: "J---aiyznGQ",
            name: "Keyboard Cat",
        },
        Cat {
            id: "z_AbfPXTKms",
            name: "Maru",
        },
        Cat {
            id: "OUtn3pvWmpg",
            name: "Henri The Existential Cat",
        },
    ]);

    view! { ctx,
        p { "The famous cats of YouTube" }
        ul {
            Indexed(IndexedProps::builder().iterable(items).template(|ctx, &Cat { id, name }| view! { ctx,
                li {
                    a(href=format!("https://www.youtube.com/watch?v={id}")) {
                        (name)
                    }
                }
            }).build())
        }
    }
}

fn main() {
    sycamore::render(|ctx| view! { ctx, App() });
}
