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
use tower::ServiceExt;

#[derive(Clone)]
struct App {
    name: &'static str,
}

struct Document;

impl View for Document {
    type State = App;
    type Parent = NoParent;

    async fn render(app: &App, outlet: Outlet) -> impl Into<Html> {
        Html::trusted(format!(
            "<title>{} - {}</title><main id=\"{}\">{}</main>",
            outlet.title,
            app.name,
            outlet.id,
            outlet.content.as_str()
        ))
    }
}

struct Catalogs;

impl View for Catalogs {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/catalogs";
    const TITLE: &'static str = "Catalogs";

    async fn render(app: &App, outlet: Outlet) -> impl Into<Html> {
        Html::trusted(format!(
            "<h1>Catalogs of {}</h1><div id=\"{}\">{}</div>",
            app.name,
            outlet.id,
            outlet.content.as_str()
        ))
    }
}

/// A view nested in [`Catalogs`].
struct Archived;

impl View for Archived {
    type State = App;
    type Parent = Catalogs;
    const PATH: &'static str = "/archived";
    const TITLE: &'static str = "Archived catalogs";

    async fn render(_: &App, _: Outlet) -> impl Into<Html> {
        Html::trusted("<p>None archived</p>")
    }
}

/// A second view that claims the path of [`Catalogs`].
struct AlsoCatalogs;

impl View for AlsoCatalogs {
    type State = App;
    type Parent = Document;
    const PATH: &'static str = "/catalogs";

    async fn render(_: &App, _: Outlet) -> impl Into<Html> {
        Html::default()
    }
}

fn router() -> Router {
    Router::new()
        .view::<Catalogs>()
        .view::<Archived>()
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

    struct Home;

    impl View for Home {
        type State = App;
        type Parent = MaudDocument;
        const PATH: &'static str = "/";
        const TITLE: &'static str = "Home";

        async fn render(app: &App, _: Outlet) -> impl Into<Html> {
            html! { h1 { "Welcome to " (app.name) } }
        }
    }

    struct MaudDocument;

    impl View for MaudDocument {
        type State = App;
        type Parent = NoParent;

        async fn render(_: &App, outlet: Outlet) -> impl Into<Html> {
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
