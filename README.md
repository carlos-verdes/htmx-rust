# htmx-rust

Server-rendered Rust web apps with [htmx](https://htmx.org) and [axum](https://github.com/tokio-rs/axum), built on how the web already works: every URL names a resource, the server answers with HTML, and links and forms are the only way to move.

**Status: early. Views nest to any depth and are served as whole documents, and a view's fields are parsed from its URL and printed back as one; the rest of the design is written down, not built.**

```rust
use axum::Router;
use htmx_rust::{Html, NoParent, Outlet, View, Views};
use maud::{DOCTYPE, html};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct App;

/// The root view: the whole document, around its outlet. It has no parent
/// and no path.
#[derive(Serialize, Deserialize)]
struct Document;

impl View for Document {
    type State = App;
    type Parent = NoParent;

    async fn render(&self, _: &App, outlet: Outlet) -> impl Into<Html> {
        html! {
            (DOCTYPE)
            title { (outlet.title) }
            main id=(outlet.id) { (outlet.content) }
        }
    }
}

/// What `/catalogs` shows, in the root view's outlet.
#[derive(Serialize, Deserialize)]
struct Catalogs;

impl View for Catalogs {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/catalogs";
    const TITLE: &'static str = "Catalogs";

    async fn render(&self, _: &App, _: Outlet) -> impl Into<Html> {
        let teas = Catalog { id: 7, q: Some("tea".into()) };
        html! {
            h1 { "Catalogs" }
            // A link is printed from a value, so it cannot point at a route
            // that does not exist: this one is `/catalogs/7?q=tea`.
            a href=(teas.url()) { "The teas of catalog 7" }
        }
    }
}

/// What `/catalogs/7?q=tea` shows. A view's fields are what its URL carries.
#[derive(Serialize, Deserialize)]
struct Catalog {
    /// The `{id}` of the path.
    id: u32,
    /// Any other field is in the query string.
    q: Option<String>,
}

impl View for Catalog {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/catalogs/{id}";
    const TITLE: &'static str = "Catalog";

    async fn render(&self, _: &App, outlet: Outlet) -> impl Into<Html> {
        html! {
            h1 { "Catalog " (self.id) }
            a href=(Items { id: self.id }.url()) { "Its items" }
            section id=(outlet.id) { (outlet.content) }
        }
    }
}

/// What `/catalogs/7/items` shows, in the outlet of the catalog's view: a view
/// names the view it is nested in, and its path follows that one's.
#[derive(Serialize, Deserialize)]
struct Items {
    /// The `{id}` of its parent's part of the path.
    id: u32,
}

impl View for Items {
    type State = App;
    type Parent = Catalog;
    const PATH: &'static str = "/items";
    const TITLE: &'static str = "Items";

    async fn render(&self, _: &App, _: Outlet) -> impl Into<Html> {
        html! { h2 { "The items of catalog " (self.id) } }
    }
}

let router: Router = Router::new()
    .view::<Catalogs>()
    .view::<Catalog>()
    .view::<Items>()
    .with_state(App);

assert_eq!(Catalog { id: 7, q: None }.url(), "/catalogs/7");
assert_eq!(Items { id: 7 }.url(), "/catalogs/7/items");
```

A view derives serde's `Serialize` and `Deserialize`: that is how its fields are read from a URL and printed back. A URL that does not parse into the fields, such as `/catalogs/seven`, is answered with `400 Bad Request` and no view runs.

Nesting is declared, never read from the path: `/catalogs/7` extends `/catalogs`, and is still shown beside it in the root view's outlet, because `Catalog` names `Document` as its parent. A nested view holds a field for every parameter of its whole path, its ancestors' included (the `id` of `Items`); one that does not is an error when it is registered. An outlet's id follows its view's depth (`outlet-0`, `outlet-1`, …) unless the view names one: `const OUTLET: Option<&'static str> = Some("catalog");`.

## REST, as it was meant

REST describes the web of pages, links and forms. htmx-rust holds an app to its constraints, so that refresh, back and forward, sharing a link and caching work without anyone building them.

| REST constraint | In htmx-rust |
|---|---|
| **Stateless**: a request carries all that is needed to understand it | The URL is the state. Nothing about what the browser shows lives in a session or in JavaScript memory. |
| **Identification of resources** | Every view has its own URL, and a URL always shows the same view, on a click and on a refresh. |
| **Manipulation through representations** | The server sends HTML: the resource itself, ready to show, not data for the browser to assemble. |
| **Self-descriptive messages** | Real status codes. A missing resource is a 404 drawn where the resource would have been. |
| **Hypermedia as the engine of application state** | Plain `<a href>` links and plain forms. Links are built from typed views, so they cannot point at a route that does not exist. |
| **Cache** | One URL can answer with the whole document or with one part of it; every response says so (`Vary`), so caches stay correct. |
| **Code on demand** (optional) | htmx is an enhancement. Every page and action works without JavaScript. |

## Principles

- **URL as state.** A link opens a view, or a view nested inside it, and the address bar always says what is on screen.
- **Convention over configuration.** Sensible defaults first; every one of them can be overridden.
- **The server leads.** The server decides what each response contains; the browser shows it.
- **Unopinionated core.** The core writes no HTML and chooses no styling. [maud](https://maud.lambda.xyz) is supported out of the box and can be switched off.
- **Ready for agents.** Few concepts, one way to do each thing, and mistakes caught by the compiler.

## Demo

[`demo`](demo) is a small app to read and run. htmx 4 comes from the `htmx.org` package, which [pnpm](https://pnpm.io) installs:

```sh
(cd demo && pnpm install)
cargo run --package htmx-rust-demo
```

[`browser-tests`](browser-tests) is a [Playwright](https://playwright.dev) suite that starts the demo and checks it in a real browser, with JavaScript on and off:

```sh
cd browser-tests
pnpm install
pnpm exec playwright install chromium
pnpm test
```

## Documentation

- [Glossary](GLOSSARY.md): the words this project uses.
- [Decisions](docs/adr): why it is built the way it is.

## Planned

- The server sending only the outlet a request names
- Forms
- Regions: parts of a page refreshed on their own
- File-system routing
- A starter with a layout and base components

## License

Licensed under the [MIT License](LICENSE).
