# The core emits no markup and passes HTML as `Html`

The core routes and cuts; it writes no HTML of its own and never puts data into HTML, so escaping stays entirely in the template engine. What views return is `Html`, a marker meaning "an engine already rendered and escaped this". An `Html` comes from an engine's conversion (maud's `Markup`, behind the `maud` feature, on by default) or from one deliberately named constructor for trusted strings. The core adds routes to a normal `axum::Router<S>` and wraps nothing.

## Considered Options

- **maud in every signature**: simplest today, but adding a second engine later would break every signature.
- **Plain `String`**: nothing would tell rendered HTML from text a reader typed.
- **A default outermost view in the core**: a new app would work sooner, but the library would be choosing markup and styling. That belongs to a starter the app copies or extends, outside the core.
