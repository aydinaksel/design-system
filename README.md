# Design System

[Leptos](https://leptos.dev) 0.8 components styled with
[Tailwind CSS](https://tailwindcss.com) 4: buttons, dialogs, menus, comboboxes,
form fields, tabs, toasts, and server-driven data tables whose state lives in
the URL. Icons come from [Lucide](https://lucide.dev) through
[lepticons](https://crates.io/crates/lepticons).

| Crate | Purpose |
| --- | --- |
| `design-system` | The components. |
| `table-query` | Serializable table state (search, filters, sorting, paging), shared by the browser and the server functions that answer it. |
| `component-gallery` | A Leptos app that renders every component, for development. |

## Using It

Tailwind only generates classes it finds in the files it scans, so an app has
to import the stylesheet this crate ships, and that needs a path it can rely on.
The simplest is to check this repository out next to the app and depend on it
by path, forwarding the app's features:

```toml
[dependencies]
design-system = { path = "../design-system/crates/design-system" }

[features]
hydrate = ["design-system/hydrate"]
ssr = ["design-system/ssr"]
```

Then import its stylesheet after Tailwind, by the same relative route:

```css
@import "tailwindcss";
@import "../../design-system/crates/design-system/style/design-system.css";
```

For Nix builds, pin this repository as a non-flake input
(`github:aydinaksel/design-system/v0.9.0`) and copy it beside the app's source
before building, so the same relative paths resolve inside the sandbox.

The imported stylesheet adds the crate's sources to Tailwind's scan and
defines the animations the components use.

## Choosing an Editing Pattern

The same kind of edit should look and behave the same on every page, so pick
by what is being edited:

| Editing | Use | Gallery |
| --- | --- | --- |
| One value on an existing table row | `CellEditor`, a popover on the cell | `/cell-editing` |
| A row of values typed in sequence, such as document lines | `LineItems`, edited in place | `/line-items` |
| A record with more fields than its table shows, such as a contact or an address | `FormDialog`, opened from the row | `/dialogs` |
| The page's own header fields | `FieldSection` with `SectionEditing` | `/field-sections` |

A document's lines are always `LineItems`, on a new document and on a posted
one alike. Where a posted document's lines can be corrected but not added to,
pass `existing_lines_only`.

Offer one way to edit a value: a cell with a `CellEditor` does not also get a
row button that opens a dialog for the same field. A save that removes
something asks first with a `ConfirmDialog`, as the container cell on
`/cell-editing` does.

## Gallery

```sh
nix run .#gallery-watch
```

serves the gallery at <http://localhost:3005> and rebuilds on every change.

## Development

`nix develop` provides the toolchain. `just check` runs rustfmt, leptosfmt,
Clippy for the server and the WebAssembly targets, and the tests.

## License

MIT, see [LICENSE](LICENSE).
