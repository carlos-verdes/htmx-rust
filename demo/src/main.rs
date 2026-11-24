//! The demo app: what htmx-rust does, in unstyled HTML. The browser suite in
//! `browser-tests` runs against it.

use std::{env, fs};

use axum::{
    Router,
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

/// The root view: the document, with the navigation around its outlet.
struct Document;

impl View for Document {
    type State = ();
    type Parent = NoParent;

    async fn render(_: &(), outlet: Outlet) -> impl Into<Html> {
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
    type State = ();
    type Parent = Document;
    const PATH: &'static str = "/";
    const TITLE: &'static str = "Home";

    async fn render(_: &(), _: Outlet) -> impl Into<Html> {
        html! {
            h1 { "Home" }
            p { "Every address here shows the same thing on a click and on a refresh." }
            p { a href="/catalogs" { "See the catalogs" } }
            form method="post" action="/catalogs" {
                button { "Start a catalog" }
            }
            form method="post" action="/catalogs/clear" {
                button { "Clear the catalogs" }
            }
        }
    }
}

struct Catalogs;

impl View for Catalogs {
    type State = ();
    type Parent = Document;
    const PATH: &'static str = "/catalogs";
    const TITLE: &'static str = "Catalogs";

    async fn render(_: &(), _: Outlet) -> impl Into<Html> {
        html! {
            h1 { "Catalogs" }
            p { "No catalogs yet." }
            p { a href="/" { "Back home" } }
        }
    }
}

/// Answers `201 Created` with the whole document around a message. Actions
/// will draw their answer in place themselves; until then the root view is
/// rendered by hand, and `Location` names the collection because a catalog has
/// no view of its own yet.
async fn create_catalog() -> impl IntoResponse {
    let outlet = Outlet {
        id: "outlet-0".into(),
        content: html! {
            h1 { "Catalog created" }
            p { a href="/catalogs" { "See the catalogs" } }
        }
        .into(),
        title: "Catalog created",
    };
    let document: Html = Document::render(&(), outlet).await.into();
    (
        StatusCode::CREATED,
        [
            (LOCATION, "/catalogs"),
            (CONTENT_TYPE, "text/html; charset=utf-8"),
        ],
        document.as_str().to_owned(),
    )
}

#[tokio::main]
async fn main() {
    let htmx = fs::read_to_string(HTMX).unwrap_or_else(|error| {
        panic!("{HTMX}: {error}; run `pnpm install` in the demo directory")
    });

    let router = Router::new()
        .view::<Home>()
        .view::<Catalogs>()
        // Ordinary axum routes, beside the views. Nothing is stored yet.
        .route("/catalogs", post(create_catalog))
        .route(
            "/catalogs/clear",
            post(|| async { Redirect::to("/catalogs") }),
        )
        .route(
            "/htmx.js",
            get(|| async move { ([(CONTENT_TYPE, "text/javascript")], htmx) }),
        );

    let port = env::var("PORT").unwrap_or_else(|_| "3000".into());
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    println!("http://{}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();
}
