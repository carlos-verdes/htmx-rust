# The server cuts the response by the outlet the request names

One URL must show the same thing on a click and on a refresh. Every in-app link is a plain `<a href>`; with htmx the request names the outlet it will fill (`HX-Target`), and the server answers with only what goes inside that outlet. Without that header (a full load, a refresh, back and forward, no JavaScript) it answers with the whole document. Every response carries `Vary: HX-Request, HX-Target`, because the same URL has more than one body. Only htmx 4 is supported.

## Considered Options

- **The browser cuts** (`hx-select` on a full document): simpler, but every click renders and sends what is thrown away.
- **The server compares the current URL with the requested one** and picks the swap itself (`HX-Current-URL`, `HX-Retarget`): links need no attributes, but the library must match two URLs per request and shared caches cannot store the answers. htmx 4's inherited attributes give attribute-free links more cheaply: the outermost view sets the default target once.
- **htmx 2 as well**: doubles the header logic; `HX-Target` as `tag#id` and `HX-Request-Type` exist only in 4.

## Consequences

- By default a link fills the outermost view's outlet. A view that draws its own navigation may name a narrower outlet on it.
- An outlet needs an id, or the server cannot tell which one was named.
- What is sent for an outlet is exactly the representation of the views inside it, so a section answers a tab click with its own content, navigation included, and nothing of its ancestors.
