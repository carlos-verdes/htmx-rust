use std::future::Future;

use axum::{Router, extract::State, routing::get};

use crate::Html;

/// What one URL path shows, nested in its [`Parent`](View::Parent).
pub trait View<S>: 'static {
    /// The layout this view is shown in.
    type Parent: Layout<S>;

    /// The path this view answers at.
    const PATH: &'static str;

    /// The title of this view, which the layout receives when it is shown.
    const TITLE: &'static str = "";

    fn render(state: &S) -> impl Future<Output = Html> + Send;
}

/// The outermost view: the whole document, around its outlet. It has no path
/// and no parent.
pub trait Layout<S>: 'static {
    /// `title` is the title of the view being shown.
    fn render(state: &S, outlet: Outlet, title: &str) -> impl Future<Output = Html> + Send;
}

/// The place in a view where the view nested inside it is shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outlet {
    /// The id to give the element that holds [`content`](Outlet::content).
    pub id: &'static str,
    /// The nested view, rendered.
    pub content: Html,
}

/// Registers views on an [`axum::Router`], beside its ordinary routes.
pub trait Views<S> {
    /// Answers GET requests to the view's path with the whole document.
    ///
    /// # Panics
    ///
    /// If the path already answers GET requests.
    fn view<V: View<S>>(self) -> Self;
}

impl<S: Clone + Send + Sync + 'static> Views<S> for Router<S> {
    fn view<V: View<S>>(self) -> Self {
        self.route(
            V::PATH,
            get(|State(state): State<S>| async move {
                let outlet = Outlet {
                    id: "outlet-0",
                    content: V::render(&state).await,
                };
                let document = V::Parent::render(&state, outlet, V::TITLE).await;
                axum::response::Html(String::from(document))
            }),
        )
    }
}
