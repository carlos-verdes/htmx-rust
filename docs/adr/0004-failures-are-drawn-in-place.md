# A view's failure is drawn in its place

When a view fails (the catalog does not exist), its failure becomes HTML shown where the view would have been, with the right status code: the layout and any ancestors stay around it on a full load, and htmx 4 swaps it into the same outlet on a click. A failure may instead replace the whole response, for cases such as a redirect to sign-in. The app defines how its error type maps to one or the other.

An empty result is not a failure: a search that finds nothing answers 200 and the view draws "nothing found" as ordinary content.

## Considered Options

- **A generic error page**: loses the navigation around the failure, and shows a different thing on a click than on a refresh.
