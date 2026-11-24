//! The demo app: what htmx-rust does, in unstyled HTML. The browser suite in
//! `browser-tests` runs against it.

use std::{
    env, fs,
    sync::{Arc, Mutex},
};

use axum::{
    Router,
    extract::{Path, State},
    http::{
        StatusCode,
        header::{CONTENT_TYPE, LOCATION},
    },
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use htmx_rust::{Html, NoParent, Outlet, View, Views};
use maud::{DOCTYPE, html};

/// htmx 4, where `pnpm install` puts it.
const HTMX: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/node_modules/htmx.org/dist/htmx.min.js"
);

/// What the app keeps, in memory: the catalogs, each with its id and its name.
#[derive(Default)]
struct Store {
    /// How many catalogs were ever started; the next one's id follows from it.
    started: u32,
    catalogs: Vec<(u32, String)>,
}

type App = Arc<Mutex<Store>>;

/// The root view: the document, with the navigation around its outlet.
struct Document;

impl View for Document {
    type State = App;
    type Parent = NoParent;

    async fn render(_: &App, outlet: Outlet) -> impl Into<Html> {
        let target = format!("#{}", outlet.id);
        html! {
            (DOCTYPE)
            html lang="en" {
                head {
                    meta charset="utf-8";
                    title { (outlet.title) }
                    script src="/htmx.js" defer {}
                }
                // Set once and inherited by every link and form below: htmx
                // sends them itself and shows the answer in this view's outlet.
                //
                // Until the server cuts the response by the outlet the request
                // names (ADR 1), every answer is the whole document, so the
                // browser cuts: `hx-select` and `hx-swap` take the outlet out
                // of the answer and put it in the place of this one.
                body
                    hx-boost:inherited="true"
                    hx-target:inherited=(target)
                    hx-select:inherited=(target)
                    hx-swap:inherited="outerHTML"
                {
                    nav {
                        a href="/" { "Home" }
                        " "
                        a href="/catalogs" { "Catalogs" }
                    }
                    main id=(outlet.id) { (outlet.content) }
                }
            }
        }
    }
}

struct Home;

impl View for Home {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/";
    const TITLE: &'static str = "Home";

    async fn render(_: &App, _: Outlet) -> impl Into<Html> {
        html! {
            h1 { "Home" }
            p { "Every address here shows the same thing on a click and on a refresh." }
            p { a href="/catalogs" { "See the catalogs" } }
            form method="post" action="/catalogs" {
                button { "Start a catalog" }
            }
        }
    }
}

struct Catalogs;

impl View for Catalogs {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/catalogs";
    const TITLE: &'static str = "Catalogs";

    async fn render(app: &App, _: Outlet) -> impl Into<Html> {
        let store = app.lock().unwrap();
        html! {
            h1 { "Catalogs" }
            @if store.catalogs.is_empty() {
                p { "No catalogs yet." }
            }
            ul {
                @for (id, name) in &store.catalogs {
                    li {
                        (name)
                        " "
                        form method="post" action={ "/catalogs/" (id) "/remove" } {
                            button { "Remove " (name) }
                        }
                    }
                }
            }
            p { a href="/" { "Back home" } }
        }
    }
}

/// Starts a catalog and answers `201 Created` with the whole document around
/// a message. Actions will draw their answer in place themselves; until then
/// the root view is rendered by hand, and `Location` names the collection
/// because a catalog has no view of its own yet.
async fn start_catalog(State(app): State<App>) -> impl IntoResponse {
    let name = {
        let mut store = app.lock().unwrap();
        store.started += 1;
        let id = store.started;
        let name = format!("Catalog {id}");
        store.catalogs.push((id, name.clone()));
        name
    };
    let outlet = Outlet {
        id: "outlet-0".into(),
        content: html! {
            h1 { "Catalog created" }
            p { strong { (name) } " is among the catalogs now." }
            p { a href="/catalogs" { "See the catalogs" } }
        }
        .into(),
        title: "Catalog created",
    };
    let document: Html = Document::render(&app, outlet).await.into();
    (
        StatusCode::CREATED,
        [
            (LOCATION, "/catalogs"),
            (CONTENT_TYPE, "text/html; charset=utf-8"),
        ],
        document.as_str().to_owned(),
    )
}

/// Removes a catalog and answers `303 See Other` to the collection.
async fn remove_catalog(State(app): State<App>, Path(id): Path<u32>) -> Redirect {
    app.lock()
        .unwrap()
        .catalogs
        .retain(|(other, _)| *other != id);
    Redirect::to("/catalogs")
}

#[tokio::main]
async fn main() {
    let htmx = fs::read_to_string(HTMX).unwrap_or_else(|error| {
        panic!("{HTMX}: {error}; run `pnpm install` in the demo directory")
    });

    let router = Router::new()
        .view::<Home>()
        .view::<Catalogs>()
        // Ordinary axum routes, beside the views.
        .route("/catalogs", post(start_catalog))
        .route("/catalogs/{id}/remove", post(remove_catalog))
        .route(
            "/htmx.js",
            get(|| async move { ([(CONTENT_TYPE, "text/javascript")], htmx) }),
        )
        .with_state(App::default());

    let port = env::var("PORT").unwrap_or_else(|_| "3000".into());
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    println!("http://{}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();
}
