use std::future::Future;

use axum::{Router, extract::State, routing::get};

use crate::Html;
use chain::Chain;

/// What one URL path shows, nested in its [`Parent`](View::Parent).
pub trait View: 'static {
    /// The application state this view renders with: the router's own.
    type State: Sync;

    /// The view this one is shown in, or [`NoParent`] for the root view.
    type Parent: Parent<Self::State>;

    /// This view's part of the path, which follows its ancestors' parts. The
    /// root view has none.
    const PATH: &'static str = "";

    /// The title of this view, which every view around it receives when it is
    /// shown.
    const TITLE: &'static str = "";

    /// Declared as `async fn render(..) -> impl Into<Html>`, it returns an
    /// engine's output, such as maud markup, as it is.
    fn render(state: &Self::State, outlet: Outlet) -> impl Future<Output = impl Into<Html>> + Send;
}

/// The parent of the root view: none. It is not a view.
pub struct NoParent;

/// What a view names as its parent: a [`View`] with the same state, or
/// [`NoParent`]. Nothing else can be one.
pub trait Parent<S>: chain::Chain<S> {}

impl<S: Sync> Parent<S> for NoParent {}

impl<V: View> Parent<V::State> for V {}

/// Private, so that only this crate decides what a [`Parent`] is.
mod chain {
    use super::*;

    pub trait Chain<S> {
        /// How many views there are from the root view down to this one.
        const DEPTH: usize;

        fn path() -> String;

        /// Renders this view around `content`, and its ancestors around it.
        fn show(state: &S, content: Html, title: &'static str)
        -> impl Future<Output = Html> + Send;
    }

    impl<S: Sync> Chain<S> for NoParent {
        const DEPTH: usize = 0;

        fn path() -> String {
            String::new()
        }

        async fn show(_: &S, content: Html, _: &'static str) -> Html {
            content
        }
    }

    impl<V: View> Chain<V::State> for V {
        const DEPTH: usize = V::Parent::DEPTH + 1;

        fn path() -> String {
            V::Parent::path() + V::PATH
        }

        async fn show(state: &V::State, content: Html, title: &'static str) -> Html {
            let outlet = Outlet {
                id: format!("outlet-{}", V::Parent::DEPTH),
                content,
                title,
            };
            let content = V::render(state, outlet).await.into();
            V::Parent::show(state, content, title).await
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
    /// # Panics
    ///
    /// If the path already answers GET requests.
    fn view<V: View<State = S>>(self) -> Self;
}

impl<S: Clone + Send + Sync + 'static> Views<S> for Router<S> {
    fn view<V: View<State = S>>(self) -> Self {
        self.route(
            &V::path(),
            get(|State(state): State<S>| async move {
                let document = V::show(&state, Html::default(), V::TITLE).await;
                axum::response::Html(String::from(document))
            }),
        )
    }
}
