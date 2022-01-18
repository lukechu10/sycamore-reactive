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

    // let items = ctx.map_indexed(items, |ctx, &Cat { id, name }| {
    //     view! {
    //         li {
    //             a(href=format!("https://www.youtube.com/watch?v={id}")) {
    //                 (name)
    //             }
    //         }
    //     }
    // });

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
