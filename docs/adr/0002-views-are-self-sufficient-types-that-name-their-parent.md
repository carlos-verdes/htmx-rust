# Views are self-sufficient types that name their parent

A view is a type whose fields are what its URL carries (path parameters, query string): the library parses a URL into it, and it prints itself back as a URL, so a link built from a view cannot point at a route that does not exist. Each view names the view it is nested in; the shape of the path says nothing about nesting, and `/catalogs` and `/catalogs/{id}` are two pages only because the app declared two views. A view loads its own data and draws everything that is its own, including where it stands among its siblings.

## Considered Options

- **Nesting read from the path** (first part is the page, the rest are sections): forces a detail view inside the list's frame, which is the app's choice to make.
- **A tree declared top-down in one place**: the parent knows its children for free, but a view's URL would depend on where it was mounted and links could only be checked at run time.
- **Handler functions** as in plain axum: no typed links, and the state of the URL is not a value the app can hold.
- **Parents load data and hand it down**, or **parents draw the navigation of their children**: both make a section depend on its parent having run, which it has not when only the section is sent ([ADR 1](0001-server-cuts-the-response-by-outlet.md)).

## Consequences

- From the registered views the library tells each view its siblings and its children (title, URL, which one is shown), so a section can draw its tabs. Their order is the order of registration.
- On a narrow swap the ancestors do not run. A check that must hold for every section of a resource (may this reader see it) belongs to the domain or to an axum layer, not to the parent view.
- A parent and its child may each load the same thing on a full load.
