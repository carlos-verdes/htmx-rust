/// HTML that a template engine has already rendered and escaped.
///
/// The library passes it along and never builds or changes it. It comes from an
/// engine's output (maud's `Markup`, with the `maud` feature) or from
/// [`Html::trusted`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Html(String);

impl Html {
    /// Marks a string as HTML, as it is: nothing is escaped.
    ///
    /// This is the only way to turn a string into [`Html`], so that every place
    /// that vouches for a string is easy to find.
    pub fn trusted(html: impl Into<String>) -> Self {
        Self(html.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<Html> for String {
    fn from(html: Html) -> Self {
        html.0
    }
}

#[cfg(feature = "maud")]
impl From<maud::Markup> for Html {
    fn from(markup: maud::Markup) -> Self {
        Self(markup.into_string())
    }
}

/// Lets maud markup hold an [`Html`], such as an outlet's content, unescaped.
#[cfg(feature = "maud")]
impl maud::Render for Html {
    fn render_to(&self, buffer: &mut String) {
        buffer.push_str(&self.0);
    }
}
