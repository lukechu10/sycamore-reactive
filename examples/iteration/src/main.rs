use sycamore::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cat {
    id: &'static str,
    name: &'static str,
}

#[component]
fn App<'a, G: Html>(ctx: ScopeRef<'a>, _: ()) -> View<G> {
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

    view! {
        p { "The famous cats of YouTube" }
        ul {
            Indexed(IndexedProps {
                iterable: items,
                template: |ctx, &Cat { id, name } | view! {
                    li {
                        a(href=format!("https://www.youtube.com/watch?v={id}")) {
                            (name)
                        }
                    }
                }
            })
        }
    }
}

fn main() {
    sycamore::render(|ctx| view! { App() });
}
