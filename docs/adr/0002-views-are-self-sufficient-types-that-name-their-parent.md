# Views are self-sufficient types that name their parent

A view is a type whose fields are what its URL carries (path parameters, query string): the library parses a URL into it, and it prints itself back as a URL, so a link built from a view cannot point at a route that does not exist. Each view names the view it is nested in; the shape of the path says nothing about nesting, and `/catalogs` and `/catalogs/{id}` are two separate views, neither inside the other, only because the app declared them so. A view loads its own data and draws everything that is its own, including where it stands among its siblings.

## Considered Options

- **Nesting read from the path** (the first part is one view, each further part a view nested in it): forces a detail view inside the list's frame, which is the app's choice to make.
- **A tree declared top-down in one place**: the parent knows its children for free, but a view's URL would depend on where it was mounted and links could only be checked at run time.
- **Handler functions** as in plain axum: no typed links, and the state of the URL is not a value the app can hold.
- **A nested view holds its parent's value** instead of repeating its parameters (`struct Items { catalog: Catalog }`): one field however deep, but a field would then hold a struct and not a plain value, which serde cannot flatten into typed fields without a parser of our own, and the parent's query string would travel into every link to a nested view.
- **Parents load data and hand it down**, or **parents draw the navigation of their children**: both make a view depend on its parent having run, which it has not when only the nested view is sent ([ADR 1](0001-server-cuts-the-response-by-outlet.md)).

## Consequences

- A view's fields are read and printed with serde, as one list of names and values: a path parameter fills the field it names (`{id}` fills `id`) and the query string fills the others. So each field holds a plain value, or a list of them that the query string carries by repeating the name (`?tag=tea&tag=green`), and a view declares no parser and no printer of its own.
- A view holds a field for every parameter of its whole path, so a nested view repeats its ancestors' parameters under the same names (`Items { id }` under `/catalogs/{id}`): it needs them to load its own data and to print its own URL. A parameter's name is therefore used once on the way down. Every view on the way is parsed from the same fields of the URL. A nested view's printed URL carries only its own fields: an ancestor's field of the query string is in it only if the nested view holds it too.
- A URL that does not parse into the view's fields, or into those of an ancestor, is a client error (`400`), answered before any view runs.
- The compiler checks a link's fields where the link is made. That a path's parameter has a field of the same name is not checked by the compiler, which cannot compare a path's text with a type's fields without a macro. It is checked when the view is registered, against its whole path: the app does not start.
- The parts of a path are joined as written, except that a part's last `/` gives way to a next part that starts with one (`/` then `/activity` is `/activity`, `/` then `recent` is `/recent`), and a whole path that is empty is `/`.
- An outlet's id is `outlet-` and its view's depth, the root view's being `outlet-0`, unless the view names another. It is known without rendering, which the server needs to tell which outlet a request names ([ADR 1](0001-server-cuts-the-response-by-outlet.md)).
- From the registered views the library tells each view its siblings and its children (title, URL, which one is shown), so a view can draw its tabs. Their order is the order of registration.
- On a narrow swap the ancestors do not run. A check that must hold for every view nested in a resource's view (may this reader see it) belongs to the domain or to an axum layer, not to the parent view.
- A parent and its child may each load the same thing on a full load.
