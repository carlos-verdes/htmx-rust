use std::future::Future;

use axum::{
    Router,
    extract::{RawPathParams, RawQuery, State},
    http::StatusCode,
    routing::get,
};
use serde::{Serialize, de::DeserializeOwned};

use crate::{Html, url};
use chain::Chain;

/// What one URL shows, nested in its [`Parent`](View::Parent).
///
/// A view's fields are what its URL carries: the parameters its path names
/// (`{id}` fills the field `id`), and the query string for every other field.
/// Each field holds a plain value, such as a number, a string or an `Option`
/// of one, or a `Vec` of them, which the query string carries by repeating the
/// field's name (`?tag=tea&tag=green`). A request's URL is parsed into the view, and
/// [`url`](View::url) prints it back.
pub trait View: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// The application state this view renders with: the router's own.
    type State: Sync;

    /// The view this one is shown in, or [`NoParent`] for the root view.
    type Parent: Parent<Self::State>;

    /// This view's part of the path, which follows its ancestors' parts. The
    /// root view has none. A parameter is a field's name in braces, as in
    /// `"/catalogs/{id}"`.
    ///
    /// A view holds a field for every parameter of its whole path, those of
    /// its ancestors' parts too, under the same names: a view nested in the one
    /// at `"/catalogs/{id}"` has an `id` of its own, or it could not print its
    /// URL. So a parameter's name is used once on the way down.
    ///
    /// The parts are joined as they are, except that a part's last `/` gives
    /// way to a next part that starts with one: a view at `"/activity"` nested
    /// in one at `"/"` is at `/activity`. A view whose whole path is empty is
    /// at `/`.
    const PATH: &'static str = "";

    /// The id of this view's outlet. By default it follows the view's depth:
    /// `outlet-0` for the root view, `outlet-1` for a view nested in it, and
    /// so on.
    const OUTLET: Option<&'static str> = None;

    /// The title of this view, which every view around it receives when it is
    /// shown.
    const TITLE: &'static str = "";

    /// Declared as `async fn render(..) -> impl Into<Html>`, it returns an
    /// engine's output, such as maud markup, as it is.
    fn render(
        &self,
        state: &Self::State,
        outlet: Outlet,
    ) -> impl Future<Output = impl Into<Html>> + Send;

    /// The URL that shows this view: its ancestors' parts of the path and its
    /// own, with the parameters filled from its fields, then its other fields
    /// as the query string. A request to it is parsed back into this value.
    ///
    /// It is the one way to print a view's URL, and not for a view to define.
    ///
    /// # Panics
    ///
    /// If a parameter of the path has no field of that name.
    fn url(&self) -> String {
        url::print(&Self::path(), self)
    }
}

/// The parent of the root view: none. It is not a view.
pub struct NoParent;

/// What a view names as its parent: a [`View`] with the same state, or
/// [`NoParent`]. Nothing else can be one, and naming anything else does not
/// compile:
///
/// ```compile_fail,E0277
/// use htmx_rust::{Html, Outlet, View};
/// use serde::{Deserialize, Serialize};
///
/// struct NotAView;
///
/// #[derive(Serialize, Deserialize)]
/// struct Catalogs;
///
/// impl View for Catalogs {
///     type State = ();
///     type Parent = NotAView;
///
///     async fn render(&self, _: &(), _: Outlet) -> impl Into<Html> {
///         Html::default()
///     }
/// }
/// ```
pub trait Parent<S>: chain::Chain<S> {}

impl<S: Sync> Parent<S> for NoParent {}

impl<V: View> Parent<V::State> for V {}

/// Private, so that only this crate decides what a [`Parent`] is.
mod chain {
    use super::*;

    pub trait Chain<S> {
        /// How many views there are from the root view down to this one.
        const DEPTH: usize;

        /// This view and its ancestors, as a URL says they are.
        type Views: Send + Sync;

        fn path() -> String;

        /// Reads this view and its ancestors from what a URL carries.
        fn parse(fields: &str) -> Result<Self::Views, url::Error>;

        /// Renders this view around `content`, and its ancestors around it.
        fn show(
            views: &Self::Views,
            state: &S,
            content: Html,
            title: &'static str,
        ) -> impl Future<Output = Html> + Send;
    }

    impl<S: Sync> Chain<S> for NoParent {
        const DEPTH: usize = 0;
        type Views = ();

        fn path() -> String {
            String::new()
        }

        fn parse(_: &str) -> Result<(), url::Error> {
            Ok(())
        }

        async fn show(_: &(), _: &S, content: Html, _: &'static str) -> Html {
            content
        }
    }

    impl<V: View> Chain<V::State> for V {
        const DEPTH: usize = V::Parent::DEPTH + 1;
        type Views = (V, <V::Parent as Chain<V::State>>::Views);

        fn path() -> String {
            let parent = V::Parent::path();
            let parent = match V::PATH.starts_with('/') {
                true => parent.strip_suffix('/').unwrap_or(&parent),
                false => &parent,
            };
            let path = parent.to_owned() + V::PATH;
            if path.is_empty() { "/".into() } else { path }
        }

        fn parse(fields: &str) -> Result<Self::Views, url::Error> {
            Ok((url::parse(fields)?, V::Parent::parse(fields)?))
        }

        async fn show(
            (view, ancestors): &Self::Views,
            state: &V::State,
            content: Html,
            title: &'static str,
        ) -> Html {
            let outlet = Outlet {
                id: V::OUTLET.map_or_else(|| format!("outlet-{}", V::Parent::DEPTH), Into::into),
                content,
                title,
            };
            let content = view.render(state, outlet).await.into();
            V::Parent::show(ancestors, state, content, title).await
        }
    }
}

/// The place in a view where the view nested inside it is shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outlet {
    /// The id to give the element that holds [`content`](Outlet::content).
    pub id: String,
    /// The nested view, rendered. Empty when nothing is nested.
    pub content: Html,
    /// The title of the view the URL names: the innermost one.
    pub title: &'static str,
}

/// Registers views on an [`axum::Router`], beside its ordinary routes.
pub trait Views<S> {
    /// Answers GET requests to the view's path with the whole document: the
    /// view in its parent's outlet, and so on up to the root view.
    ///
    /// The view and its ancestors are read from the request's URL first. If
    /// one of them cannot be, because a parameter or the query string does not
    /// parse into its fields, the answer is `400 Bad Request` and no view
    /// renders.
    ///
    /// # Panics
    ///
    /// If the path already answers GET requests, has a parameter that no field
    /// of the view fills, or names a parameter twice.
    fn view<V: View<State = S>>(self) -> Self;
}

impl<S: Clone + Send + Sync + 'static> Views<S> for Router<S> {
    fn view<V: View<State = S>>(self) -> Self {
        url::check::<V>(&V::path());
        self.route(
            &V::path(),
            get(
                |State(state): State<S>, parameters: RawPathParams, RawQuery(query): RawQuery| async move {
                    let views = V::parse(&url::fields(&parameters, query.as_deref()))
                        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?;
                    let document = V::show(&views, &state, Html::default(), V::TITLE).await;
                    Ok::<_, (StatusCode, String)>(axum::response::Html(String::from(document)))
                },
            ),
        )
    }
}
