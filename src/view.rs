use std::future::Future;

use axum::{Router, extract::State, routing::get};

use crate::Html;

/// What one URL path shows, nested in its [`Parent`](View::Parent).
pub trait View<S>: 'static {
    /// The view this one is shown in. The layout names [`Root`].
    type Parent: View<S>;

    /// This view's part of the path, which follows its ancestors' parts. The
    /// layout has none.
    const PATH: &'static str = "";

    /// The title of this view, which every view around it receives when it is
    /// shown.
    const TITLE: &'static str = "";

    /// Declared as `async fn render(..) -> impl Into<Html>`, it returns an
    /// engine's output, such as maud markup, as it is.
    fn render(state: &S, outlet: Outlet) -> impl Future<Output = impl Into<Html>> + Send;

    /// How many views there are from the layout down to this one.
    #[doc(hidden)]
    const DEPTH: usize = Self::Parent::DEPTH + 1;

    #[doc(hidden)]
    fn full_path() -> String {
        Self::Parent::full_path() + Self::PATH
    }

    /// Renders this view around `content`, and its ancestors around it.
    #[doc(hidden)]
    fn show(state: &S, content: Html, title: &'static str) -> impl Future<Output = Html> + Send
    where
        S: Sync,
    {
        async move {
            let outlet = Outlet {
                id: format!("outlet-{}", Self::Parent::DEPTH),
                content,
                title,
            };
            let content = Self::render(state, outlet).await.into();
            Self::Parent::show(state, content, title).await
        }
    }
}

/// The view every layout is nested in: the one view with no parent. It shows
/// nothing of its own.
pub struct Root;

impl<S: Sync> View<S> for Root {
    type Parent = Root;
    const DEPTH: usize = 0;

    async fn render(_: &S, outlet: Outlet) -> impl Into<Html> {
        outlet.content
    }

    fn full_path() -> String {
        String::new()
    }

    async fn show(_: &S, content: Html, _: &'static str) -> Html {
        content
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
    /// view in its parent's outlet, and so on up to the layout.
    ///
    /// # Panics
    ///
    /// If the path already answers GET requests.
    fn view<V: View<S>>(self) -> Self;
}

impl<S: Clone + Send + Sync + 'static> Views<S> for Router<S> {
    fn view<V: View<S>>(self) -> Self {
        self.route(
            &V::full_path(),
            get(|State(state): State<S>| async move {
                let document = V::show(&state, Html::default(), V::TITLE).await;
                axum::response::Html(String::from(document))
            }),
        )
    }
}
