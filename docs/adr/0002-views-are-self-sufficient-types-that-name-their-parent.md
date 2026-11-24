# Views are self-sufficient types that name their parent

A view is a type whose fields are what its URL carries (path parameters, query string): the library parses a URL into it, and it prints itself back as a URL, so a link built from a view cannot point at a route that does not exist. Each view names the view it is nested in; the shape of the path says nothing about nesting, and `/catalogs` and `/catalogs/{id}` are two separate views, neither inside the other, only because the app declared them so. A view loads its own data and draws everything that is its own, including where it stands among its siblings.

## Considered Options

- **Nesting read from the path** (the first part is one view, each further part a view nested in it): forces a detail view inside the list's frame, which is the app's choice to make.
- **A tree declared top-down in one place**: the parent knows its children for free, but a view's URL would depend on where it was mounted and links could only be checked at run time.
- **Handler functions** as in plain axum: no typed links, and the state of the URL is not a value the app can hold.
- **Parents load data and hand it down**, or **parents draw the navigation of their children**: both make a view depend on its parent having run, which it has not when only the nested view is sent ([ADR 1](0001-server-cuts-the-response-by-outlet.md)).

## Consequences

- A view's fields are read and printed with serde, as one list of names and values: a path parameter fills the field it names (`{id}` fills `id`) and the query string fills the others. So each field holds a plain value, or a list of them that the query string carries by repeating the name (`?tag=tea&tag=green`), and a view declares no parser and no printer of its own.
- A URL that does not parse into the view's fields, or into those of an ancestor, is a client error (`400`), answered before any view runs.
- The compiler checks a link's fields where the link is made. That a path's parameter has a field of the same name is not checked by the compiler, which cannot compare a path's text with a type's fields without a macro. It is checked when the view is registered: the app does not start.
- From the registered views the library tells each view its siblings and its children (title, URL, which one is shown), so a view can draw its tabs. Their order is the order of registration.
- On a narrow swap the ancestors do not run. A check that must hold for every view nested in a resource's view (may this reader see it) belongs to the domain or to an axum layer, not to the parent view.
- A parent and its child may each load the same thing on a full load.
