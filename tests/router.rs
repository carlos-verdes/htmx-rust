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
        Html::trusted(self.url())
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
        .view::<Entry>()
        .view::<Featured>()
        .view::<Unfilled>()
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
async fn path_parameters_and_the_query_string_are_parsed_into_the_views_fields() {
    let response = get_path(router(), "/catalogs/7/entries/tea?page=2&q=green%20leaf").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body(response).await,
        "<title>Entry - Loom</title><main id=\"outlet-0\">\
         <a href=\"/catalogs/7/entries/tea?q=green+leaf&page=2\">\
         Entry { catalog: 7, name: \"tea\", q: \"green leaf\", page: Some(2) }</a></main>"
    );
}

#[tokio::test]
async fn a_field_the_url_leaves_out_takes_its_default() {
    let response = get_path(router(), "/catalogs/7/entries/tea").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(body(response).await.contains(
        "<a href=\"/catalogs/7/entries/tea?q=\">\
             Entry { catalog: 7, name: \"tea\", q: \"\", page: None }</a>"
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
        "/catalogs/7/entries/red%20%26%20white%2Fros%C3%A9?q=a%2Bb+c%3F&page=2"
    );

    let response = get_path(router(), url).await;

    assert_eq!(response.status(), StatusCode::OK);
    let entry = body(response).await;
    assert!(entry.contains(
        "Entry { catalog: 7, name: \"red & white/rosé\", q: \"a+b c?\", page: Some(2) }"
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

#[tokio::test]
#[should_panic(expected = "/unfilled/{id}")]
async fn printing_a_view_whose_path_has_a_parameter_without_a_field_is_an_error() {
    get_path(router(), "/unfilled/1").await;
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
