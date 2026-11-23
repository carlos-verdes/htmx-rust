# htmx-rust

A library for server-rendered Rust web apps enhanced with htmx, where the URL alone decides what the browser shows.

## Language

**View**:
What one URL path shows: the path names a resource and the view is its representation, complete on its own. A view may be nested inside another view.
_Avoid_: Route, screen, handler

**Outlet**:
The one place in a view where the view nested inside it is shown.
_Avoid_: Slot, frame, placeholder

**Layout**:
The outermost view, the whole document, shared by every page.
_Avoid_: Shell, template, frame

**Page**:
A view shown directly in the layout's outlet.

**Section**:
A view shown in the outlet of a page or of another section.
_Avoid_: Tab, subpage

**Region**:
A part of the interface that is refreshed on its own, without changing the path. Its state lives in the query string, and it can be reused on any page or section.
_Avoid_: Fragment, partial, widget
