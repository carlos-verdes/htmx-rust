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
use serde::{Deserialize, Serialize};

/// htmx 4, where `pnpm install` puts it.
const HTMX: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/node_modules/htmx.org/dist/htmx.min.js"
);

/// The tags a catalog can carry.
const TAGS: [&str; 4] = ["tea", "coffee", "hot", "iced"];

struct Record {
    id: u32,
    name: String,
    tags: Vec<&'static str>,
    items: Vec<&'static str>,
}

/// What the app keeps, in memory: the catalogs.
struct Store {
    /// How many catalogs were ever started; the next one's id follows from it.
    started: u32,
    catalogs: Vec<Record>,
}

impl Store {
    fn catalog(&self, id: u32) -> Option<&Record> {
        self.catalogs.iter().find(|catalog| catalog.id == id)
    }
}

/// The catalogs the demo starts with.
impl Default for Store {
    fn default() -> Self {
        let catalogs = [
            ("Spring teas", ["tea", "hot"], ["Sencha", "Darjeeling"]),
            (
                "Summer teas",
                ["tea", "iced"],
                ["Iced oolong", "Cold brew green"],
            ),
            (
                "Autumn coffees",
                ["coffee", "iced"],
                ["Cold brew", "Iced latte"],
            ),
            (
                "Winter coffees",
                ["coffee", "hot"],
                ["Espresso", "Flat white"],
            ),
        ];
        Self {
            started: catalogs.len() as u32,
            catalogs: (1..)
                .zip(catalogs)
                .map(|(id, (name, tags, items))| Record {
                    id,
                    name: name.into(),
                    tags: tags.into(),
                    items: items.into(),
                })
                .collect(),
        }
    }
}

type App = Arc<Mutex<Store>>;

/// The root view: the document, with the navigation around its outlet.
#[derive(Serialize, Deserialize)]
struct Document;

impl View for Document {
    type State = App;
    type Parent = NoParent;

    async fn render(&self, _: &App, outlet: Outlet) -> impl Into<Html> {
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
                        a href=(Home.url()) { "Home" }
                        " "
                        a href=(Catalogs::default().url()) { "Catalogs" }
                    }
                    main id=(outlet.id) { (outlet.content) }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Home;

impl View for Home {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/";
    const TITLE: &'static str = "Home";

    async fn render(&self, _: &App, _: Outlet) -> impl Into<Html> {
        html! {
            h1 { "Home" }
            p { "Every address here shows the same thing on a click and on a refresh." }
            p { a href=(Catalogs::default().url()) { "See the catalogs" } }
            form method="post" action=(Catalogs::default().url()) {
                button { "Start a catalog" }
            }
        }
    }
}

/// The collection: every catalog, or those the search in its URL finds.
/// `/catalogs?q=tea&tag=hot&tag=iced` is a search like any other address.
#[derive(Default, Serialize, Deserialize)]
struct Catalogs {
    /// What a catalog's name must contain. `/catalogs` searches for nothing.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    q: String,
    /// The tags a catalog must carry, all of them. The query string repeats
    /// the name, once for each.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tag: Vec<String>,
}

impl View for Catalogs {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/catalogs";
    const TITLE: &'static str = "Catalogs";

    async fn render(&self, app: &App, _: Outlet) -> impl Into<Html> {
        let q = self.q.to_lowercase();
        let store = app.lock().unwrap();
        let found = store
            .catalogs
            .iter()
            .filter(|catalog| catalog.name.to_lowercase().contains(&q))
            .filter(|catalog| {
                self.tag
                    .iter()
                    .all(|tag| catalog.tags.contains(&tag.as_str()))
            });
        html! {
            h1 { "Catalogs" }
            // A GET form is a link whose query string comes from its fields:
            // each is named after a field of the view at its address.
            form action=(Catalogs::default().url()) {
                label { "Name " input type="search" name="q" value=(self.q); }
                " "
                @for tag in TAGS {
                    label {
                        input type="checkbox" name="tag" value=(tag)
                            checked[self.tag.iter().any(|checked| checked == tag)];
                        " " (tag)
                    }
                    " "
                }
                button { "Search" }
            }
            ul {
                @for Record { id, name, .. } in found {
                    li {
                        a href=(Catalog { id: *id }.url()) { (name) }
                        " "
                        form method="post" action={ "/catalogs/" (id) "/remove" } {
                            button { "Remove " (name) }
                        }
                    }
                }
            }
            p { a href=(Home.url()) { "Back home" } }
        }
    }
}

/// One catalog, the resource its path parameter names. It is nested in the
/// root view, beside the collection and not inside it, although its path
/// extends the collection's: the path says nothing about nesting.
#[derive(Serialize, Deserialize)]
struct Catalog {
    id: u32,
}

impl View for Catalog {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/catalogs/{id}";
    const TITLE: &'static str = "Catalog";

    async fn render(&self, app: &App, outlet: Outlet) -> impl Into<Html> {
        let Self { id } = *self;
        let store = app.lock().unwrap();
        html! {
            @match store.catalog(id) {
                Some(catalog) => {
                    h1 { (catalog.name) }
                    nav {
                        a href=(Items { id }.url()) { "Items" }
                        " "
                        a href=(Tags { id }.url()) { "Tags" }
                    }
                    section id=(outlet.id) { (outlet.content) }
                }
                // A missing resource is not yet a failure (ADR 4).
                None => p { "There is no catalog " (id) "." },
            }
            p { a href=(Catalogs::default().url()) { "All catalogs" } }
        }
    }
}

/// A catalog's tags, nested in the catalog: `/catalogs/{id}/tags`. Its `id` is
/// its parent's parameter, which it holds to load its own data and to print
/// its own URL.
#[derive(Serialize, Deserialize)]
struct Tags {
    id: u32,
}

impl View for Tags {
    type State = App;
    type Parent = Catalog;
    const PATH: &'static str = "/tags";
    const TITLE: &'static str = "Tags";

    async fn render(&self, app: &App, _: Outlet) -> impl Into<Html> {
        let store = app.lock().unwrap();
        let tags = store.catalog(self.id).map(|catalog| &catalog.tags);
        html! {
            h2 { "Tags" }
            ul {
                @for tag in tags.into_iter().flatten() {
                    li { (tag) }
                }
            }
        }
    }
}

/// A catalog's items, nested in the catalog: `/catalogs/{id}/items`. The one
/// the URL names is shown in its outlet, whose id it chooses itself.
#[derive(Serialize, Deserialize)]
struct Items {
    id: u32,
}

impl View for Items {
    type State = App;
    type Parent = Catalog;
    const PATH: &'static str = "/items";
    const TITLE: &'static str = "Items";
    const OUTLET: Option<&'static str> = Some("item");

    async fn render(&self, app: &App, outlet: Outlet) -> impl Into<Html> {
        let Self { id } = *self;
        let store = app.lock().unwrap();
        let items = store.catalog(id).map(|catalog| &catalog.items);
        html! {
            h2 { "Items" }
            ul {
                @for &item in items.into_iter().flatten() {
                    li { a href=(Item { id, item: item.into() }.url()) { (item) } }
                }
            }
            div id=(outlet.id) { (outlet.content) }
        }
    }
}

/// One item of a catalog, nested in the catalog's items at depth three:
/// `/catalogs/{id}/items/{item}`.
#[derive(Serialize, Deserialize)]
struct Item {
    id: u32,
    item: String,
}

impl View for Item {
    type State = App;
    type Parent = Items;
    const PATH: &'static str = "/{item}";
    const TITLE: &'static str = "Item";

    async fn render(&self, app: &App, _: Outlet) -> impl Into<Html> {
        let store = app.lock().unwrap();
        let catalog = store
            .catalog(self.id)
            .filter(|catalog| catalog.items.contains(&self.item.as_str()));
        html! {
            @match catalog {
                Some(catalog) => {
                    h3 { (self.item) }
                    p { "One of the items of " (catalog.name) "." }
                }
                None => p { "There is no item " (self.item) " here." },
            }
        }
    }
}

/// Starts a catalog and answers `201 Created` with the whole document around
/// a message that links to it. Actions will draw their answer in place
/// themselves; until then the root view is rendered by hand.
async fn start_catalog(State(app): State<App>) -> impl IntoResponse {
    let (catalog, name) = {
        let mut store = app.lock().unwrap();
        store.started += 1;
        let id = store.started;
        let name = format!("Catalog {id}");
        store.catalogs.push(Record {
            id,
            name: name.clone(),
            tags: Vec::new(),
            items: Vec::new(),
        });
        (Catalog { id }, name)
    };
    let outlet = Outlet {
        id: "outlet-0".into(),
        content: html! {
            h1 { "Catalog created" }
            p { strong { (name) } " is among the catalogs now." }
            p { a href=(catalog.url()) { "View " (name) } }
            p { a href=(Catalogs::default().url()) { "See the catalogs" } }
        }
        .into(),
        title: "Catalog created",
    };
    let document: Html = Document.render(&app, outlet).await.into();
    (
        StatusCode::CREATED,
        [
            (LOCATION, catalog.url()),
            (CONTENT_TYPE, "text/html; charset=utf-8".into()),
        ],
        document.as_str().to_owned(),
    )
}

/// Removes a catalog and answers `303 See Other` to the collection.
async fn remove_catalog(State(app): State<App>, Path(id): Path<u32>) -> Redirect {
    app.lock()
        .unwrap()
        .catalogs
        .retain(|catalog| catalog.id != id);
    Redirect::to(&Catalogs::default().url())
}

#[tokio::main]
async fn main() {
    let htmx = fs::read_to_string(HTMX).unwrap_or_else(|error| {
        panic!("{HTMX}: {error}; run `pnpm install` in the demo directory")
    });

    let router = Router::new()
        .view::<Home>()
        .view::<Catalogs>()
        .view::<Catalog>()
        .view::<Tags>()
        .view::<Items>()
        .view::<Item>()
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
