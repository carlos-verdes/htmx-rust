# htmx-rust

Server-rendered Rust web apps with [htmx](https://htmx.org) and [axum](https://github.com/tokio-rs/axum), built on how the web already works: every URL names a resource, the server answers with HTML, and links and forms are the only way to move.

**Status: early. A page in a layout is served; the rest of the design is written down, not built.**

```rust
use axum::Router;
use htmx_rust::{Html, Layout, Outlet, View, Views};
use maud::{DOCTYPE, html};

#[derive(Clone)]
struct App;

/// The layout: the whole document, around its outlet.
struct Document;

impl Layout<App> for Document {
    async fn render(_: &App, outlet: Outlet, title: &str) -> impl Into<Html> {
        html! {
            (DOCTYPE)
            title { (title) }
            main id=(outlet.id) { (outlet.content) }
        }
    }
}

/// A page: what `/catalogs` shows, in the layout's outlet.
struct Catalogs;

impl View<App> for Catalogs {
    type Parent = Document;
    const PATH: &'static str = "/catalogs";
    const TITLE: &'static str = "Catalogs";

    async fn render(_: &App) -> impl Into<Html> {
        html! { h1 { "Catalogs" } }
    }
}

let router: Router = Router::new().view::<Catalogs>().with_state(App);
```

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

- **URL as state.** A link opens a page, or a section inside it, and the address bar always says what is on screen.
- **Convention over configuration.** Sensible defaults first; every one of them can be overridden.
- **The server leads.** The server decides what each response contains; the browser shows it.
- **Unopinionated core.** The core writes no HTML and chooses no styling. [maud](https://maud.lambda.xyz) is supported out of the box and can be switched off.
- **Ready for agents.** Few concepts, one way to do each thing, and mistakes caught by the compiler.

## Documentation

- [Glossary](GLOSSARY.md): the words this project uses.
- [Decisions](docs/adr): why it is built the way it is.

## Planned

- Views, layouts and nested sections served from one axum router
- Forms
- Regions: parts of a page refreshed on their own
- File-system routing
- A starter with a layout and base components

## License

Licensed under the [MIT License](LICENSE).
