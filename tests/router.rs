//! Every test builds a router, sends one request and looks only at the
//! response's status, headers and body.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
    response::Response,
    routing::get,
};
use htmx_rust::{Html, NoParent, Outlet, View, Views};
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

#[derive(Clone)]
struct App {
    name: &'static str,
}

#[derive(Serialize, Deserialize)]
struct Document;

impl View for Document {
    type State = App;
    type Parent = NoParent;

    async fn render(&self, app: &App, outlet: Outlet) -> impl Into<Html> {
        Html::trusted(format!(
            "<title>{} - {}</title><main id=\"{}\">{}</main>",
            outlet.title,
            app.name,
            outlet.id,
            outlet.content.as_str()
        ))
    }
}

#[derive(Serialize, Deserialize)]
struct Catalogs;

impl View for Catalogs {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/catalogs";
    const TITLE: &'static str = "Catalogs";

    async fn render(&self, app: &App, outlet: Outlet) -> impl Into<Html> {
        Html::trusted(format!(
            "<h1>Catalogs of {}</h1><div id=\"{}\">{}</div>",
            app.name,
            outlet.id,
            outlet.content.as_str()
        ))
    }
}

/// A view nested in [`Catalogs`].
#[derive(Serialize, Deserialize)]
struct Archived;

impl View for Archived {
    type State = App;
    type Parent = Catalogs;
    const PATH: &'static str = "/archived";
    const TITLE: &'static str = "Archived catalogs";

    async fn render(&self, _: &App, _: Outlet) -> impl Into<Html> {
        Html::trusted("<p>None archived</p>")
    }
}

/// A view whose URL carries its fields: two in the path, the others in the
/// query string. It shows itself as a link to itself.
#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    catalog: u32,
    name: String,
    #[serde(default)]
    q: String,
    page: Option<u32>,
    #[serde(default)]
    tag: Vec<String>,
}

impl View for Entry {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/catalogs/{catalog}/entries/{name}";
    const TITLE: &'static str = "Entry";

    async fn render(&self, _: &App, _: Outlet) -> impl Into<Html> {
        Html::trusted(format!("<a href=\"{}\">{self:?}</a>", self.url()))
    }
}

/// One catalog, nested in the root view beside [`Catalogs`], although its path
/// extends that one's. It names its outlet's id itself.
#[derive(Serialize, Deserialize)]
struct Catalog {
    catalog: u32,
}

impl View for Catalog {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/catalogs/{catalog}";
    const TITLE: &'static str = "Catalog";
    const OUTLET: Option<&'static str> = Some("catalog");

    async fn render(&self, _: &App, outlet: Outlet) -> impl Into<Html> {
        Html::trusted(format!(
            "<h2>Catalog {}</h2><section id=\"{}\">{}</section>",
            self.catalog,
            outlet.id,
            outlet.content.as_str()
        ))
    }
}

/// A view nested in [`Catalog`], at depth two. It holds its parent's parameter
/// and shows itself as a link to itself.
#[derive(Serialize, Deserialize)]
struct Notes {
    catalog: u32,
}

impl View for Notes {
    type State = App;
    type Parent = Catalog;
    const PATH: &'static str = "/notes";
    const TITLE: &'static str = "Notes";

    async fn render(&self, _: &App, outlet: Outlet) -> impl Into<Html> {
        Html::trusted(format!(
            "<a href=\"{}\">Notes of {}</a><div id=\"{}\">{}</div>",
            self.url(),
            self.catalog,
            outlet.id,
            outlet.content.as_str()
        ))
    }
}

/// A view nested in [`Notes`], at depth three, with a parameter of its own.
#[derive(Debug, Serialize, Deserialize)]
struct Note {
    catalog: u32,
    note: String,
    page: Option<u32>,
}

impl View for Note {
    type State = App;
    type Parent = Notes;
    const PATH: &'static str = "/{note}";
    const TITLE: &'static str = "Note";

    async fn render(&self, _: &App, _: Outlet) -> impl Into<Html> {
        Html::trusted(format!("<a href=\"{}\">{self:?}</a>", self.url()))
    }
}

/// One catalog again, at the path of [`Catalog`], but nested in [`Catalogs`].
#[derive(Serialize, Deserialize)]
struct Listed {
    catalog: u32,
}

impl View for Listed {
    type State = App;
    type Parent = Catalogs;
    const PATH: &'static str = "/{catalog}";
    const TITLE: &'static str = "Catalog";

    async fn render(&self, _: &App, _: Outlet) -> impl Into<Html> {
        Html::trusted(format!("<h2>Catalog {}</h2>", self.catalog))
    }
}

/// A view nested in [`Catalog`] that does not hold its parent's parameter.
#[derive(Serialize, Deserialize)]
struct Unheld;

impl View for Unheld {
    type State = App;
    type Parent = Catalog;
    const PATH: &'static str = "/unheld";

    async fn render(&self, _: &App, _: Outlet) -> impl Into<Html> {
        Html::default()
    }
}

/// A view whose path is `/` alone.
#[derive(Serialize, Deserialize)]
struct Home;

impl View for Home {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/";
    const TITLE: &'static str = "Home";

    async fn render(&self, _: &App, outlet: Outlet) -> impl Into<Html> {
        Html::trusted(format!("<h1>Home</h1>{}", outlet.content.as_str()))
    }
}

/// A view nested in [`Home`], which shows itself as a link to itself.
#[derive(Serialize, Deserialize)]
struct Activity;

impl View for Activity {
    type State = App;
    type Parent = Home;
    const PATH: &'static str = "/activity";
    const TITLE: &'static str = "Activity";

    async fn render(&self, _: &App, _: Outlet) -> impl Into<Html> {
        Html::trusted(format!("<a href=\"{}\">Activity</a>", self.url()))
    }
}

/// A view that links to another, from a value.
#[derive(Serialize, Deserialize)]
struct Featured;

impl View for Featured {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/featured";

    async fn render(&self, _: &App, _: Outlet) -> impl Into<Html> {
        let entry = Entry {
            catalog: 7,
            name: "red & white/rosé".into(),
            q: "a+b c?".into(),
            page: Some(2),
            tag: vec!["red".into(), "dry".into()],
        };
        Html::trusted(format!("<a href=\"{}\">Featured</a>", entry.url()))
    }
}

/// A view whose path has a parameter that no field fills.
#[derive(Serialize, Deserialize)]
struct Unfilled;

impl View for Unfilled {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/unfilled/{id}";

    async fn render(&self, _: &App, _: Outlet) -> impl Into<Html> {
        Html::default()
    }
}

/// A second view that claims the path of [`Catalogs`].
#[derive(Serialize, Deserialize)]
struct AlsoCatalogs;

impl View for AlsoCatalogs {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/catalogs";

    async fn render(&self, _: &App, _: Outlet) -> impl Into<Html> {
        Html::default()
    }
}

fn router() -> Router {
    Router::new()
        .view::<Catalogs>()
        .view::<Archived>()
        .view::<Catalog>()
        .view::<Notes>()
        .view::<Note>()
        .view::<Home>()
        .view::<Activity>()
        .view::<Entry>()
        .view::<Featured>()
        .route("/health", get(|| async { "ok" }))
        .with_state(App { name: "Loom" })
}

async fn get_path(router: Router, path: &str) -> Response {
    let request = Request::get(path).body(Body::empty()).unwrap();
    router.oneshot(request).await.unwrap()
}

async fn body(response: Response) -> String {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn a_view_answers_at_its_path_inside_the_root_view() {
    let response = get_path(router(), "/catalogs").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "text/html; charset=utf-8");
    assert_eq!(
        body(response).await,
        "<title>Catalogs - Loom</title><main id=\"outlet-0\">\
         <h1>Catalogs of Loom</h1><div id=\"outlet-1\"></div></main>"
    );
}

#[tokio::test]
async fn a_nested_view_answers_at_its_ancestors_path_and_its_own_inside_them() {
    let response = get_path(router(), "/catalogs/archived").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body(response).await,
        "<title>Archived catalogs - Loom</title><main id=\"outlet-0\">\
         <h1>Catalogs of Loom</h1><div id=\"outlet-1\"><p>None archived</p></div></main>"
    );
}

#[tokio::test]
async fn a_view_nested_in_one_with_a_parameter_answers_inside_it_at_depth_two() {
    let response = get_path(router(), "/catalogs/7/notes").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body(response).await,
        "<title>Notes - Loom</title><main id=\"outlet-0\">\
         <h2>Catalog 7</h2><section id=\"catalog\">\
         <a href=\"/catalogs/7/notes\">Notes of 7</a><div id=\"outlet-2\"></div>\
         </section></main>"
    );
}

#[tokio::test]
async fn a_view_at_depth_three_answers_inside_every_ancestor_and_prints_its_url() {
    let response = get_path(router(), "/catalogs/7/notes/first%20draft?page=2").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body(response).await,
        "<title>Note - Loom</title><main id=\"outlet-0\">\
         <h2>Catalog 7</h2><section id=\"catalog\">\
         <a href=\"/catalogs/7/notes\">Notes of 7</a><div id=\"outlet-2\">\
         <a href=\"/catalogs/7/notes/first%20draft?page=2\">\
         Note { catalog: 7, note: \"first draft\", page: Some(2) }</a>\
         </div></section></main>"
    );
}

/// The ids of the outlets in `html`, outermost first.
fn outlets(html: &str) -> Vec<&str> {
    let ids = html.split(" id=\"").skip(1);
    ids.map(|rest| rest.split_once('"').unwrap().0).collect()
}

#[tokio::test]
async fn an_outlets_id_follows_its_views_depth_unless_the_view_names_one() {
    let catalogs = body(get_path(router(), "/catalogs/archived").await).await;
    let notes = body(get_path(router(), "/catalogs/7/notes/first").await).await;

    assert_eq!(outlets(&catalogs), ["outlet-0", "outlet-1"]);
    // The one a view names does not shift those that follow it.
    assert_eq!(outlets(&notes), ["outlet-0", "catalog", "outlet-2"]);
}

#[tokio::test]
async fn a_path_that_extends_anothers_is_nested_in_it_only_when_declared_so() {
    let beside = Router::new().view::<Catalogs>().view::<Catalog>();
    let inside = Router::new().view::<Catalogs>().view::<Listed>();
    let app = App { name: "Loom" };

    let beside = get_path(beside.with_state(app.clone()), "/catalogs/7").await;
    let inside = get_path(inside.with_state(app), "/catalogs/7").await;

    assert_eq!(
        body(beside).await,
        "<title>Catalog - Loom</title><main id=\"outlet-0\">\
         <h2>Catalog 7</h2><section id=\"catalog\"></section></main>"
    );
    assert_eq!(
        body(inside).await,
        "<title>Catalog - Loom</title><main id=\"outlet-0\">\
         <h1>Catalogs of Loom</h1><div id=\"outlet-1\"><h2>Catalog 7</h2></div></main>"
    );
}

#[tokio::test]
async fn a_view_nested_in_the_one_at_the_root_path_is_not_served_behind_two_slashes() {
    let response = get_path(router(), "/activity").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body(response).await,
        "<title>Activity - Loom</title><main id=\"outlet-0\">\
         <h1>Home</h1><a href=\"/activity\">Activity</a></main>"
    );
}

#[tokio::test]
async fn the_root_view_on_its_own_is_served_at_the_root_path() {
    let router = Router::new().view::<Document>();

    let response = get_path(router.with_state(App { name: "Loom" }), "/").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body(response).await,
        "<title> - Loom</title><main id=\"outlet-0\"></main>"
    );
}

#[tokio::test]
async fn path_parameters_and_the_query_string_are_parsed_into_the_views_fields() {
    let response = get_path(router(), "/catalogs/7/entries/tea?page=2&q=green%20leaf").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body(response).await,
        "<title>Entry - Loom</title><main id=\"outlet-0\">\
         <a href=\"/catalogs/7/entries/tea?q=green+leaf&page=2\">\
         Entry { catalog: 7, name: \"tea\", q: \"green leaf\", page: Some(2), tag: [] }</a></main>"
    );
}

#[tokio::test]
async fn a_field_the_url_leaves_out_takes_its_default() {
    let response = get_path(router(), "/catalogs/7/entries/tea").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(body(response).await.contains(
        "<a href=\"/catalogs/7/entries/tea?q=\">\
             Entry { catalog: 7, name: \"tea\", q: \"\", page: None, tag: [] }</a>"
    ));
}

/// The `href` of the one link in `html`.
fn link(html: &str) -> &str {
    let (_, rest) = html.split_once("<a href=\"").unwrap();
    rest.split_once('"').unwrap().0
}

#[tokio::test]
async fn a_link_rendered_from_a_view_value_requested_back_answers_with_that_view() {
    let featured = body(get_path(router(), "/featured").await).await;
    let url = link(&featured);
    assert_eq!(
        url,
        "/catalogs/7/entries/red%20%26%20white%2Fros%C3%A9?q=a%2Bb+c%3F&page=2&tag=red&tag=dry"
    );

    let response = get_path(router(), url).await;

    assert_eq!(response.status(), StatusCode::OK);
    let entry = body(response).await;
    assert!(entry.contains(
        "Entry { catalog: 7, name: \"red & white/rosé\", q: \"a+b c?\", page: Some(2), tag: [\"red\", \"dry\"] }"
    ));
    // The view the URL was parsed into prints the URL it came from.
    assert_eq!(link(&entry), url);
}

#[tokio::test]
async fn a_view_with_no_fields_answers_whatever_the_query_string() {
    let response = get_path(router(), "/catalogs?utm_source=mail").await;

    assert_eq!(response.status(), StatusCode::OK);
}

/// A client error, and nothing of any view in the body.
async fn assert_rejected(path: &str) {
    let response = get_path(router(), path).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{path}");
    assert_eq!(
        response.headers()[CONTENT_TYPE],
        "text/plain; charset=utf-8"
    );
    let body = body(response).await;
    assert!(!body.contains("Loom") && !body.contains("Entry"), "{body}");
}

#[tokio::test]
async fn a_path_parameter_that_does_not_parse_is_rejected_and_no_view_runs() {
    assert_rejected("/catalogs/seven/entries/tea").await;
}

#[tokio::test]
async fn a_query_string_that_does_not_parse_is_rejected_and_no_view_runs() {
    assert_rejected("/catalogs/7/entries/tea?page=two").await;
}

#[tokio::test]
async fn a_query_string_that_repeats_a_path_parameter_is_rejected() {
    assert_rejected("/catalogs/7/entries/tea?catalog=8").await;
}

#[test]
#[should_panic(expected = "no field for the parameter {id} of its path: /unfilled/{id}")]
fn a_path_parameter_without_a_field_is_an_error_at_startup() {
    let _: Router<App> = Router::new().view::<Unfilled>();
}

#[test]
#[should_panic(
    expected = "no field for the parameter {catalog} of its path: /catalogs/{catalog}/unheld"
)]
fn an_ancestors_path_parameter_without_a_field_is_an_error_at_startup() {
    let _: Router<App> = Router::new().view::<Unheld>();
}

#[tokio::test]
async fn ordinary_routes_answer_beside_views() {
    let response = get_path(router(), "/health").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body(response).await, "ok");
}

#[tokio::test]
async fn a_path_no_view_claims_is_not_found() {
    let response = get_path(router(), "/nothing").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
#[should_panic(expected = "/catalogs")]
fn two_views_on_one_path_are_an_error_at_startup() {
    let _: Router<App> = Router::new().view::<Catalogs>().view::<AlsoCatalogs>();
}

#[cfg(feature = "maud")]
mod with_maud {
    use maud::html;

    use super::*;

    #[derive(Serialize, Deserialize)]
    struct Home;

    impl View for Home {
        type State = App;
        type Parent = MaudDocument;
        const PATH: &'static str = "/";
        const TITLE: &'static str = "Home";

        async fn render(&self, app: &App, _: Outlet) -> impl Into<Html> {
            html! { h1 { "Welcome to " (app.name) } }
        }
    }

    #[derive(Serialize, Deserialize)]
    struct MaudDocument;

    impl View for MaudDocument {
        type State = App;
        type Parent = NoParent;

        async fn render(&self, _: &App, outlet: Outlet) -> impl Into<Html> {
            html! {
                title { (outlet.title) }
                main id=(outlet.id) { (outlet.content) }
            }
        }
    }

    #[tokio::test]
    async fn maud_markup_becomes_html_and_holds_an_outlet() {
        let router = Router::new()
            .view::<Home>()
            .with_state(App { name: "<Loom>" });

        let response = get_path(router, "/").await;

        assert_eq!(
            body(response).await,
            "<title>Home</title><main id=\"outlet-0\"><h1>Welcome to &lt;Loom&gt;</h1></main>"
        );
    }
}
